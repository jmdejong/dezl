
use std::cell::RefMut;
use core::ops::Not;
use serde::{Serialize, Deserialize};
use enum_assoc::Assoc;
use crate::{
	sprite::Sprite,
	pos::{Pos, Direction},
	timestamp::Duration,
	inventory::{Inventory, InventorySave},
	worldmessages::SoundType,
	timestamp::Timestamp,
	controls::{Control, Plan, DirectChange},
	creatures::CreatureId,
	world::CreatureMap,
	random,
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
	pub blocking: bool,
	pub faction: Faction,
	attack: i32,
	health: i32,
	max_health: i32,
	autoheal: Option<AutoHeal>,
	wounds: Vec<Wound>,
	mind: Mind,
	target: Option<CreatureId>,
	home: Pos,
	aggro_distance: i32,
	give_up_distance: i32,
}

impl Creature {
	
	pub fn load_player(id: CreatureId, saved: PlayerSave) -> Self {
		Self {
			id,
			pos: saved.pos,
			walk_cooldown: Duration(2),
			attack_cooldown: Duration(10),
			sprite: Sprite::PlayerDefault,
			inventory: Inventory::load(saved.inventory),
			heard_sounds: Vec::new(),
			activity: None,
			plan: None,
			name: saved.name,
			blocking: false,
			faction: Faction::Player,
			health: saved.health,
			max_health: 100,
			wounds: Vec::new(),
			attack: 5,
			autoheal: Some(AutoHeal { cooldown: Duration(100), amount: 1, next: None }),
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
			blocking: npc.blocking(),
			faction: npc.faction(),
			health: npc.health(),
			max_health: npc.health(),
			wounds: Vec::new(),
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
		let damage = self.attack;
		opponent.health -= damage;
		self.activity = Some(Activity {
			typ: ActivityType::Attack{ target: opponent.pos, damage },
			start: time,
			end: time + self.attack_cooldown
		});
		opponent.wounds.push(
			Wound {
				damage,
				time,
				rind: random::randomize_u32(random::randomize_pos(self.pos) + 37 * random::randomize_pos(self.home) + 17 * time.random_seed())
			}
		);
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
			id: self.id,
			pos: self.pos,
			sprite: self.sprite,
			blocking: self.blocking,
			activity: self.activity.clone(),
			health: self.health.max(0),
			max_health: self.max_health,
			wounds: self.wounds.clone(),
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
		self.heard_sounds.push((typ, text));
	}

	pub fn reset(&mut self, time: Timestamp) {
		if let Some(Plan::Movement(_)) = self.plan {
		} else {
			self.plan = None;
		}
		self.heard_sounds = Vec::new();
		if self.activity.as_ref().is_some_and(|activity| time > activity.end) {
			self.activity = None;
		}
		self.wounds.retain(|wound| time - wound.time <= Duration(10));
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

	pub fn plan(&mut self, creature_map: &CreatureMap, time: Timestamp) {
		let home_pos = self.home;
			match self.mind {
				Mind::Player => {},
				Mind::Idle => {
					let rind = random::randomize_u32(random::randomize_pos(home_pos) + time.0 as u32);
					if random::percentage(rind + 543, 10) {
						let directions = if self.pos != home_pos && random::percentage(rind + 471, 10) {
								self.pos.directions_to(home_pos)
							} else {
								vec![Direction::North, Direction::South, Direction::East, Direction::West]
							};
						let direction = *random::pick(random::randomize_u32(rind + 385), &directions);
						let control = Plan::Move(direction);
						self.control(Control::Plan(control));
					}
				}
				Mind::Aggressive => {
					let rind = random::randomize_u32(random::randomize_pos(home_pos) + time.0 as u32);
					if let Some(target_id) = self.target {
						if let Some(target) = creature_map.get_creature(&target_id) {
							if self.pos.distance_to(target.pos) > self.give_up_distance {
								self.target = None;
							}
						} else  {
							self.target = None;
						}
					}
					if self.target.is_none() {
						self.target = creature_map.nearby(self.pos, self.aggro_distance)
							.filter(|other| self.faction.is_enemy(other.faction))
							.min_by_key(|other| self.pos.distance_to(other.pos))
							.map(|other| other.id);
					}
					if let Some(target_id) = self.target {
						let target = creature_map.get_creature(&target_id).unwrap();
						if self.pos == target.pos {
							self.control(Control::Plan(Plan::Fight(None)));
						} else if self.pos.distance_to(target.pos) == 1 {
							let direction: Direction = self.pos.directions_to(target.pos)[0];
							self.control(Control::Plan(Plan::Fight(Some(direction))));
						} else {
							let directions = self.pos.directions_to(target.pos);
							let direction = *random::pick(random::randomize_u32(rind + 385), &directions);
							self.control(Control::Plan(Plan::Move(direction)));
						}
					} else if random::percentage(rind + 543, 10) {
						let directions = if self.pos != home_pos && random::percentage(rind + 471, 10) {
								self.pos.directions_to(home_pos)
							} else {
								vec![Direction::North, Direction::South, Direction::East, Direction::West]
							};
						let direction = *random::pick(random::randomize_u32(rind + 385), &directions);
						self.control(Control::Plan(Plan::Move(direction)));
					}
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
	#[serde(rename = "i")]
	pub id: CreatureId,
	#[serde(rename = "s")]
	pub sprite: Sprite,
	#[serde(rename = "p")]
	pub pos: Pos,
	#[serde(rename="a", skip_serializing_if = "Option::is_none")]
	pub activity: Option<Activity>,
	#[serde(rename = "h")]
	pub health: i32,
	#[serde(rename = "hh")]
	pub max_health: i32,
	#[serde(rename = "w")]
	pub wounds: Vec<Wound>,
	#[serde(rename = "b", skip_serializing_if = "Not::not")]
	pub blocking: bool,
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
	Attack{
		#[serde(rename="t")]
		target: Pos,
		#[serde(rename="d")]
		damage: i32,
	},
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct Wound {
	#[serde(rename="d")]
	damage: i32,
	#[serde(rename="t")]
	time: Timestamp,
	#[serde(rename="r")]
	rind: u32
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
#[func(fn blocking(&self) -> bool {false})]
pub enum Npc {
	#[assoc(name = "Frog")]
	#[assoc(sprite = Sprite::Frog)]
	#[assoc(walk_cooldown = Duration(5))]
	Frog,
	#[assoc(name = "Worm")]
	#[assoc(sprite = Sprite::Worm)]
	#[assoc(mind = Mind::Aggressive)]
	#[assoc(blocking = true)]
	#[assoc(walk_cooldown = Duration(5))]
	#[assoc(attack_cooldown = Duration(20))]
	#[assoc(faction = Faction::Evil)]
	#[assoc(health = 12)]
	#[assoc(attack = 3)]
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

