

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
	is_dead: bool,
	movement: Option<Movement>,
	pub plan: Option<Plan>,
	pub name: String,
	pub faction: Faction,
}

impl Creature {
	
	pub fn load_player(saved: PlayerSave) -> Self {
		Self {
			pos: saved.pos,
			walk_cooldown: Duration(2),
			sprite: Sprite::PlayerDefault,
			inventory: Inventory::load(saved.inventory),
			heard_sounds: Vec::new(),
			is_dead: false,
			movement: None,
			plan: None,
			name: saved.name,
			faction: Faction::Player
		}
	}

	pub fn spawn_npc(spawn_id: SpawnId, npc: Npc) -> Self {
		Self {
			pos: spawn_id.0,
			walk_cooldown: Duration(5),
			sprite: npc.sprite(),
			inventory: Inventory::load(Vec::new()),
			heard_sounds: Vec::new(),
			is_dead: false,
			movement: None,
			plan: None,
			name: npc.name().to_string(),
			faction: npc.faction()
		}
	}
	
	pub fn kill(&mut self) {
		self.is_dead = true;
	}
	
	pub fn save(&self) -> PlayerSave {
		PlayerSave {
			name: self.name.clone(),
			pos: self.pos,
			inventory: self.inventory.save()
		}
	}

	pub fn view(&self) -> CreatureView {
		CreatureView {
			pos: self.pos,
			sprite: self.sprite,
			movement: self.movement.clone(),
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerSave {
	pub name: String,
	pub pos: Pos,
	pub inventory: InventorySave,
}

impl PlayerSave {
	pub fn new(name: String, pos: Pos) -> Self {
		Self {
			name,
			pos,
			inventory: Vec::new()
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
pub enum Npc {
	#[assoc(sprite = Sprite::Frog)]
	#[assoc(name = "Frog")]
	Frog,
	#[assoc(sprite = Sprite::Worm)]
	#[assoc(faction = Faction::Evil)]
	#[assoc(name = "Worm")]
	Worm
}
