
use std::collections::{HashMap};
use std::cell::{RefCell, Ref, RefMut};
use serde::{Serialize, Serializer};

use crate::{
	PlayerId,
	controls::{Control},
	pos::Pos,
	creature::{Creature, PlayerSave, SpawnId, Npc},
	loadedareas::LoadedAreas,
};

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
	spawned_creatures: HashMap<SpawnId, RefCell<Creature>>,
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
		let body = Creature::load_player(saved);
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

	pub fn npcs_mut(&self) -> impl Iterator<Item=(&SpawnId, RefMut<Creature>)> {
		self.spawned_creatures.iter()
			.map(|(spawn_id, creature)| (spawn_id, creature.borrow_mut()))
	}

	fn iter_cell(&self) -> impl Iterator<Item=(CreatureId, &RefCell<Creature>)> {
		self.players.iter()
			.map(|(player_id, player)| (CreatureId::Player(*player_id), &player.body))
			.chain(
				self.spawned_creatures.iter()
					.map(|(spawn_id, creature)| (CreatureId::Spawned(*spawn_id), creature))
			 )
	}

	pub fn all_mut(&self) -> impl Iterator<Item=(CreatureId, RefMut<Creature>)> {
		self.iter_cell().map(|(id, body)| (id, body.borrow_mut()))
	}

	pub fn all(&self) -> impl Iterator<Item=(CreatureId, Ref<Creature>)> {
		self.iter_cell().map(|(id, body)| (id, body.borrow()))
	}

	fn creature_cell(&self, id: &CreatureId) -> Option<&RefCell<Creature>> {
		match id {
			CreatureId::Player(player_id) => self.players.get(player_id).map(|player| &player.body),
			CreatureId::Spawned(spawn_id) => self.spawned_creatures.get(spawn_id),
		}
	}

	pub fn get_creature(&self, id: &CreatureId) -> Option<Ref<Creature>>{
		self.creature_cell(id).map(RefCell::borrow)
	}

	pub fn get_creature_mut(&self, id: &CreatureId) -> Option<RefMut<Creature>>{
		self.creature_cell(id).map(RefCell::borrow_mut)
	}

	pub fn spawn(&mut self, id: SpawnId, npc: Npc) {
		if self.spawned_creatures.contains_key(&id) {
			return;
		}
		// println!("spawning {:?} npc at {:?}", npc, spawn_id);
		let body = Creature::spawn_npc(id, npc);
		self.spawned_creatures.insert(id, RefCell::new(body));
	}

	pub fn despawn(&mut self, loaded_areas: &LoadedAreas) {
		self.spawned_creatures.retain(|_spawn_id, body| !body.borrow().is_dead && loaded_areas.is_loaded(body.borrow().pos));
	}
}

#[derive(Debug, Clone)]
pub struct Player {
	pub plan: Option<Control>,
	pub body: RefCell<Creature>
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
