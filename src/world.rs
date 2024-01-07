
use std::collections::{HashMap};
use serde::{Serialize, Deserialize};

use crate::{
	PlayerId,
	config::MapDef,
	controls::{Control},
	pos::{Pos, Direction},
	worldmessages::{WorldMessage, ViewAreaMessage, ChangeMessage, SoundType::{BuildError}, PositionMessage},
	timestamp::{Timestamp},
	creature::{PlayerSave, CreatureView},
	creatures::{Creatures, CreatureId, PlayerNotFound, PlayerAlreadyExists, CreatureNotFound},
	map::{Map, MapSave},
	basemap::BaseMapImpl,
	loadedareas::LoadedAreas,
	random,
};

pub struct World {
	pub name: String,
	pub time: Timestamp,
	ground: Map,
	creatures: Creatures,
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
			time,
			claims: HashMap::new(),
			creatures: Creatures::new(),
			mapdef,
			loaded_areas: LoadedAreas::new(),
		}
	}
	
	pub fn default_player(&mut self) -> PlayerSave {
		PlayerSave::new(self.ground.player_spawn())
	}
	
	pub fn add_player(&mut self, playerid: &PlayerId, saved: PlayerSave) -> Result<(), PlayerAlreadyExists> {
		self.creatures.add_player(playerid, saved)
	}
	
	pub fn remove_player(&mut self, playerid: &PlayerId) -> Result<(), PlayerNotFound> {
		self.creatures.remove_player(playerid)
	}
	
	pub fn save_player(&self, playerid: &PlayerId) -> Option<PlayerSave> {
		self.creatures.save_player(playerid)
	}
	
	pub fn control_player(&mut self, playerid: &PlayerId, control: Control) -> Result<(), CreatureNotFound> {
		self.creatures.control_player(playerid, control)
	}
	
	pub fn list_players(&self) -> Vec<PlayerId> {
		self.creatures.list_players()
	}
	
	
	fn update_creatures(&mut self) -> Option<()> {
		let mut creature_map: HashMap<Pos, CreatureId> = self.creatures.all()
			.map(|(creatureid, creature)| (creature.pos, creatureid.clone()))
			.collect();
		for npc in self.creatures.npcs() {
			let rind = random::randomize_u32(npc.seed() + self.time.0 as u32);
			let control = if random::randomize_u32(rind + 543) % 10 == 0 {
					Some(Control::Move(*random::pick(random::randomize_u32(rind + 385), &[Direction::North, Direction::South, Direction::East, Direction::West])))
				} else {
					None
				};
			let _ = self.creatures.control_creature(&npc, control);
		}
		let creatures: Vec<CreatureId> = self.creatures.all().map(|(id, _)| id).collect();

		for id in creatures {
			let Some(creature) = self.creatures.get_creature_mut(&id)
				else { continue };
			if !creature.can_move(self.time) {
				continue;
			}
			creature.heard_sounds = Vec::new();
			let Some(plan) = &creature.plan
				else { continue };
			match plan {
				Control::Move(direction) => {
					let newpos = creature.pos + *direction;
					let tile = self.ground.cell(newpos);
					if !tile.blocking() && !creature_map.contains_key(&newpos) {
						if creature_map.get(&creature.pos) == Some(&id){
							creature_map.remove(&creature.pos);
						}
						creature_map.insert(newpos, id.clone());
						creature.move_to(newpos, self.time);
					}
				}
				Control::Movement(direction) => {
					let newpos = creature.pos + *direction;
					let tile = self.ground.cell(newpos);
					if !tile.blocking() && !creature_map.contains_key(&newpos) {
						if creature_map.get(&creature.pos) == Some(&id){
							creature_map.remove(&creature.pos);
						}
						creature_map.insert(newpos, id.clone());
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
						if let Some(player_id) = id.player() {
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
							self.claims.insert(player_id.clone(), pos);
						} else {
							creature.heard_sounds.push((
								BuildError,
								"Only players can claim land and you're not a player. If you read this something has probably gone wrong.".to_string()
							));
							continue;
						}
					}
					if interaction.build {
						if let Some(claim_pos) = id.player().and_then(|player_id| self.claims.get(player_id)) {
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
		self.creatures.reset_plans();
		Some(())
	}

	fn update_loaded_areas(&mut self) {
		let player_positions: HashMap<PlayerId, Pos> = self.creatures.players.iter()
			.map(|(player_id, player)| (player_id.clone(), player.body.pos))
			.collect();
		self.loaded_areas.update(&player_positions);
		for fresh_area in self.loaded_areas.all_fresh() {
			self.ground.load_area(fresh_area);
		}
		self.ground.tick(self.time, self.loaded_areas.all_loaded());
	}

	fn spawn_creatures(&mut self) {
		for (spawn_id, npc) in self.ground.spawns() {
			self.creatures.spawn(spawn_id, npc);
		}
		self.creatures.despawn(&self.loaded_areas);
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
		let dynamics: HashMap<CreatureId, CreatureView> = self.creatures.all()
			.map(|(id, creature)| (id.clone(), creature.view()))
			.collect();
		for (id, player) in self.creatures.players.iter() {
			let mut wm = WorldMessage::new(self.time);
			let body = &player.body;
			wm.viewarea = self.loaded_areas.loaded(id).map(|area| ViewAreaMessage{area});
			wm.section = self.loaded_areas.fresh(id).map(|area| self.ground.view(area));
			if changes.is_some() {
				wm.change = changes.clone();
			}
			wm.dynamics = Some(dynamics.clone());
			wm.pos = Some(PositionMessage{pos: body.pos, movement: body.current_movement(self.time)});
			wm.inventory = Some(body.inventory.view());
			wm.sounds = body.heard_sounds.clone();

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
			creatures: Creatures::new(),
			time: save.time,
			claims: save.claims,
			mapdef: save.mapdef,
			loaded_areas: LoadedAreas::new(),
		}
	}
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldSave {
	name: String,
	time: Timestamp,
	ground: MapSave,
	claims: HashMap<PlayerId, Pos>,
	pub mapdef: MapDef,
}

