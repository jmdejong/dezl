
use std::collections::{HashMap, HashSet};
use serde::Serialize;
use crate::{
	pos::{Pos, Area, Direction},
	tile::{Tile, Structure, Ground},
	basemap::{BaseMap, BaseMapImpl},
	timestamp::{Timestamp, Duration},
	sprite::Sprite,
	creature::{Npc},
	creatures::SpawnId,
	item::Item,
	randomtick
};

pub struct Map {
	basemap: BaseMapImpl,
	changes: HashMap<Pos, (Tile, Timestamp)>,
	time: Timestamp,
	modifications: HashSet<Pos>,
	spawns: Vec<(SpawnId, Npc)>
}

impl Map {
	
	pub fn new(basemap: BaseMapImpl, time: Timestamp) -> Self {
		Self {
			basemap,
			changes: HashMap::new(),
			time,
			modifications: HashSet::new(),
			spawns: Vec::new(),
		}
	}
	
	fn base_cell(&self, pos: Pos) -> Tile {
		self.basemap.cell(pos, self.time)
	}
	
	pub fn cell(&self, pos: Pos) -> Tile {
		self.changes.get(&pos).map_or_else(|| self.base_cell(pos), |change| change.0)
	}

	fn region(&self, area: Area) -> impl Iterator<Item = Tile> + '_ {
		self.basemap.region(area, self.time).into_iter().map(|(pos, base_cell)| {
			self.changes.get(&pos).map_or(base_cell, |change| change.0)
		})
	}

	pub fn load_area(&mut self, area: Area) {
		for pos in Area::centered(area.min() + area.size() / 2, Pos::new(128, 128)).iter() {
			self.tick_one(pos);
		}
	}

	pub fn set(&mut self, pos: Pos, tile: Tile) {
		if tile == self.base_cell(pos) {
			self.changes.remove(&pos);
		} else {
			self.changes.insert(pos, (tile, self.time));
		}
		self.modifications.insert(pos);
	}
	
	pub fn set_structure(&mut self, pos: Pos, structure: Structure) {
		let new_tile = Tile::structure(self.cell(pos).ground, structure) ;
		self.set(pos, new_tile);
	}
	
	pub fn set_ground(&mut self, pos: Pos, ground: Ground) {
		let new_tile = Tile::structure(ground, self.cell(pos).structure);
		self.set(pos, new_tile);
	}

	pub fn take(&mut self, pos: Pos) -> Option<Item> {
		let (new_tile, item) = self.cell(pos).take()?;
		self.set(pos, new_tile);
		Some(item)
	}
	
	pub fn player_spawn(&self) -> Pos {
		self.basemap.player_spawn()
	}
	
	pub fn tick(&mut self, time: Timestamp, areas: &[Area]) {
		self.time = time;
		let chunk_size = randomtick::CHUNK_SIZE;
		let tick_pos = randomtick::tick_position(time);
		let tick_positions = areas.iter()
			.flat_map(|area| {
				let chunk_min = area.min() / chunk_size;
				let chunk_max = (area.max() / chunk_size) + Pos::new(1, 1);
				let chunk_area = Area::new(chunk_min, chunk_max - chunk_min);
				chunk_area.iter()
					.map(|chunk_pos| chunk_pos * chunk_size + tick_pos)
					.filter(|pos| area.contains(*pos))
			})
			.collect::<HashSet<Pos>>();
		for pos in tick_positions {
			self.modifications.insert(pos);
			self.tick_one(pos);
		}
	}
	
	fn tick_one(&mut self, pos: Pos) {
		let tick_interval = randomtick::CHUNK_AREA;
		if let Some((mut built, mut built_time)) = self.changes.get(&pos) {
			while let Some((nticks, stage, surround)) = built.grow() {
				let update_time = built_time + Duration(nticks * tick_interval);
				if update_time <= self.time {
					built.structure = stage;
					built_time = update_time;
					self.changes.insert(pos, (built, built_time));
					if let Some(shoot) = surround {
						for d in Direction::DIRECTIONS {
							let npos = pos + d;
							let mut ntile = self.cell(npos);
							if let Some(product) = shoot.joined(ntile.structure) {
								ntile.structure = product;
								self.changes.insert(npos, (ntile, built_time));
								self.modifications.insert(npos);
							} else if ntile.structure.is_open() {
								ntile.structure = shoot;
								self.changes.insert(npos, (ntile, built_time));
								self.modifications.insert(npos);
							}
						}
					}
				} else {
					break
				}
			}
			if built.structure.is_open() {
				let base_cell = self.base_cell(pos);
				if  (built.ground.restoring() || built.ground == base_cell.ground) && base_cell.structure.is_open() {
					self.changes.remove(&pos);
				}
			}
		}
		if let Some(npc) = self.cell(pos).spawn() {
			self.spawns.push((SpawnId(pos), npc));
		}
	}
	
	pub fn flush(&mut self) {
		self.modifications.clear();
		self.spawns.clear();
	}
	
	pub fn modified(&self) -> HashMap<Pos, Tile> {
		self.modifications.clone().into_iter().map(|pos| (pos, self.cell(pos))).collect()
	}

	pub fn spawns(&self) -> Vec<(SpawnId, Npc)> {
		self.spawns.clone()
	}
	
	pub fn save(&self) -> MapSave {
		self.changes.clone().into_iter().collect()
	}
	
	pub fn load(changes: MapSave, time: Timestamp, basemap: BaseMapImpl) -> Self {
		Self {
			basemap,
			changes: changes.into_iter().collect(),
			time,
			modifications: HashSet::new(),
			spawns: Vec::new(),
		}
	}

	pub fn view(&self, area: Area) -> SectionView {
		let mut values :Vec<usize> = Vec::with_capacity((area.size().x * area.size().y) as usize);
		let mut mapping: Vec<Vec<Sprite>> = Vec::new();
		for tile in self.region(area) {
			let tile_sprites = tile.sprites();
			values.push(
				if let Some(index) = mapping.iter().position(|x| x == &tile_sprites) {
					index
				} else {
					mapping.push(tile_sprites);
					mapping.len() - 1
				}
			);
		}
		SectionView {
			area,
			field: values,
			mapping
		}
	}
}


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct SectionView {
	pub field: Vec<usize>,
	pub mapping: Vec<Vec<Sprite>>,
	pub area: Area
}

pub type MapSave = Vec<(Pos, (Tile, Timestamp))>;

