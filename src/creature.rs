
use std::cell::RefMut;
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
	creatures::CreatureId,
};

#[derive(Debug, Clone)]
pub struct Creature {
	pub id: CreatureId,
	pub pos: Pos,
	walk_cooldown: Duration,
	attack_cooldown: Duration,
	sprite: Sprite,
	pub inventory: Inventory,
	pub heard_sounds: Vec<(SoundType, String)>,
	activity: Option<Activity>,
	pub plan: Option<Plan>,
	pub name: String,
	pub faction: Faction,
	pub attack: i32,
	health: i32,
	max_health: i32,
	autoheal: Option<AutoHeal>,
	pub mind: Mind,
	pub target: Option<CreatureId>,
	pub home: Pos,
	pub aggro_distance: i32,
	pub give_up_distance: i32,
}

impl Creature {
	
	pub fn load_player(id: CreatureId, saved: PlayerSave) -> Self {
		Self {
			id,
			pos: saved.pos,
			walk_cooldown: Duration(2),
			attack_cooldown: Duration(2),
			sprite: Sprite::PlayerDefault,
			inventory: Inventory::load(saved.inventory),
			heard_sounds: Vec::new(),
			activity: None,
			plan: None,
			name: saved.name,
			faction: Faction::Player,
			health: saved.health,
			max_health: 100,
			attack: 5,
			autoheal: Some(AutoHeal { cooldown: Duration(600), amount: 1, next: None }),
			mind: Mind::Player,
			target: None,
			home: saved.pos,
			aggro_distance: -1,
			give_up_distance: 16,
		}
	}

	pub fn spawn_npc(id: CreatureId, pos: Pos, npc: Npc) -> Self {
		Self {
			id,
			pos,
			walk_cooldown: npc.walk_cooldown(),
			attack_cooldown: npc.attack_cooldown(),
			sprite: npc.sprite(),
			inventory: Inventory::empty(),
			heard_sounds: Vec::new(),
			activity: None,
			plan: None,
			name: npc.name().to_string(),
			faction: npc.faction(),
			health: npc.health(),
			max_health: npc.health(),
			attack: npc.attack(),
			autoheal: None,
			mind: npc.mind(),
			target: None,
			home: pos,
			aggro_distance: npc.aggro_distance(),
			give_up_distance: npc.give_up_distance(),
		}
	}
	
	pub fn is_dead(&self) -> bool {
		self.health <= 0
	}

	pub fn attack(&mut self, mut opponent: RefMut<Creature>, time: Timestamp) {
		opponent.health -= self.attack;
		self.activity = Some(Activity {
			typ: ActivityType::Attack(opponent.pos),
			start: time,
			end: time + self.attack_cooldown
		});
	}
	
	pub fn save(&self) -> PlayerSave {
		PlayerSave {
			name: self.name.clone(),
			pos: self.pos,
			inventory: self.inventory.save(),
			health: self.health.max(0),
		}
	}

	pub fn view(&self) -> CreatureView {
		CreatureView {
			pos: self.pos,
			sprite: self.sprite,
			activity: self.activity.clone(),
			health: self.health.clamp(0, self.max_health),
			max_health: self.max_health,
		}
	}

	pub fn move_to(&mut self, newpos: Pos, time: Timestamp) {
		self.activity = Some(Activity {
			typ: ActivityType::Walk(self.pos),
			start: time,
			end: time + self.walk_cooldown
		});
		self.pos = newpos;
	}

	pub fn current_activity(&self, time: Timestamp) -> Option<Activity> {
		if time < self.activity.as_ref()?.end {
			self.activity.clone()
		} else {
			None
		}
	}

	pub fn can_act(&self, time: Timestamp) -> bool {
		if let Some(activity) = &self.activity {
			time >= activity.end
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
	#[serde(skip_serializing_if = "Option::is_none", rename="a")]
	pub activity: Option<Activity>,
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
pub struct Activity {
	#[serde(flatten)]
	pub typ: ActivityType,
	#[serde(rename = "s")]
	pub start: Timestamp,
	#[serde(rename = "e")]
	pub end: Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub enum ActivityType {
	#[serde(rename = "M")]
	Walk(Pos),
	#[serde(rename = "F")]
	Attack(Pos),
}




#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Assoc, Serialize, Deserialize)]
#[func(fn sprite(&self) -> Sprite)]
#[func(fn name(&self) -> &str)]
#[func(fn faction(&self) -> Faction {Faction::Neutral})]
#[func(fn health(&self) -> i32 {1})]
#[func(fn attack(&self) -> i32 {0})]
#[func(fn mind(&self) -> Mind {Mind::Idle})]
#[func(fn aggro_distance(&self) -> i32 {-1})]
#[func(fn give_up_distance(&self) -> i32 {-1})]
#[func(fn walk_cooldown(&self) -> Duration {Duration(10)})]
#[func(fn attack_cooldown(&self) -> Duration {Duration(100)})]
pub enum Npc {
	#[assoc(name = "Frog")]
	#[assoc(sprite = Sprite::Frog)]
	#[assoc(walk_cooldown = Duration(5))]
	Frog,
	#[assoc(name = "Worm")]
	#[assoc(sprite = Sprite::Worm)]
	#[assoc(mind = Mind::Aggressive)]
	#[assoc(walk_cooldown = Duration(5))]
	#[assoc(attack_cooldown = Duration(10))]
	#[assoc(faction = Faction::Evil)]
	#[assoc(health = 12)]
	#[assoc(attack = 6)]
	#[assoc(aggro_distance = 4)]
	#[assoc(give_up_distance = 16)]
	Worm
}


#[derive(Debug, Clone)]
struct AutoHeal {
	cooldown: Duration,
	amount: i32,
	next: Option<Timestamp>
}

#[derive(Debug, Clone)]
pub enum Mind {
	Player,
	Idle,
	Aggressive
}
