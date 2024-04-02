

use serde::{Serialize, Deserialize};
use enum_assoc::Assoc;
use crate::{
	sprite::Sprite,
	Pos,
	timestamp::Duration,
	inventory::{Inventory, InventorySave},
	worldmessages::SoundType,
	timestamp::Timestamp,
	controls::{Control, Plan, DirectChange},
};

#[derive(Debug, Clone)]
pub struct Creature {
	pub pos: Pos,
	walk_cooldown: Duration,
	sprite: Sprite,
	pub inventory: Inventory,
	pub heard_sounds: Vec<(SoundType, String)>,
	movement: Option<Movement>,
	pub plan: Option<Plan>,
	pub name: String,
	pub faction: Faction,
	pub attack: i32,
	health: i32,
	max_health: i32,
	autoheal: Option<AutoHeal>,
}

impl Creature {
	
	pub fn load_player(saved: PlayerSave) -> Self {
		Self {
			pos: saved.pos,
			walk_cooldown: Duration(2),
			sprite: Sprite::PlayerDefault,
			inventory: Inventory::load(saved.inventory),
			heard_sounds: Vec::new(),
			movement: None,
			plan: None,
			name: saved.name,
			faction: Faction::Player,
			health: saved.health,
			max_health: 100,
			attack: 5,
			autoheal: Some(AutoHeal { cooldown: Duration(600), amount: 1, next: None }),
		}
	}

	pub fn spawn_npc(spawn_id: SpawnId, npc: Npc) -> Self {
		Self {
			pos: spawn_id.0,
			walk_cooldown: Duration(5),
			sprite: npc.sprite(),
			inventory: Inventory::empty(),
			heard_sounds: Vec::new(),
			movement: None,
			plan: None,
			name: npc.name().to_string(),
			faction: npc.faction(),
			health: npc.health(),
			max_health: npc.health(),
			attack: npc.attack(),
			autoheal: None
		}
	}
	
	pub fn is_dead(&self) -> bool {
		self.health <= 0
	}

	pub fn damage(&mut self, amount: i32) {
		self.health -= amount;
	}
	
	pub fn save(&self) -> PlayerSave {
		PlayerSave {
			name: self.name.clone(),
			pos: self.pos,
			inventory: self.inventory.save(),
			health: self.health,
		}
	}

	pub fn view(&self) -> CreatureView {
		CreatureView {
			pos: self.pos,
			sprite: self.sprite,
			movement: self.movement.clone(),
			health: self.health.clamp(0, self.max_health),
			max_health: self.max_health,
		}
	}

	pub fn move_to(&mut self, newpos: Pos, time: Timestamp) {
		self.movement = Some(Movement {
			from: self.pos,
			start: time,
			end: time + self.walk_cooldown
		});
		self.pos = newpos;
	}

	pub fn current_movement(&self, time: Timestamp) -> Option<Movement> {
		if time < self.movement.as_ref()?.end {
			self.movement.clone()
		} else {
			None
		}
	}

	pub fn can_move(&self, time: Timestamp) -> bool {
		if let Some(movement) = &self.movement {
			time >= movement.end
		} else {
			true
		}
	}

	pub fn control(&mut self, control: Control) {
		match control {
			Control::Plan(plan) => self.plan = Some(plan),
			Control::Direct(DirectChange::MoveItem(from, target)) => self.inventory.move_item(from, target),
		}
	}

	pub fn hear(&mut self, typ: SoundType, text: String) {
		self.heard_sounds.push((typ, text))
	}

	pub fn reset(&mut self) {
		if let Some(Plan::Movement(_)) = self.plan {
		} else {
			self.plan = None;
		}
		self.heard_sounds = Vec::new()
	}

	pub fn autoheal_tick(&mut self, now: Timestamp) {
		if let Some(autoheal) = &mut self.autoheal {
			if autoheal.next.is_some_and(|next| now >= next) {
				self.health = (self.health + autoheal.amount).min(self.max_health).max(self.health);
				autoheal.next = None;
			}
			if autoheal.next.is_none() && self.health < self.max_health {
				autoheal.next = Some(now + autoheal.cooldown);
			}
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerSave {
	pub name: String,
	pub pos: Pos,
	pub inventory: InventorySave,
	#[serde(default="one")]
	pub health: i32,
}
fn one() -> i32 {1}

impl PlayerSave {
	pub fn new(name: String, pos: Pos) -> Self {
		Self {
			name,
			pos,
			inventory: Vec::new(),
			health: 100,
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureView {
	#[serde(rename = "s")]
	pub sprite: Sprite,
	#[serde(rename = "p")]
	pub pos: Pos,
	#[serde(skip_serializing_if = "Option::is_none", rename="m")]
	pub movement: Option<Movement>,
	#[serde(rename = "h")]
	pub health: i32,
	#[serde(rename = "hh")]
	pub max_health: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Faction {
	Player,
	Neutral,
	Evil
}

impl Faction {
	pub fn is_enemy(&self, other: Faction) -> bool {
		matches!(
			(self, other),
			(Faction::Player, Faction::Evil) | (Faction::Evil, Faction::Player)
		)
	}
}


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct Movement {
	#[serde(rename = "f")]
	pub from: Pos,
	#[serde(rename = "s")]
	pub start: Timestamp,
	#[serde(rename = "e")]
	pub end: Timestamp,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SpawnId(pub Pos);


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Assoc, Serialize, Deserialize)]
#[func(fn sprite(&self) -> Sprite)]
#[func(fn name(&self) -> &str)]
#[func(fn faction(&self) -> Faction {Faction::Neutral})]
#[func(fn health(&self) -> i32 {1})]
#[func(fn attack(&self) -> i32 {0})]
pub enum Npc {
	#[assoc(name = "Frog")]
	#[assoc(sprite = Sprite::Frog)]
	Frog,
	#[assoc(name = "Worm")]
	#[assoc(sprite = Sprite::Worm)]
	#[assoc(faction = Faction::Evil)]
	#[assoc(health = 12)]
	#[assoc(attack = 6)]
	Worm
}


#[derive(Debug, Clone)]
struct AutoHeal {
	cooldown: Duration,
	amount: i32,
	next: Option<Timestamp>
}

