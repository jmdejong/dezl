
use enum_dispatch::enum_dispatch;

use crate::{
	pos::{Pos, Area},
	timestamp::Timestamp,
	infinitemap::InfiniteMap,
	tiledmap::{TiledMap, MapLoadError},
	tile::Tile,
	config::MapDef,
};

#[derive(Debug)]
pub enum MapGenError {
	LoadError(MapLoadError)
}

#[enum_dispatch]
pub trait BaseMap {

	fn cell(&self, pos: Pos, time: Timestamp) -> Tile;
	
	fn region(&self, area: Area, time: Timestamp) -> Vec<(Pos, Tile)> {
		area.iter().map(|pos| (pos, self.cell(pos, time))).collect()
	}
	
	fn player_spawn(&self) -> Pos;
}


#[enum_dispatch(BaseMap)]
pub enum BaseMapImpl {
	InfiniteMap,
	TiledMap,
}

impl BaseMapImpl {
	pub fn from_mapdef(mapdef: MapDef) -> Result<BaseMapImpl, MapGenError> {
		Ok(match mapdef {
			MapDef::Infinite{seed} => InfiniteMap::new(seed).into(),
			MapDef::Tiled{path} => TiledMap::load(&path).map_err(MapGenError::LoadError)?.into(),
		})
	}
}
