
use std::collections::{HashMap};
use serde::{Serialize, Deserialize};

use crate::{
	PlayerId,
	config::MapDef,
	controls::{Control},
	pos::{Pos, Direction},
	util::Holder,
	worldmessages::{WorldMessage, ViewAreaMessage, ChangeMessage, SoundType::{BuildError}, PositionMessage},
	timestamp::{Timestamp},
	creature::{Creature, Mind, CreatureId, PlayerSave, CreatureView, SpawnId},
	player::Player,
	map::{Map, MapSave},
	basemap::BaseMapImpl,
	loadedareas::LoadedAreas,
	random,
};

pub struct World {
	pub name: String,
	pub time: Timestamp,
	ground: Map,
	players: HashMap<PlayerId, Player>,
	creatures: Holder<CreatureId, Creature>,
	spawned_creatures: HashMap<SpawnId, CreatureId>,
	claims: HashMap<PlayerId, Pos>,
	mapdef: MapDef,
	loaded_areas: LoadedAreas,
}

impl World {
	
	pub fn new(name: String, basemap: BaseMapImpl, mapdef: MapDef) -> Self {
		let time = Timestamp(0);
		Self {
			name,
			ground: Map::new(basemap, time),
			players: HashMap::new(),
			creatures: Holder::new(),
			time,
			claims: HashMap::new(),
			spawned_creatures: HashMap::new(),
			mapdef,
			loaded_areas: LoadedAreas::new(),
		}
	}
	
	pub fn default_player(&mut self) -> PlayerSave {
		PlayerSave::new(self.ground.player_spawn())
	}
	
	pub fn add_player(&mut self, playerid: &PlayerId, saved: PlayerSave) -> Result<(), PlayerError> {
		if self.players.contains_key(playerid){
			return Err(PlayerError::AlreadyExists(playerid.clone()));
		}
		let body = self.creatures.insert(Creature::load_player(playerid.clone(), saved));
		self.players.insert(
			playerid.clone(),
			Player::new(body)
		);
		Ok(())
	}
	
	pub fn remove_player(&mut self, playerid: &PlayerId) -> Result<(), PlayerError> {
		let player = self.players.remove(playerid).ok_or_else(|| PlayerError::NotFound(playerid.clone()))?;
		self.creatures.remove(&player.body);
		Ok(())
	}
	
	pub fn save_player(&self, playerid: &PlayerId) -> Result<PlayerSave, PlayerError> {
		let player = self.players.get(playerid).ok_or_else(|| PlayerError::NotFound(playerid.clone()))?;
		let body = self.creatures.get(&player.body).ok_or_else(|| PlayerError::BodyNotFound(playerid.clone()))?;
		Ok(body.save())
	}
	
	pub fn control_player(&mut self, playerid: &PlayerId, control: Control) -> Result<(), PlayerError> {
		let player = self.players.get_mut(playerid).ok_or_else(|| PlayerError::NotFound(playerid.clone()))?;
		player.plan = Some(control);
		Ok(())
	}
	
	pub fn has_player(&mut self, playerid: &PlayerId) -> bool {
		self.players.contains_key(playerid)
	}
	
	pub fn list_players(&self) -> Vec<PlayerId> {
		self.players.keys().cloned().collect()
	}
	
	fn creature_plan(&self, creature: &Creature) -> Option<Control> {
		match &creature.mind {
			Mind::Player(playerid) => {
				if let Some(player) = self.players.get(playerid) {
					player.plan.clone()
				} else {Some(Control::Suicide)}
			}
			Mind::Spawned(SpawnId(origin)) => {
				let rind = random::randomize_u32(random::randomize_pos(*origin) + self.time.0 as u32);
				if random::randomize_u32(rind + 543) % 10 == 0 {
					Some(Control::Move(*random::pick(random::randomize_u32(rind + 385), &[Direction::North, Direction::South, Direction::East, Direction::West])))
				} else {
					None
				}
			}
		}
	}
	
	fn update_creatures(&mut self) -> Option<()> {
		let mut creature_map: HashMap<Pos, CreatureId> = self.creatures.iter()
			.map(|(creatureid, creature)| (creature.pos, *creatureid))
			.collect();
		let plans: HashMap<CreatureId, Control> = self.creatures.iter()
			.filter(|(_k, c)| c.can_move(self.time))
			.filter_map(|(k, c)|
				Some((*k, self.creature_plan(c)?))
			).collect();
		for (id, creature) in self.creatures.iter_mut() {
			creature.heard_sounds = Vec::new();
			let Some(plan) = plans.get(id) 
				else {
					continue 
				};
			match plan {
				Control::Move(direction) => {
					let newpos = creature.pos + *direction;
					let tile = self.ground.cell(newpos);
					if !tile.blocking() && !creature_map.contains_key(&newpos) {
						if creature_map.get(&creature.pos) == Some(id){
							creature_map.remove(&creature.pos);
						}
						creature_map.insert(newpos, *id);
						creature.move_to(newpos, self.time);
					}
				}
				Control::Movement(direction) => {
					let newpos = creature.pos + *direction;
					let tile = self.ground.cell(newpos);
					if !tile.blocking() && !creature_map.contains_key(&newpos) {
						if creature_map.get(&creature.pos) == Some(id){
							creature_map.remove(&creature.pos);
						}
						creature_map.insert(newpos, *id);
						creature.move_to(newpos, self.time);
					}
				}
				Control::Suicide => {
					creature.kill();
				}
				Control::Select(selector) => {
					creature.inventory.select(*selector);
				}
				Control::MoveSelected(selector) => {
					creature.inventory.move_selected(*selector);
				}
				Control::Interact(direction) => {
					let pos = creature.pos + direction.map(|dir| dir.to_position()).unwrap_or_else(Pos::zero);
					let tile = self.ground.cell(pos);
					let item = creature.inventory.selected();
					let Some(interaction) = tile.interact(item, self.time) 
						else {
							continue
						};
					if interaction.claim {
						if let Some(player_id) = creature.player() {
							if self.claims.contains_key(&player_id) {
								creature.heard_sounds.push((BuildError, "Only one claim per player allowed".to_string()));
								continue;
							}
							if self.claims.values().any(|p| p.distance_to(pos) < 64) {
								creature.heard_sounds.push((BuildError, "Too close to existing claim".to_string()));
								continue;
							}
							if pos.distance_to(self.ground.player_spawn()) < 96 {
								creature.heard_sounds.push((BuildError, "Too close to spawn".to_string()));
								continue;
							}
							self.claims.insert(player_id, pos);
						} else {
							creature.heard_sounds.push((
								BuildError,
								"Only players can claim land and you're not a player. If you read this something has probably gone wrong.".to_string()
							));
							continue;
						}
					}
					if interaction.build {
						if let Some(claim_pos) = creature.player().as_ref().and_then(|player_id| self.claims.get(player_id)) {
							if pos.distance_to(*claim_pos) > 24 {
								creature.heard_sounds.push((
									BuildError,
									"Too far from land claim to build".to_string()
								));
								continue;
							}
						} else {
							creature.heard_sounds.push((
								BuildError,
								"Need land claim to build".to_string()
							));
							continue;
						}
					}
					if !creature.inventory.pay(interaction.cost) {
						continue;
					}
					for item in interaction.items {
						creature.inventory.add(item);
					}
					if let Some(remains) = interaction.remains {
						self.ground.set_structure(pos, remains);
					}
					if let Some(remains_ground) = interaction.remains_ground {
						self.ground.set_ground(pos, remains_ground);
					}
					if let Some(message) = interaction.message {
						creature.heard_sounds.push(message);
					}
				}
				Control::Stop => {}
			}
		}
		for player in self.players.values_mut() {
			if let Some(Control::Movement(_)) = player.plan {

			} else {
				player.plan = None;
			}
		}
		Some(())
	}

	fn update_loaded_areas(&mut self) {
		let player_positions: Vec<(PlayerId, Pos)> = self.players.iter()
			.filter_map(|(player_id, player)| Some((player_id.clone(), self.creatures.get(&player.body)?.pos)))
			.collect();
		self.loaded_areas.update(&player_positions);
		for fresh_area in self.loaded_areas.all_fresh() {
			self.ground.load_area(fresh_area);
		}
		self.ground.tick(self.time, self.loaded_areas.all_loaded());
	}

	fn spawn_creatures(&mut self) {
		for (spawn_id, npc) in self.ground.spawns() {
			if self.spawned_creatures.contains_key(&spawn_id) {
				continue;
			}
			// println!("spawning {:?} npc at {:?}", npc, spawn_id);
			let body = self.creatures.insert(Creature::spawn_npc(spawn_id, npc));
			self.spawned_creatures.insert(spawn_id, body);
		}
		self.spawned_creatures.retain(|_spawn_id, body_id| {
			let pos = self.creatures.get(body_id).unwrap().pos;
			if !self.loaded_areas.is_loaded(pos) {
				// println!("despawning npc {:?} from {:?}", _spawn_id, pos);
				self.creatures.remove(body_id);
				false
			} else {
				true
			}
		})
	}
	
	pub fn update(&mut self) {
		self.ground.flush();
		self.time.increment();
		self.update_creatures();
		self.update_loaded_areas();
		self.spawn_creatures();
	}
	
	
	fn draw_changes(&self) -> Option<ChangeMessage> {
		Some(
			self.ground.modified().into_iter()
				.map(|(pos, tile)| (pos, tile.sprites()))
				.collect()
		)
	}
	
	pub fn view(&self) -> HashMap<PlayerId, WorldMessage> {
		let changes = self.draw_changes();
		let mut views: HashMap<PlayerId, WorldMessage> = HashMap::new();
		let dynamics: HashMap<CreatureId, CreatureView> = self.creatures.iter()
			.map(|(id, creature)| (*id, creature.view()))
			.collect();
		for (id, player) in self.players.iter() {
			let mut wm = WorldMessage::new(self.time);
			if let Some(body) = self.creatures.get(&player.body) {
				wm.viewarea = self.loaded_areas.loaded(id).map(|area| ViewAreaMessage{area});
				wm.section = self.loaded_areas.fresh(id).map(|area| self.ground.view(area));
				if changes.is_some() {
					wm.change = changes.clone();
				}
				wm.dynamics = Some(dynamics.clone());
				wm.pos = Some(PositionMessage{pos: body.pos, movement: body.current_movement(self.time)});
				wm.inventory = Some(body.inventory.view());
				wm.sounds = body.heard_sounds.clone();

			}
			views.insert(id.clone(), wm);
		}
		views
	}
	
	pub fn save(&self) -> WorldSave {
		WorldSave {
			name: self.name.clone(),
			time: self.time,
			ground: self.ground.save(),
			claims: self.claims.clone(),
			mapdef: self.mapdef.clone(),
		}
	}
	
	pub fn load(save: WorldSave, basemap: BaseMapImpl) -> World {
		World {
			name: save.name,
			ground: Map::load(save.ground, save.time, basemap),
			players: HashMap::new(),
			creatures: Holder::new(),
			spawned_creatures: HashMap::new(),
			time: save.time,
			claims: save.claims,
			mapdef: save.mapdef,
			loaded_areas: LoadedAreas::new(),
		}
	}
}


#[derive(Debug)]
pub enum PlayerError {
	NotFound(PlayerId),
	BodyNotFound(PlayerId),
	AlreadyExists(PlayerId)
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldSave {
	name: String,
	time: Timestamp,
	ground: MapSave,
	claims: HashMap<PlayerId, Pos>,
	pub mapdef: MapDef,
}

