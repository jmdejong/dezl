
use std::collections::{HashMap};
use std::cell::{RefCell, Ref, RefMut};
use serde::{Serialize, Serializer};

use crate::{
	PlayerId,
	controls::{Control},
	pos::Pos,
	creature::{Creature, PlayerSave, SpawnId, Npc},
	loadedareas::LoadedAreas,
	random,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

	pub fn seed(&self) -> u32 {
		match self {
			Self::Player(PlayerId(name)) => random::randomize_str(name),
			Self::Spawned(SpawnId(origin)) => random::randomize_pos(*origin),
		}
	}
}

impl Serialize for CreatureId {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where S: Serializer {
		match self {
			Self::Player(PlayerId(name)) => format!("p-{}", name),
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
			return Err(PlayerAlreadyExists(playerid.clone()));
		}
		let body = Creature::load_player(saved);
		self.players.insert(
			playerid.clone(),
			Player::new(body)
		);
		Ok(())
	}

	pub fn remove_player(&mut self, playerid: &PlayerId) -> Result<(), PlayerNotFound> {
		self.players.remove(playerid)
			.ok_or_else(|| PlayerNotFound(playerid.clone()))?;
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

	// pub fn player_positions(&self) -> HashMap<PlayerId, Pos> {
	// 	self.players.iter()
	// 		.map(|(player_id, player)| (player_id.clone(), player.body.borrow().pos))
	// 		.collect()
	// }

	pub fn npcs(&self) -> Vec<CreatureId> {
		self.spawned_creatures.keys().map(|spawn_id| CreatureId::Spawned(*spawn_id)).collect()
	}

	pub fn control_creature(&mut self, id: &CreatureId, control: Control) -> Result<(), CreatureNotFound> {
		self.get_creature_mut(id).ok_or(CreatureNotFound(id.clone()))?.control(control);
		Ok(())
	}

	fn iter_cell(&self) -> impl Iterator<Item=(CreatureId, &RefCell<Creature>)> {
		self.players.iter()
			.map(|(player_id, player)| (CreatureId::Player(player_id.clone()), &player.body))
			.chain(
				self.spawned_creatures.iter()
					.map(|(spawn_id, creature)| (CreatureId::Spawned(*spawn_id), creature))
			 )
	}

	pub fn all_mut(&self) -> impl Iterator<Item=(CreatureId, RefMut<Creature>)> {
		self.iter_cell().map(|(id, body)| (id, body.borrow_mut()))
	}
	// 	self.players.iter()
	// 		.map(|(player_id, player)| (CreatureId::Player(player_id.clone()), player.body.borrow_mut()))
	// 		.chain(
	// 			self.spawned_creatures.iter_mut()
	// 				.map(|(spawn_id, creature)| (CreatureId::Spawned(*spawn_id), creature.borrow_mut()))
	// 		 )
	// }
 //
	pub fn all(&self) -> impl Iterator<Item=(CreatureId, Ref<Creature>)> {
		self.iter_cell().map(|(id, body)| (id, body.borrow()))
	}
	// 	self.players.iter()
	// 		.map(|(player_id, player)| (CreatureId::Player(player_id.clone()), player.body.borrow()))
	// 		.chain(
	// 			self.spawned_creatures.iter()
	// 				.map(|(spawn_id, creature)| (CreatureId::Spawned(*spawn_id), creature.borrow()))
	// 		 )
	// }

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
		self.spawned_creatures.retain(|_spawn_id, body| loaded_areas.is_loaded(body.borrow().pos));
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
