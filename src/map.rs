
use std::collections::{HashMap, HashSet};
use crate::{
	pos::{Pos, Area, Direction},
	tile::{Tile, Structure, Ground},
	basemap::{BaseMap, BaseMapImpl},
	timestamp::{Timestamp, Duration},
	randomtick
};

pub struct Map {
	basemap: BaseMapImpl,
	changes: HashMap<Pos, (Tile, Timestamp)>,
	time: Timestamp,
	modifications: HashSet<Pos>
}

impl Map {
	
	pub fn new(basemap: BaseMapImpl, time: Timestamp) -> Self {
		Self {
			basemap,
			changes: HashMap::new(),
			time,
			modifications: HashSet::new()
		}
	}
	
	fn base_cell(&mut self, pos: Pos) -> Tile {
		self.basemap.cell(pos, self.time)
	}
	
	pub fn cell(&mut self, pos: Pos) -> Tile {
		self.changes.get(&pos).map(|change| change.0).unwrap_or_else(|| self.base_cell(pos))
	}

	pub fn load_area(&mut self, area: Area) -> impl Iterator<Item = (Pos, Tile)> + '_ {
		// let base_grid = self.basemap.region(area, self.time);
		self.basemap.region(area, self.time).into_iter().map(|(pos, base_cell)| {
			// let base_cell = self.base_cell(pos);
			self.tick_one(pos, base_cell);
			let cell = self.changes.get(&pos).map(|change| change.0).unwrap_or(base_cell);
			(pos, cell)
		})
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
		self.set(pos, new_tile )
	}
	
	pub fn set_ground(&mut self, pos: Pos, ground: Ground) {
		let new_tile = Tile::structure(ground, self.cell(pos).structure);
		self.set(pos, new_tile )
	}
	
	pub fn player_spawn(&mut self) -> Pos {
		self.basemap.player_spawn()
	}
	
	pub fn tick(&mut self, time: Timestamp, areas: Vec<Area>) {
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
			let base_cell = self.basemap.cell(pos, self.time);
			self.tick_one(pos, base_cell);
		}
	}
	
	fn tick_one(&mut self, pos: Pos, base_cell: Tile) {
		self.modifications.insert(pos);
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
			if built.structure.is_open()
					&& (built.ground.restoring() || built.ground == base_cell.ground)
					&& base_cell.structure.is_open() {
				self.changes.remove(&pos);
			}
		}
	}
	
	pub fn flush(&mut self) {
		self.modifications.clear()
	}
	
	pub fn modified(&mut self) -> HashMap<Pos, Tile> {
		self.modifications.clone().into_iter().map(|pos| (pos, self.cell(pos))).collect()
	}
	
	pub fn save(&self) -> MapSave {
		self.changes.clone().into_iter().collect()
	}
	
	pub fn load(changes: MapSave, time: Timestamp, basemap: BaseMapImpl) -> Self {
		Self {
			basemap,
			changes: changes.into_iter().collect(),
			time,
			modifications: HashSet::new()
		}
	}
}

pub type MapSave = Vec<(Pos, (Tile, Timestamp))>;

