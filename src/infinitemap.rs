

use crate::{
	pos::{Pos, Area},
	timestamp::Timestamp,
	tile::{Tile, Ground, Structure},
	basemap::BaseMap,
	random::{WhiteNoise, randomize_u32, pick, pick_weighted},
	randomtick,
	util::math,
	heightmap::{HeightMap, LazyHeightMap}
};

macro_rules! t {
	($g:ident) => {Tile::ground(Ground::$g)};
	($g:ident, $s:ident) => {Tile::structure(Ground::$g, Structure::$s)};
	($g:expr) => {Tile::ground($g)};
	($g:expr, $s:expr) => {Tile::structure($g, $s)};
}

const BIOME_SIZE: i32 = 48;
const EDGE_SIZE: i32 = BIOME_SIZE / 4;

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
enum Biome {
	Start,
	Forest,
	Field,
	Lake,
	Rocks,
	Bog
}


pub struct InfiniteMap {
	seed: u32,
	heightmaps: HeightMaps<LazyHeightMap>
}

impl InfiniteMap {
	pub fn new(seed: u32) -> Self {
		Self {
			seed,
			heightmaps: HeightMaps {
				lake: LazyHeightMap::new(seed + 344, vec![(3,0.12), (5,0.20), (7,0.26), (11,0.42)]),
				rock: LazyHeightMap::new(seed + 344, vec![(3,0.12), (5,0.20), (7,0.26), (11,0.42)]),
				bog: LazyHeightMap::new(seed + 382, vec![(3,0.25), (5,0.35), (7,0.4)]),
				reed: LazyHeightMap::new(seed+276, vec![(7, 0.5), (11, 0.5)])
			},
		}
	}


	fn start_biome(&self) -> BPos {
		BPos(Pos::new(0, 0))
	}

	fn start_pos(&self) -> Pos {
		self.biome_core(self.start_biome()) + Pos::new(0, 2)
	}

	fn biome_at(&self, b_pos: BPos) -> Biome {
		if b_pos == self.start_biome() {
			Biome::Start
		} else {
			*pick_weighted(
				WhiteNoise::new(self.seed+333).gen(b_pos.0),
				&[
					(Biome::Field, 10),
					(Biome::Forest, 10),
					(Biome::Lake, 5),
					(Biome::Rocks, 5),
					(Biome::Bog, 5),
				]
			)
		}
	}

	fn biome_core(&self, bpos: BPos) -> Pos {
		let rind = WhiteNoise::new(self.seed+821).gen(bpos.0);
		let core_size = BIOME_SIZE / 2;
		let core_offset = if bpos == self.start_biome() {
			Pos::new(0, 0)
		} else {
			Area::centered(Pos::new(0, 0), Pos::new(core_size, core_size))
				.random_pos(rind)
		};
		bpos.0 * BIOME_SIZE + core_offset + Pos::new(bpos.0.y * BIOME_SIZE / 2, 0)
	}

	fn neighbour_biomes(&self, pos: Pos) -> impl Iterator<Item=(i32, BPos)> + '_ {
		let bpos = BPos(Pos::new(pos.x - (pos.y / 2), pos.y)  / BIOME_SIZE);
		[(0, 0), (1, 0), (0, 1), (1, 1), (0, 2), (1, 2)].into_iter()
			.map(move |p| {
				let b = BPos(bpos.0 + p);
				let dist = pos.distance_to(self.biome_core(b));
				(dist, b)
			})
	}

	fn closest_biome_pos(&self, pos: Pos) -> BPos {
		self.neighbour_biomes(pos)
			.min_by_key(|(d, _)| *d)
			.unwrap()
			.1
	}

	fn edge_distance(&self, pos: Pos) -> i32 {
		let mut distances: Vec<(i32, BPos)> = self.neighbour_biomes(pos)
			.collect();
		distances.sort_by_key(|(d, _)| *d);
		let (dist, bpos) = distances[0];
		let my_biome = self.biome_at(bpos);
		distances[1..].iter()
			.find(|(_, b)| self.biome_at(*b) != my_biome)
			.map(|(d, _)| d - dist)
			.unwrap_or(BIOME_SIZE / 2)
	}

	fn biome_pos(&self, pos: Pos) -> (BPos, Pos) {
		let rind = WhiteNoise::new(self.seed+343).gen(pos);
		let edge_size = EDGE_SIZE;
		let mut offset = Pos::new((rind % edge_size as u32) as i32 - edge_size / 2, ((rind / edge_size as u32) % edge_size as u32) as i32 - edge_size / 2);
		if offset.size() > edge_size / 2 {
			offset = offset % edge_size - Pos::new(edge_size/2, edge_size/2);
		}
		let fuzzy_pos = pos + offset;
		let b_pos = self.closest_biome_pos(fuzzy_pos);
		let dpos = pos - self.biome_core(b_pos);
		(b_pos, dpos)
	}


	fn tile(&self, pos: Pos, time: Timestamp) -> Tile {
		let (bpos, dpos) = self.biome_pos(pos);
		let biome = self.biome_at(bpos);
		let rind = WhiteNoise::new(self.seed + 7943).gen(pos);
		let rtime = randomtick::tick_num(pos, time) as u32 + WhiteNoise::new(self.seed + 356).gen(pos);
		match biome {
			Biome::Start => {
				let dspawn = dpos.abs();
				if dspawn.x == 0 && dspawn.y == 0 {
					t!(StoneFloor, MarkerAltar)
				} else if dspawn.x <= 4 && dspawn.y <= 4 && !(dspawn.y == 4 && dspawn.x == 4){
					if dspawn.x + dspawn.y <= 5 {
						t!(StoneFloor)
					} else {
						t!(StoneFloor, Wall)
					}
				} else if dspawn.x <= 1 || dspawn.y <= 1 {
					t!(Dirt)
				} else if Area::centered(Pos::new(9, -8), Pos::new(5, 5)).contains(dpos) {
					let dhouse = dpos - Pos::new(9, -8);
					if dhouse == Pos::new(0, -1) {
						t!(Dirt, Sage)
					} else if dhouse  == Pos::new(1, 1) {
						t!(Dirt, GardeningTable)
					} else if dhouse == Pos::new(0, 2) || dhouse.abs().x < 2 && dhouse.abs().y < 2 {
						t!(Dirt)
					} else {
						t!(Dirt, WoodWall)
					}

				} else {
					*pick(rind, &[
						t!(Grass1),
						t!(Grass2),
						t!(Grass3)
					])
				}
			}
			Biome::Field => {
				t!(
					*pick(rind, &[
						Ground::Grass1,
						Ground::Grass2,
						Ground::Grass3
					]),
					if WhiteNoise::new(self.seed + 9429).gen_f(pos) < 0.02 {
						Structure::Shrub
					} else {
						*pick_weighted(randomize_u32(randomize_u32(rtime/4).wrapping_add(5924)), &[
							(Structure::Air, 40),
							(Structure::DenseGrassGrn, 4),
							(Structure::DenseGrassBrn, 3),
							(Structure::DenseGrassY, 3),
							(Structure::Flower, 1)
						])
					}
				)
			}
			Biome::Forest => {
				*pick_weighted(rtime, &[
					(*pick_weighted(rind, &[
						(t!(Grass1), 10),
						(t!(Grass2), 10),
						(t!(Grass3), 10),
						(t!(Moss), 40),
						(t!(DeadLeaves), 40),
						(t!(Dirt), 30),
						(t!(Dirt, Stick), 1)
					]), 100),
					(t!(Grass1, Sapling), 3),
					(t!(Dirt, YoungTree), 4),
					(t!(Dirt, Tree), 13),
					(t!(Dirt, OldTreeTinder), 1),
					(t!(Dirt), 1)
				])
			}
			Biome::Lake => {
				let c = ((self.edge_distance(pos) - EDGE_SIZE) as f32 / 12.0).clamp(0.0, 1.0);
				let reed_density = self.heightmaps.reed.height(pos) * 0.4 - 0.2;
				let height = 0.4 - self.heightmaps.lake.height(pos) + (1.0 - c) * 0.6;
				if height.abs() < reed_density {
					t!(
						if height > 0.0 { Ground::Dirt } else { Ground::Water },
						if randomtick::tick_num(pos, time).rem_euclid(4) as u32 != rind.rem_euclid(4) {
							Structure::Reed
						} else {
							Structure::Air
						}
					)
				} else if height < 0.0 {
					t!(Water)
				} else {
					*pick_weighted(rind, &[
						(t!(Grass1), 10),
						(t!(Grass2), 10),
						(t!(Grass3), 10),
						(t!(Grass1, DenseGrassGrn), 3),
						(t!(Grass2, DenseGrassBrn), 3),
						(t!(Grass3, DenseGrassY), 3),
						(t!(Grass1, Shrub), 2)
					])
				}
			}
			Biome::Rocks => {
				let min_height = 0.6;
				let height = self.rock_height(pos);
				if height > min_height {
					let ismid = [(0, -1), (0, 1), (-1, 0), (1, 0), (-1, -1), (-1, 1), (1, -1), (1, 1)]
						.into_iter()
						.all(|d| self.rock_height(pos + d) > min_height);
					t!(
						Ground::RockFloor,
						if ismid {
							Structure::RockMid
						} else {
							Structure::Rock
						}
					)
				} else {
					*pick_weighted((height * 100.0) as u32, &[
						(*pick_weighted(rind, &[
							(t!(Grass1), 10),
							(t!(Grass2), 10),
							(t!(Dirt), 1),
							(t!(RockFloor), (height * 10.0) as u32),
						]), 50),
						(*pick_weighted(rind, &[
							(t!(Grass1), 1),
							(t!(Grass2), 1),
							(*pick_weighted(rtime, &[
								(t!(RockFloor, Gravel), 20),
								(t!(RockFloor), 50),
								(t!(RockFloor, Stone), 5),
								(t!(RockFloor, Gravel), 20),
								(t!(RockFloor), 50),
								(t!(RockFloor, Pebble), 3),
								(t!(RockFloor), 50),
							]), 3),
						]), 50),
					])
				}
			}
			Biome::Bog => {
				let height = self.heightmaps.bog.height(pos) + WhiteNoise::new(self.seed+3294).gen_f(pos) * 0.1;
				if height < 0.45 {
					t!(Water)
				} else {
					*pick_weighted(rind, &[
						(t!(Grass1), 50),
						(t!(Grass2), 50),
						(t!(Grass3), 50),
						(t!(Grass1, Shrub), 1),
						(t!(Grass1, Rush), 10),
						(*pick(
							rtime / 2,
							&[
								t!(Grass1, PitcherPlant),
								t!(Grass1)
							]
						), 1)
					])
				}
			}
		}
	}

	fn rock_height(&self, pos: Pos) -> f32 {
		let c = ((self.edge_distance(pos) - EDGE_SIZE) as f32 / 4.0).clamp(0.0, 1.0);
		math::ease_in_out_cubic(self.heightmaps.rock.height(pos)) * c
	}
}

impl BaseMap for InfiniteMap {
	fn cell(&mut self, pos: Pos, time: Timestamp) -> Tile {
		self.tile(pos, time)
	}


	fn region(&mut self, area: Area, time: Timestamp) -> Vec<(Pos, Tile)> {
		area.iter().map(|pos| (pos, self.tile(pos, time))).collect()
	}

	fn player_spawn(&mut self) -> Pos {
		self.start_pos()
	}
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Default)]
struct BPos(Pos);

struct HeightMaps<T: HeightMap> {
	lake: T,
	bog: T,
	rock: T,
	reed: T
}


#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn core_is_in_own_biome() {
		let map = InfiniteMap::new(678);
		for x in -15..15 {
			for y in -15..15 {
				let bpos = BPos(Pos::new(x, y));
				assert_eq!(bpos, map.closest_biome_pos(map.biome_core(bpos)));
				assert_eq!((bpos, Pos::new(0, 0)), map.biome_pos(map.biome_core(bpos)));
			}
		}
	}

	#[test]
	fn start_is_start_biome() {
		let map = InfiniteMap::new(9876);
		assert_eq!(map.biome_at(map.biome_pos(map.start_pos()).0), Biome::Start);
	}

	#[test]
	fn start_pos_has_stone_floor() {
		let mut map = InfiniteMap::new(9876);
		assert_eq!(map.cell(map.start_pos(), Timestamp(1)), t!(StoneFloor));
	}

	#[test]
	fn generating_region_gives_same_result_as_separate_cells(){
		let mut map = InfiniteMap::new(9876);
		let time = Timestamp(3412);
		let mut failed = Vec::new();
		let mut total = 0;
		for cx in -2..=2 {
			for cy in -2..=2 {
				let area = Area::centered(Pos::new(221 + cx * 1357 , -873 + cy * 2931), Pos::new(32, 32));
				let region = map.region(area, time);
				for (pos, tile) in region {
					total += 1;
					let single = map.cell(pos, time);
					if tile != single {
						failed.push((pos, tile, single));
					// assert_eq!(tile,  "unequal tiles on pos {:?}", pos);
					}
				}
				// let region_map: HashMap<Pos, Tile> = region.into_iter().collect();
				// for pos in area.iter() {
					// assert_eq(region_map
				// }
				// let separate: Vec<(Pos, Tile)> = area.iter().map(|pos| (pos, map.cell(pos, time))).collect();
				// assert_eq!(region, separate);
			}
		}
		eprintln!("total: {}, failed: {}", total, failed.len());
		assert_eq!(failed, Vec::new());
	}
}
