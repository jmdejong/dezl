
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
	world::{CreatureMap, CreatureTile},
	map::Map,
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
	mortal: bool,
	is_dead: bool,
	movement: Option<Direction>,
	path: Vec<Pos>,
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
			mortal: false,
			is_dead: false,
			movement: None,
			path: Vec::new(),
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
			mortal: true,
			is_dead: false,
			movement: None,
			path: Vec::new(),
		}
	}
	
	pub fn is_dead(&self) -> bool {
		self.is_dead
	}

	pub fn attack(&mut self, mut opponent: RefMut<Creature>, time: Timestamp) {
		self.target = Some(opponent.id);
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
				rind: random::randomize_u32(random::randomize_pos(self.pos) + 37 * random::randomize_pos(self.home) + 17 * time.random_seed()),
				by: self.id
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

	pub fn is_dying(&self, tick: Timestamp) -> bool {
		self.activity.as_ref().is_some_and(|activity| matches!(activity.typ, ActivityType::Die(_)) && activity.is_active(tick))
	}

	pub fn view(&self) -> CreatureView {
		CreatureView {
			id: self.id,
			pos: self.pos,
			sprite: self.sprite,
			blocking: self.blocking,
			activity: self.activity.clone(),
			health: (self.health.max(0), self.max_health),
			wounds: self.wounds.iter().rev().cloned().collect(),
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
			Control::Plan(plan) => {
				self.plan = Some(plan);
				self.path = Vec::new();
			}
			Control::Direct(DirectChange::MoveItem(from, target)) => self.inventory.move_item(from, target),
			Control::Direct(DirectChange::Movement(Some(direction))) => {
				self.plan = Some(Plan::Move(direction));
				self.movement = Some(direction);
			},
			Control::Direct(DirectChange::Movement(None)) => {
				self.movement = None;
			}
			Control::Direct(DirectChange::Path(mut path)) => {
				path.retain(|p| self.pos.distance_to(*p) <= 64);
				path.truncate(32);
				if let Some(idx) = path.iter().position(|p| *p == self.pos) {
					path.drain(..(idx+1));
				}
				self.path = path;
			}
		}
	}

	pub fn hear(&mut self, typ: SoundType, text: String) {
		self.heard_sounds.push((typ, text));
	}

	pub fn reset(&mut self, time: Timestamp) {
		self.heard_sounds = Vec::new();
		if self.activity.as_ref().is_some_and(|activity| time > activity.end) {
			self.activity = None;
		}
		self.wounds.retain(|wound| time - wound.time <= Duration(10));
	}

	pub fn update(&mut self, now: Timestamp) {
		if self.mortal && self.health <= 0 {
			self.is_dead = true;
			self.activity = Some(Activity {
				typ: ActivityType::Die(true),
				start: now,
				end: now + Duration(10)
			});
			return;
		}

		if let Some(autoheal) = &mut self.autoheal {
			if autoheal.next.is_some_and(|next| now >= next) {
				self.health = (self.health + autoheal.amount).min(self.max_health).max(self.health).max(0);
				autoheal.next = None;
			}
			if autoheal.next.is_none() && self.health < self.max_health {
				autoheal.next = Some(now + autoheal.cooldown);
			}
		}
	}

	pub fn plan(&mut self, creature_map: &CreatureMap, map: &Map, time: Timestamp) {
		let ct = CreatureTile::new(self);
		let can_walk = |d: &Direction| {
			let p = self.pos + *d;
			!creature_map.blocking(p, &ct) && !map.cell(p).blocking()
		};
		let rind = random::randomize_u32(random::randomize_pos(self.home) + random::randomize_pos(self.pos) + time.0 as u32);
		match self.mind {
			Mind::Player => {
				if self.plan.is_none() {
					if let Some(direction) = self.movement {
						self.plan = Some(Plan::Move(direction));
						self.path = Vec::new();
						return;
					}
					while let Some(path_next) = self.path.first() {
						if *path_next == self.pos {
							self.path.remove(0);
							continue;
						}
						let directions: Vec<Direction> = self.pos.directions_to(*path_next)
							.into_iter()
							.filter(can_walk)
							.collect();
						if directions.is_empty() {
							break;
						}
						let direction = *random::pick(random::randomize_u32(rind + 2849), &directions);
						self.plan = Some(Plan::Move(direction));
						return
					}

					if self.target.is_none() {
						for wound in self.wounds.iter().rev() {
							let age = time - wound.time;
							if age >= Duration(2) {
								self.target = Some(wound.by);
							}
						}
					}
					if let Some(target_id) = self.target {
						let Some(target) = creature_map.get_creature(&target_id) else {
							self.target = None;
							return;
						};
						if self.pos.distance_to(target.pos) > 1 {
							self.target = None;
							return;
						}
						self.plan = Some(Plan::Fight(self.pos.directions_to(target.pos).first().cloned()));
					}
				}
			},
			Mind::Idle => {
				if random::percentage(rind + 543, 10) {
					let directions = if self.pos != self.home && random::percentage(rind + 471, 10) {
							self.pos.directions_to(self.home)
						} else {
							vec![Direction::North, Direction::South, Direction::East, Direction::West]
						};
					let direction = *random::pick(random::randomize_u32(rind + 385), &directions);
					let control = Plan::Move(direction);
					self.plan = Some(control);
				}
			}
			Mind::Aggressive => {
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
					if self.pos.distance_to(target.pos) <= 1 {
						self.plan = Some(Plan::Fight(self.pos.directions_to(target.pos).first().cloned()));
					} else {
						let mut directions: Vec<Direction> = self.pos.directions_to(target.pos)
							.into_iter()
							.filter(can_walk)
							.collect();
						if directions.is_empty() {
							directions = Direction::DIRECTIONS
								.into_iter()
								.filter(can_walk)
								.collect();
							if directions.is_empty() {
								return;
							}
						}
						let direction = *random::pick(random::randomize_u32(rind + 386), &directions);
						self.plan = Some(Plan::Move(direction));
					}
				} else if random::percentage(rind + 543, 10) {
					let directions = if self.pos != self.home && random::percentage(rind + 471, 10) {
							self.pos.directions_to(self.home)
						} else {
							vec![Direction::North, Direction::South, Direction::East, Direction::West]
						};
					let direction = *random::pick(random::randomize_u32(rind + 385), &directions);
					self.plan = Some(Plan::Move(direction));
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
	pub health: (i32, i32),
	#[serde(rename = "w", skip_serializing_if = "Vec::is_empty")]
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
	#[serde(rename = "D")]
	Die(bool),
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
impl Activity {
	pub fn is_active(&self, tick: Timestamp) -> bool {
		tick <= self.end
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct Wound {
	#[serde(rename="d")]
	damage: i32,
	#[serde(rename="t")]
	time: Timestamp,
	#[serde(rename="r")]
	rind: u32,
	#[serde(rename="b")]
	by: CreatureId,
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
	#[assoc(give_up_distance = 10)]
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

