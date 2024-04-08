
use std::collections::{HashMap};
use std::cell::{RefCell, Ref, RefMut};
use serde::{Serialize, Serializer};

use crate::{
	PlayerId,
	controls::{Control},
	pos::Pos,
	creature::{Creature, PlayerSave, CreatureType as Npc},
	loadedareas::LoadedAreas,
	timestamp::{Timestamp, Duration},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpawnId(pub Pos);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CreatureId {
	Player(PlayerId),
	Spawned(SpawnId),
}

impl CreatureId {
	pub fn player(&self) -> Option<&PlayerId> {
		if let Self::Player(id) = self {
			Some(id)
		} else {
			None
		}
	}
}

impl Serialize for CreatureId {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where S: Serializer {
		match self {
			Self::Player(id) => format!("p-{}", id.name()),
			Self::Spawned(SpawnId(Pos{x, y})) => format!("s-{},{}", x, y),
		}.serialize(serializer)
	}
}

#[derive(Debug)]
pub struct Creatures {
	players: HashMap<PlayerId, Player>,
	spawned_creatures: HashMap<SpawnId, SpawnedCreature>,
}

impl Creatures {

	pub fn new() -> Self {
		Self {
			players: HashMap::new(),
			spawned_creatures: HashMap::new(),
		}
	}

	pub fn add_player(&mut self, playerid: &PlayerId, saved: PlayerSave) -> Result<(), PlayerAlreadyExists> {
		if self.players.contains_key(playerid){
			return Err(PlayerAlreadyExists(*playerid));
		}
		let body = Creature::load_player(CreatureId::Player(*playerid), saved);
		self.players.insert(
			*playerid,
			Player::new(body)
		);
		Ok(())
	}

	pub fn remove_player(&mut self, playerid: &PlayerId) -> Result<(), PlayerNotFound> {
		self.players.remove(playerid)
			.ok_or(PlayerNotFound(*playerid))?;
		Ok(())
	}

	pub fn save_player(&self, playerid: &PlayerId) -> Option<PlayerSave> {
		Some(self.get_player(playerid)?.save())
	}

	pub fn list_players(&self) -> Vec<PlayerId> {
		self.players.keys().cloned().collect()
	}


	pub fn iter_players(&self) -> impl Iterator<Item=(&PlayerId, Ref<Creature>)> {
		self.players.iter()
			.map(|(player_id, player)| (player_id, player.body.borrow()))
	}

	pub fn get_player(&self, playerid: &PlayerId) -> Option<Ref<Creature>> {
		Some(self.players.get(playerid)?.body.borrow())
	}

	pub fn get_player_mut(&self, playerid: &PlayerId) -> Option<RefMut<Creature>> {
		Some(self.players.get(playerid)?.body.borrow_mut())
	}

	fn iter_cell(&self) -> impl Iterator<Item=&RefCell<Creature>> {
		self.players.values()
			.map(|player| &player.body)
			.chain(
				self.spawned_creatures.values()
				.filter(|s| !s.body.borrow().is_dead())
				.map(|s| &s.body)
			)
	}

	pub fn all_mut(&self) -> impl Iterator<Item=RefMut<Creature>> {
		self.iter_cell().map(|body| body.borrow_mut())
	}

	pub fn all(&self) -> impl Iterator<Item=Ref<Creature>> {
		self.iter_cell().map(|body| body.borrow())
	}

	fn creature_cell(&self, id: &CreatureId) -> Option<&RefCell<Creature>> {
		match id {
			CreatureId::Player(player_id) => self.players.get(player_id).map(|player| &player.body),
			CreatureId::Spawned(spawn_id) => self.spawned_creatures.get(spawn_id).map(|s| &s.body),
		}
	}

	pub fn get_creature(&self, id: &CreatureId) -> Option<Ref<Creature>>{
		self.creature_cell(id).map(RefCell::borrow)
	}

	pub fn get_creature_mut(&self, id: &CreatureId) -> Option<RefMut<Creature>>{
		self.creature_cell(id).map(RefCell::borrow_mut)
	}

	pub fn dead(&self) -> impl Iterator<Item=Ref<Creature>> {
		self.spawned_creatures.values()
			.filter(|s| s.body.borrow().is_dead())
			.map(|s| s.body.borrow())
	}

	pub fn spawn(&mut self, pos: Pos, npc: Npc) {
		let id = SpawnId(pos);
		if self.spawned_creatures.contains_key(&id) {
			return;
		}
		// println!("spawning {:?} npc at {:?}", npc, spawn_id);
		let body = Creature::spawn_npc(CreatureId::Spawned(id), pos, npc);
		let spawned_creature = SpawnedCreature { body: RefCell::new(body), last_load: Timestamp(0) };
		self.spawned_creatures.insert(id, spawned_creature);
	}

	pub fn despawn(&mut self, loaded_areas: &LoadedAreas, time: Timestamp) {
		for spawned in self.spawned_creatures.values_mut() {
			let body = spawned.body.borrow();
			if !body.is_dead() && loaded_areas.is_loaded(body.pos) {
				spawned.last_load = time;
			}
		}
		self.spawned_creatures.retain(|_spawn_id, spawned| spawned.last_load > time - Duration(500));
	}
}

#[derive(Debug, Clone)]
pub struct Player {
	pub plan: Option<Control>,
	pub body: RefCell<Creature>
}

#[derive(Debug, Clone)]
pub struct SpawnedCreature {
	pub body: RefCell<Creature>,
	pub last_load: Timestamp,
}

impl Player {
	pub fn new(body: Creature) -> Self {
		Self {
			plan: None,
			body: RefCell::new(body),
		}
	}
}

#[derive(Debug)]
pub struct PlayerNotFound(pub PlayerId);
#[derive(Debug)]
pub struct PlayerAlreadyExists(pub PlayerId);
#[derive(Debug)]
pub struct CreatureNotFound(pub CreatureId);
