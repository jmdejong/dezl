
use std::collections::{HashMap};
use std::cell::Ref;
use serde::{Serialize, Deserialize};

use crate::{
	PlayerId,
	config::MapDef,
	controls::{Plan, Control},
	pos::{Pos, Direction},
	worldmessages::{WorldMessage, ViewAreaMessage, ChangeMessage, SoundType::{BuildError}, PositionMessage, SoundType},
	timestamp::{Timestamp},
	creature::{Creature, PlayerSave, CreatureView, Faction},
	creatures::{Creatures, CreatureId, PlayerNotFound, PlayerAlreadyExists, CreatureNotFound},
	map::{Map, MapSave},
	basemap::BaseMapImpl,
	loadedareas::LoadedAreas,
	item::Item,
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
		let mut creature_map = CreatureMap::new(self.creatures.all());
		for (id, mut creature) in self.creatures.npcs_mut() {
			let home_pos = id.0;
			let rind = random::randomize_u32(random::randomize_pos(home_pos) + self.time.0 as u32);
			if random::percentage(rind + 543, 10) {
				let directions = if creature.pos != home_pos && random::percentage(rind + 471, 10) {
						creature.pos.directions_to(home_pos)
					} else {
						vec![Direction::North, Direction::South, Direction::East, Direction::West]
					};
				let direction = *random::pick(random::randomize_u32(rind + 385), &directions);
				let control = Plan::Move(direction);
				creature.control(Control::Plan(control));
			}
		}
		let creatures: Vec<CreatureId> = self.creatures.all().map(|(id, _)| id).collect();

		for id in creatures {
			let plan = {
				let Some(creature) = self.creatures.get_creature_mut(&id) else { continue };
				if !creature.can_move(self.time) {
					continue;
				}
				let Some(plan) = &creature.plan
					else { continue };
				plan.clone()
			};
			match plan {
				Plan::Move(direction) => {
					let _ = self.move_creature(&id, direction, &mut creature_map);
				}
				Plan::Movement(direction) => {
					let _ = self.move_creature(&id, direction, &mut creature_map);
				}
				Plan::Inspect(direction) => {
					let mut creature = self.creatures.get_creature_mut(&id).unwrap();
					let pos = creature.pos + direction.map(|dir| dir.to_position()).unwrap_or_else(Pos::zero);
					let tile = self.ground.cell(pos);
					let mut text = tile.inspect();
					for other in creature_map.get(&pos) {
						if other != id {
							let name = &self.creatures.get_creature(&other).unwrap().name;
							text = format!("{} | {}", text, name);
						}
					}
					creature.hear(SoundType::Explain, text);
				}
				Plan::Take(direction) => {
					let mut creature = self.creatures.get_creature_mut(&id).unwrap();
					let pos = creature.pos + direction.map(|dir| dir.to_position()).unwrap_or_else(Pos::zero);
					if let Some(item) = self.ground.take(pos) {
						creature.inventory.add(item);
					}
				}
				Plan::Use(index, direction) => {
					let item = self.creatures.get_creature(&id).unwrap().inventory.get_item(index);
					let _ = self.interact_creature(&id, direction, item);
				}
				Plan::Suicide => {
					let mut creature = self.creatures.get_creature_mut(&id).unwrap();
					creature.kill();
				}
				Plan::Stop => {}
			}
		}
	}

	fn move_creature(&mut self, id: &CreatureId, direction: Direction, creature_map: &mut CreatureMap) -> Result<(), CreatureNotFound> {
		let mut creature = self.creatures.get_creature_mut(id).unwrap();
		let newpos = creature.pos + direction;
		let tile = self.ground.cell(newpos);
		if !tile.blocking() && !creature_map.blocking(&newpos, &creature) {
			creature_map.move_creature(*id, &creature, &creature.pos, newpos);
			creature.move_to(newpos, self.time);
		}
		Ok(())
	}

	fn interact_creature(&mut self, id: &CreatureId, direction: Option<Direction>, item: Item) -> Result<(), CreatureNotFound> {
		let mut creature = self.creatures.get_creature_mut(id).unwrap();
		let pos = creature.pos + direction.map(|dir| dir.to_position()).unwrap_or_else(Pos::zero);
		let tile = self.ground.cell(pos);
		let Some(interaction) = tile.interact(item, self.time)
			else {
				return Ok(());
			};
		if interaction.claim {
			if let Some(player_id) = id.player() {
				if self.claims.contains_key(player_id) {
					creature.hear(BuildError, "Only one claim per player allowed".to_string());
					return Ok(())
				}
				if self.claims.values().any(|p| p.distance_to(pos) < 64) {
					creature.hear(BuildError, "Too close to existing claim".to_string());
					return Ok(())
				}
				if pos.distance_to(self.ground.player_spawn()) < 96 {
					creature.hear(BuildError, "Too close to spawn".to_string());
					return Ok(())
				}
				self.claims.insert(*player_id, pos);
			} else {
				creature.hear(
					BuildError,
					"Only players can claim land and you're not a player. If you read this something has probably gone wrong.".to_string()
				);
				return Ok(())
			}
		}
		if interaction.build {
			if let Some(claim_pos) = id.player().and_then(|player_id| self.claims.get(player_id)) {
				if pos.distance_to(*claim_pos) > 24 {
					creature.hear(
						BuildError,
						"Too far from land claim to build".to_string()
					);
					return Ok(())
				}
			} else {
				creature.hear(
					BuildError,
					"Need land claim to build".to_string()
				);
				return Ok(())
			}
		}
		if !creature.inventory.pay(interaction.cost) {
			return Ok(())
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
		Ok(())
	}

	fn update_loaded_areas(&mut self) {
		let player_positions: HashMap<PlayerId, Pos> = self.creatures.iter_players()
			.map(|(id, body)| (*id, body.pos))
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
			.map(|(id, creature)| (id, creature.view()))
			.collect();
		for (id, body) in self.creatures.iter_players() {
			let mut wm = WorldMessage::new(self.time);
			wm.viewarea = self.loaded_areas.loaded(id).map(|area| ViewAreaMessage{area});
			wm.section = self.loaded_areas.fresh(id).map(|area| self.ground.view(area));
			if changes.is_some() {
				wm.change = changes.clone();
			}
			wm.dynamics = Some(dynamics.clone());
			wm.pos = Some(PositionMessage{pos: body.pos, movement: body.current_movement(self.time)});
			wm.inventory = Some(body.inventory.view());
			wm.sounds = body.heard_sounds.clone();

			views.insert(*id, wm);
		}
		views
	}

	pub fn clear_step(&mut self) {
		for (_, mut creature) in self.creatures.all_mut() {
			creature.reset();
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
struct CreatureMap {
	map: HashMap<Pos, HashMap<CreatureId, CreatureTile>>
}

impl CreatureMap {

	pub fn new<'a>(creatures: impl Iterator<Item=(CreatureId, Ref<'a, Creature>)>) -> Self {
		let mut map = Self { map: HashMap::new() };
		for (id, creature) in creatures {
			map.insert(creature.pos, id, &creature);
		}
		map
	}

	pub fn get(&self, pos: &Pos) -> Vec<CreatureId> {
		self.map.get(pos).map(|c| c.keys().copied().collect()).unwrap_or_default()
	}

	pub fn blocking(&self, pos: &Pos, creature: &Creature) -> bool {
		self.map.get(pos)
			.map(|creatures| creatures.values().any(|c| c.faction.is_enemy(creature.faction)))
			.unwrap_or(false)
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
		self.map.entry(pos).or_default().insert(id, CreatureTile::new(creature));
	}
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CreatureTile {
	pub faction: Faction
}

impl CreatureTile {
	fn new(creature: &Creature) -> Self {
		Self {faction: creature.faction}
	}
}
