
use std::collections::{HashMap};
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
	pub players: HashMap<PlayerId, Player>,
	spawned_creatures: HashMap<SpawnId, Creature>,
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
		Some(self.players.get(playerid)?.body.save())
	}

	pub fn list_players(&self) -> Vec<PlayerId> {
		self.players.keys().cloned().collect()
	}

	pub fn get_player_mut(&mut self, playerid: &PlayerId) -> Option<&mut Creature> {
		self.players.get_mut(playerid).map(|player| &mut player.body)
	}

	pub fn npcs(&self) -> Vec<CreatureId> {
		self.spawned_creatures.keys().map(|spawn_id| CreatureId::Spawned(*spawn_id)).collect()
	}

	pub fn control_creature(&mut self, id: &CreatureId, control: Control) -> Result<(), CreatureNotFound> {
		self.get_creature_mut(id).ok_or(CreatureNotFound(id.clone()))?.control(control);
		Ok(())
	}

	pub fn all_mut(&mut self) -> impl Iterator<Item=(CreatureId, &mut Creature)> {
		self.players.iter_mut()
			.map(|(player_id, player)| (CreatureId::Player(player_id.clone()), &mut player.body))
			.chain(
				self.spawned_creatures.iter_mut()
					.map(|(spawn_id, creature)| (CreatureId::Spawned(*spawn_id), creature))
			 )
	}

	pub fn all(&self) -> impl Iterator<Item=(CreatureId, &Creature)> {
		self.players.iter()
			.map(|(player_id, player)| (CreatureId::Player(player_id.clone()), &player.body))
			.chain(
				self.spawned_creatures.iter()
					.map(|(spawn_id, creature)| (CreatureId::Spawned(*spawn_id), creature))
			 )
	}

	pub fn get_creature(&mut self, id: &CreatureId) -> Option<&Creature>{
		match id {
			CreatureId::Player(player_id) => self.players.get(player_id).map(|player| &player.body),
			CreatureId::Spawned(spawn_id) => self.spawned_creatures.get(spawn_id),
		}
	}

	pub fn get_creature_mut(&mut self, id: &CreatureId) -> Option<&mut Creature>{
		match id {
			CreatureId::Player(player_id) => self.players.get_mut(player_id).map(|player| &mut player.body),
			CreatureId::Spawned(spawn_id) => self.spawned_creatures.get_mut(spawn_id),
		}
	}

	pub fn spawn(&mut self, id: SpawnId, npc: Npc) {
		if self.spawned_creatures.contains_key(&id) {
			return;
		}
		// println!("spawning {:?} npc at {:?}", npc, spawn_id);
		let body = Creature::spawn_npc(id, npc);
		self.spawned_creatures.insert(id, body);
	}

	pub fn despawn(&mut self, loaded_areas: &LoadedAreas) {
		self.spawned_creatures.retain(|_spawn_id, body| loaded_areas.is_loaded(body.pos));
	}
}

#[derive(Debug, Clone)]
pub struct Player {
	pub plan: Option<Control>,
	pub body: Creature
}

impl Player {
	pub fn new(body: Creature) -> Self {
		Self {
			plan: None,
			body,
		}
	}
}

#[derive(Debug)]
pub struct PlayerNotFound(pub PlayerId);
#[derive(Debug)]
pub struct PlayerAlreadyExists(pub PlayerId);
#[derive(Debug)]
pub struct CreatureNotFound(pub CreatureId);
