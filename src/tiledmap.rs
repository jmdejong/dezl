
use std::collections::HashMap;
use std::path::{PathBuf};
use serde::{Deserialize};
use std::fs;
use std::io::ErrorKind;

use crate::{
	pos::Pos,
	timestamp::Timestamp,
	basemap::{BaseMap},
	tile::{Tile, Ground, Structure},
	errors::AnyError,
};

#[derive(Debug)]
pub enum MapLoadError {
	MissingResource(AnyError),
	InvalidResource(AnyError),
}

pub struct TiledMap{
	chunks: HashMap<Pos, Vec<Tile>>,
	chunk_size: usize
}

impl TiledMap {

	pub fn load(path: &PathBuf) -> Result<TiledMap, MapLoadError> {
		let blueprint = load_tmx(path)?;

		let tileset = parse_tileset(blueprint.tilesets);



		let mut map = TiledMap {
			chunk_size: 32,
			chunks: HashMap::new()
		};
		for layer in blueprint.layers {
			if let TmxLayer::TileLayer{chunks} = layer {
				for tiled_chunk in chunks {
					for (i, tile_id) in tiled_chunk.data.iter().enumerate() {
						if let Some(tile) = tileset.get(tile_id) {
							let x = i as i32 % tiled_chunk.width + tiled_chunk.x;
							let y = i as i32 / tiled_chunk.width + tiled_chunk.y;
							let existing_tile = map.get_mut(Pos{x, y});
							if tile.ground != Ground::Empty {
								existing_tile.ground = tile.ground;
							}
							if tile.structure != Structure::Air {
								existing_tile.structure = tile.structure;
							}
						}
					}
				}
			}
		}

		Ok(map)
	}

	fn get_mut(&mut self, pos: Pos) -> &mut Tile {
		let chunk = self.chunks.entry(pos / self.chunk_size as i32)
			.or_insert_with(|| [Tile::empty()].repeat(self.chunk_size * self.chunk_size));
		let lp = pos % self.chunk_size as i32;
		chunk.get_mut(lp.x as usize + lp.y as usize * self.chunk_size).unwrap()
	}

}

impl BaseMap for TiledMap {

	fn cell(&self, pos: Pos, _time: Timestamp) -> Tile {
		let Some(chunk) = self.chunks.get(&(pos / self.chunk_size as i32)) else {
			return Tile::empty();
		};
		let lp = pos % self.chunk_size as i32;
		chunk[lp.x as usize + lp.y as usize * self.chunk_size]
	}

	fn player_spawn(&self) -> Pos {
		Pos::new(0, 0)
	}
}


fn load_tmx(path: &PathBuf) -> Result<TmxJson, MapLoadError> {

	let text = fs::read_to_string(path).map_err(|err| {
		if err.kind() == ErrorKind::NotFound {
			MapLoadError::MissingResource(Box::new(err))
		} else {
			MapLoadError::InvalidResource(Box::new(err))
		}
	})?;
	serde_json::from_str(&text).map_err(|err| MapLoadError::InvalidResource(Box::new(err)))
}

fn parse_tileset(tilesets: Vec<TmxTileset>) -> HashMap<i32, Tile> {
	tilesets.into_iter()
		.flat_map(|tileset|
			tileset.tiles.into_iter()
				.map(move |tile|( tile.id + tileset.firstgid, parse_tile_properties(tile.properties)))
		 )
		.collect()
}

fn parse_tile_properties(properties: Vec<TmxProperty>) -> Tile {
	let mut tile: Tile = Tile::empty();
	for property in properties {
		match property {
			TmxProperty::Ground(ground) => tile.ground = ground,
			TmxProperty::Structure(structure) => tile.structure = structure
		}
	}
	tile
}


#[derive(Debug, Deserialize)]
struct TmxJson {
	layers: Vec<TmxLayer>,
	tilesets: Vec<TmxTileset>
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum TmxLayer {
	TileLayer {
		chunks: Vec<TmxChunk>
	},
	ObjectGroup {},
}

#[derive(Debug, Deserialize)]
struct TmxChunk {
	x: i32,
	y: i32,
	width: i32,
	#[allow(dead_code)]
	height: i32,
	data: Vec<i32>
}

#[derive(Debug, Deserialize)]
struct TmxTileset {
	firstgid: i32,
	tiles: Vec<TmxTile>
}

#[derive(Debug, Deserialize)]
struct TmxTile {
	id: i32,
	properties: Vec<TmxProperty>
}

#[derive(Debug, Deserialize)]
#[serde(tag = "name", content = "value")]
enum TmxProperty {
	Ground(Ground),
	Structure(Structure)
}

