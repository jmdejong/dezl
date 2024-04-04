
use std::collections::{HashMap};
use std::cell::Ref;
use serde::{Serialize, Deserialize};

use crate::{
	PlayerId,
	config::MapDef,
	controls::{Plan, Control},
	pos::{Pos, Direction},
	worldmessages::{WorldMessage, ViewAreaMessage, ChangeMessage, SoundType::{BuildError}, SoundType},
	timestamp::{Timestamp},
	creature::{Creature, PlayerSave, CreatureView, Faction},
	creatures::{Creatures, CreatureId, PlayerNotFound, PlayerAlreadyExists, CreatureNotFound},
	map::{Map, MapSave},
	basemap::BaseMapImpl,
	loadedareas::LoadedAreas,
	item::Item,
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
	
	pub fn default_player(&mut self, name: String) -> PlayerSave {
		PlayerSave::new(name, self.ground.player_spawn())
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
		self.creatures.get_player_mut(playerid).ok_or(CreatureNotFound(CreatureId::Player(*playerid)))?.control(control);
		Ok(())
	}
	
	pub fn list_players(&self) -> Vec<PlayerId> {
		self.creatures.list_players()
	}
	

	fn update_creatures(&mut self) {

		for mut creature in self.creatures.all_mut() {
			creature.autoheal_tick(self.time);
		}

		let mut creature_map = CreatureMap::new(self.creatures.all());
		for mut creature in self.creatures.all_mut() {
			creature.plan(&creature_map, self.time);
		}
		let creatures: Vec<CreatureId> = self.creatures.all().map(|creature| creature.id).collect();

		for id in creatures {
			let plan = {
				let Some(creature) = self.creatures.get_creature_mut(&id) else { continue };
				if !creature.can_act(self.time) {
					continue;
				}
				let Some(plan) = &creature.plan
					else { continue };
				plan.clone()
			};
			match plan {
				Plan::Move(direction) | Plan::Movement(direction)=> {
					self.move_creature(&id, direction, &mut creature_map);
				}
				Plan::Inspect(direction) => {
					let mut creature = self.creatures.get_creature_mut(&id).unwrap();
					let pos = creature.pos + direction;
					let tile = self.ground.cell(pos);
					let mut text = tile.inspect();
					for other in creature_map.get(&pos) {
						if other.id != id {
							let name = &self.creatures.get_creature(&other.id).unwrap().name;
							text = format!("{} | {}", text, name);
						}
					}
					creature.hear(SoundType::Explain, text);
				}
				Plan::Take(direction) => {
					let mut creature = self.creatures.get_creature_mut(&id).unwrap();
					let pos = creature.pos + direction;
					if let Some(item) = self.ground.take(pos) {
						creature.inventory.add(item);
					}
				}
				Plan::Use(index, direction) => {
					let item = self.creatures.get_creature(&id).unwrap().inventory.get_item(index);
					self.interact_creature(&id, direction, item);
				}
				Plan::Fight(direction) => {
					let mut creature = self.creatures.get_creature_mut(&id).unwrap();
					let pos = creature.pos + direction;
					if let Some(opponent) = creature_map.get(&pos).iter().find(|o| creature.faction.is_enemy(o.faction)) {
						creature.attack(self.creatures.get_creature_mut(&opponent.id).unwrap(), self.time);
					}
				}
				Plan::Stop => {}
			}
		}
	}

	fn move_creature(&mut self, id: &CreatureId, direction: Direction, creature_map: &mut CreatureMap) {
		let mut creature = self.creatures.get_creature_mut(id).unwrap();
		let newpos = creature.pos + direction;
		let tile = self.ground.cell(newpos);
		if !tile.blocking() && !creature_map.blocking(&newpos, &creature) {
			creature_map.move_creature(*id, &creature, &creature.pos, newpos);
			creature.move_to(newpos, self.time);
		}
	}

	fn interact_creature(&mut self, id: &CreatureId, direction: Option<Direction>, item: Item) {
		let mut creature = self.creatures.get_creature_mut(id).unwrap();
		let pos = creature.pos + direction;
		let tile = self.ground.cell(pos);
		let Some(interaction) = tile.interact(item, self.time)
			else {
				return;
			};
		if interaction.claim {
			if let Some(player_id) = id.player() {
				if self.claims.contains_key(player_id) {
					creature.hear(BuildError, "Only one claim per player allowed".to_string());
					return;
				}
				if self.claims.values().any(|p| p.distance_to(pos) < 64) {
					creature.hear(BuildError, "Too close to existing claim".to_string());
					return;
				}
				if pos.distance_to(self.ground.player_spawn()) < 96 {
					creature.hear(BuildError, "Too close to spawn".to_string());
					return;
				}
				self.claims.insert(*player_id, pos);
			} else {
				creature.hear(
					BuildError,
					"Only players can claim land and you're not a player. If you read this something has probably gone wrong.".to_string()
				);
				return;
			}
		}
		if interaction.build {
			if let Some(claim_pos) = id.player().and_then(|player_id| self.claims.get(player_id)) {
				if pos.distance_to(*claim_pos) > 24 {
					creature.hear(
						BuildError,
						"Too far from land claim to build".to_string()
					);
					return;
				}
			} else {
				creature.hear(
					BuildError,
					"Need land claim to build".to_string()
				);
				return;
			}
		}
		if !creature.inventory.pay(interaction.cost) {
			return;
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
		if let Some((message_type, message_text)) = interaction.message {
			creature.hear(message_type, message_text);
		}
	}

	fn update_loaded_areas(&mut self) {
		let player_positions: HashMap<PlayerId, Pos> = self.creatures.iter_players()
			.map(|(id, body)| (*id, body.pos))
			.collect();
		self.loaded_areas.update(&player_positions);
		for fresh_area in self.loaded_areas.all_fresh() {
			self.ground.load_area(fresh_area);
		}
		self.ground.tick(self.time, &self.loaded_areas.all_loaded());
	}

	fn spawn_creatures(&mut self) {
		for (pos, npc) in self.ground.spawns() {
			self.creatures.spawn(pos, npc);
		}
		self.creatures.despawn(&self.loaded_areas, self.time);
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
			.map(|creature| (creature.id, creature.view()))
			.collect();
		for (id, body) in self.creatures.iter_players() {
			let mut wm = WorldMessage::new(self.time);
			wm.viewarea = self.loaded_areas.loaded(id).map(|area| ViewAreaMessage{area});
			wm.section = self.loaded_areas.fresh(id).map(|area| self.ground.view(area));
			if changes.is_some() {
				wm.change = changes.clone();
			}
			wm.dynamics = Some(dynamics.clone());
			wm.me = Some(body.view());
			wm.inventory = Some(body.inventory.view());
			wm.sounds = body.heard_sounds.clone();

			views.insert(*id, wm);
		}
		views
	}

	pub fn clear_step(&mut self) {
		for mut creature in self.creatures.all_mut() {
			creature.reset(self.time);
		}
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

#[derive(Debug)]
pub struct CreatureMap {
	map: HashMap<Pos, HashMap<CreatureId, CreatureTile>>,
	all: HashMap<CreatureId, CreatureTile>
}

impl CreatureMap {

	pub fn new<'a>(creatures: impl Iterator<Item=Ref<'a, Creature>>) -> Self {
		let mut map = Self { map: HashMap::new(), all: HashMap::new() };
		for creature in creatures {
			map.insert(creature.pos, creature.id, &creature);
		}
		map
	}

	pub fn get(&self, pos: &Pos) -> Vec<CreatureTile> {
		self.map.get(pos).map(|c| c.values().copied().collect()).unwrap_or_default()
	}

	pub fn get_creature(&self, id: &CreatureId) -> Option<&CreatureTile> {
		self.all.get(id)
	}

	pub fn nearby(&self, pos: Pos, distance: i32) -> impl Iterator<Item=&CreatureTile> {
		self.map.iter()
			.filter(move |(p, _)| p.distance_to(pos) <= distance)
			.flat_map(|(_, creatures)| creatures.values())
	}

	pub fn blocking(&self, pos: &Pos, creature: &Creature) -> bool {
		self.map.get(pos)
			.is_some_and(|creatures| creatures.values().any(|c| c.faction.is_enemy(creature.faction)))
	}


	pub fn move_creature(&mut self, id: CreatureId, creature: &Creature, from: &Pos, to: Pos) {
		if let Some(creatures) = self.map.get_mut(from) {
			creatures.remove(&id);
			if creatures.is_empty() {
				self.map.remove(from);
			}
		}
		self.insert(to, id, creature);
	}

	fn insert(&mut self, pos: Pos, id: CreatureId, creature: &Creature) {
		let mut tile = CreatureTile::new(id, creature);
		tile.pos = pos;
		self.map.entry(pos).or_default().insert(id, tile);
		self.all.insert(id, tile);
	}
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreatureTile {
	pub id: CreatureId,
	pub faction: Faction,
	pub pos: Pos,
}

impl CreatureTile {
	fn new(id: CreatureId, creature: &Creature) -> Self {
		Self {
			id,
			pos: creature.pos,
			faction: creature.faction,
		}
	}
}
