
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
	creaturemap::{CreatureMap, CreatureTile},
	map::Map,
	random,
};

#[derive(Debug, Clone)]
pub struct Creature {
	typ: CreatureType,
	pub id: CreatureId,
	pub pos: Pos,
	pub inventory: Inventory,
	pub heard_sounds: Vec<(SoundType, String)>,
	activity: Option<Activity>,
	pub plan: Option<Plan>,
	pub name: String,
	health: i32,
	last_autoheal: Timestamp,
	wounds: Vec<Wound>,
	target: Option<CreatureId>,
	home: Pos,
	is_dead: bool,
	movement: Option<Direction>,
	path: Vec<Pos>,
}

impl Creature {

	pub fn spawn_npc(id: CreatureId, pos: Pos, typ: CreatureType) -> Self {
		Self {
			typ,
			id,
			pos,
			inventory: Inventory::empty(),
			heard_sounds: Vec::new(),
			activity: None,
			plan: None,
			name: typ.name().to_string(),
			health: typ.health(),
			wounds: Vec::new(),
			target: None,
			home: pos,
			is_dead: false,
			movement: None,
			path: Vec::new(),
			last_autoheal: Timestamp::zero(),
		}
	}

	pub fn load_player(id: CreatureId, saved: PlayerSave) -> Self {
		Self {
			name: saved.name,
			inventory: Inventory::load(saved.inventory),
			health: saved.health,
			..Self::spawn_npc(id, saved.pos, CreatureType::Player)
		}
	}

	pub fn is_dead(&self) -> bool {
		self.is_dead
	}

	pub fn attack(&mut self, mut opponent: RefMut<Creature>, time: Timestamp) {
		self.target = Some(opponent.id);
		let damage = self.typ.attack();
		opponent.health -= damage;
		self.activity = Some(Activity {
			typ: ActivityType::Attack{ target: opponent.pos, damage },
			start: time,
			end: time + self.typ.attack_cooldown()
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

	pub fn move_to(&mut self, newpos: Pos, time: Timestamp) {
		self.activity = Some(Activity {
			typ: ActivityType::Walk(self.pos),
			start: time,
			end: time + self.typ.walk_cooldown()
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
				self.path = Vec::new()
			},
			Control::Direct(DirectChange::Movement(None)) => {
				self.movement = None;
			}
			Control::Direct(DirectChange::Path(mut path)) => {
				path.truncate(16);
				path.retain(|p| self.pos.distance_to(*p) <= 32);
				self.path = path;
				self.movement = None;
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
		if self.typ.mortal() && self.health <= 0 {
			self.is_dead = true;
			self.activity = Some(Activity {
				typ: ActivityType::Die(true),
				start: now,
				end: now + Duration(10)
			});
			return;
		}

		if self.health >= self.typ.health() {
			self.last_autoheal = Timestamp::zero();
		} else if let Some(autoheal) = self.typ.autoheal() {
			let next_autoheal = self.last_autoheal + autoheal.cooldown;
			if now == next_autoheal {
				self.health = (self.health + autoheal.amount).min(self.typ.health()).max(self.health).max(0);
			}
			if now >= next_autoheal {
				self.last_autoheal = now;
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
		match self.typ.mind() {
			Mind::Player => {
				if self.plan.is_none() {
					if let Some(direction) = self.movement {
						self.plan = Some(Plan::Move(direction));
						self.path = Vec::new();
						return;
					}
					if let Some(idx) = self.path.iter().position(|p| *p == self.pos) {
						self.path.drain(..(idx+1));
					}
					if let Some(path_next) = self.path.first() {
						let directions: Vec<Direction> = self.pos.directions_to(*path_next)
							.into_iter()
							.filter(can_walk)
							.collect();
						if !directions.is_empty() {
							let direction = *random::pick(random::randomize_u32(rind + 2849), &directions);
							self.plan = Some(Plan::Move(direction));
							return;
						}
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
						if self.pos.distance_to(target.pos) > self.typ.give_up_distance() {
							self.target = None;
						}
					} else  {
						self.target = None;
					}
				}
				if self.target.is_none() {
					self.target = creature_map.nearby(self.pos, self.typ.aggro_distance())
						.filter(|other| self.faction().is_enemy(other.faction))
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

	pub fn blocking(&self) -> bool {
		self.typ.blocking()
	}

	pub fn faction(&self) -> Faction {
		self.typ.faction()
	}

	pub fn view(&self) -> CreatureView {
		CreatureView {
			id: self.id,
			pos: self.pos,
			sprite: self.typ.sprite(),
			blocking: self.blocking(),
			activity: self.activity.clone(),
			health: (self.health.max(0), self.typ.health()),
			wounds: self.wounds.iter().rev().cloned().collect(),
			walk_speed: None,
		}
	}

	pub fn view_ext(&self) -> CreatureView {
		CreatureView {
			walk_speed: Some((1, self.typ.walk_cooldown())),
			..self.view()
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerSave {
	name: String,
	pos: Pos,
	inventory: InventorySave,
	#[serde(default="one")]
	health: i32,
}
fn one() -> i32 {1}

impl PlayerSave {
	pub fn new(name: String, pos: Pos) -> Self {
		Self {
			name,
			pos,
			inventory: Vec::new(),
			health: CreatureType::Player.health(),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureView {
	#[serde(rename = "i")]
	id: CreatureId,
	#[serde(rename = "s")]
	sprite: Sprite,
	#[serde(rename = "p")]
	pos: Pos,
	#[serde(rename="a", skip_serializing_if = "Option::is_none")]
	activity: Option<Activity>,
	#[serde(rename = "h")]
	health: (i32, i32),
	#[serde(rename = "w", skip_serializing_if = "Vec::is_empty")]
	wounds: Vec<Wound>,
	#[serde(rename = "b", skip_serializing_if = "Not::not")]
	blocking: bool,
	#[serde(rename = "v", skip_serializing_if = "Option::is_none")]
	walk_speed: Option<(i32, Duration)>,
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
enum ActivityType {
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
struct Activity {
	#[serde(flatten)]
	typ: ActivityType,
	#[serde(rename = "s")]
	start: Timestamp,
	#[serde(rename = "e")]
	end: Timestamp,
}
impl Activity {
	fn is_active(&self, tick: Timestamp) -> bool {
		tick <= self.end
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
struct Wound {
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
#[func(fn sprite(self) -> Sprite)]
#[func(fn name(&self) -> &str)]
#[func(fn faction(self) -> Faction {Faction::Neutral})]
#[func(fn health(self) -> i32 {1})]
#[func(fn attack(self) -> i32 {0})]
#[func(fn mind(self) -> Mind {Mind::Idle})]
#[func(fn aggro_distance(self) -> i32 {-1})]
#[func(fn give_up_distance(self) -> i32 {-1})]
#[func(fn walk_cooldown(self) -> Duration {Duration(10)})]
#[func(fn attack_cooldown(self) -> Duration {Duration(100)})]
#[func(fn blocking(self) -> bool {false})]
#[func(fn mortal(self) -> bool {true})]
#[func(fn autoheal(self) -> Option<AutoHeal>)]
pub enum CreatureType {
	#[assoc(name = "Player")]
	#[assoc(sprite = Sprite::PlayerDefault)]
	#[assoc(mind = Mind::Player)]
	#[assoc(walk_cooldown = Duration(2))]
	#[assoc(attack_cooldown = Duration(10))]
	#[assoc(faction = Faction::Player)]
	#[assoc(blocking = false)]
	#[assoc(attack = 5)]
	#[assoc(health = 100)]
	#[assoc(mortal = false)]
	#[assoc(autoheal = AutoHeal {cooldown: Duration(100), amount: 1})]
	Player,
	#[assoc(name = "Frog")]
	#[assoc(sprite = Sprite::Frog)]
	#[assoc(walk_cooldown = Duration(5))]
	Frog,
	#[assoc(name = "Worm")]
	#[assoc(sprite = Sprite::Worm)]
	#[assoc(mind = Mind::Aggressive)]
	#[assoc(blocking = true)]
	#[assoc(walk_cooldown = Duration(5))]
	#[assoc(attack_cooldown = Duration(15))]
	#[assoc(faction = Faction::Evil)]
	#[assoc(health = 12)]
	#[assoc(attack = 2)]
	#[assoc(aggro_distance = 4)]
	#[assoc(give_up_distance = 10)]
	Worm
}


#[derive(Debug, Clone)]
struct AutoHeal {
	cooldown: Duration,
	amount: i32,
}

#[derive(Debug, Clone)]
pub enum Mind {
	Player,
	Idle,
	Aggressive
}

