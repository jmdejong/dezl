
use std::collections::HashMap;

use crate::{
	player::PlayerId,
	pos::{Pos, Area},
};

const EDGE_OFFSET: i32 = 32;
const DESPAWN_OFFSET: i32 = 32;
const VIEW_AREA_SIZE: Pos = Pos::new(128, 128);


#[derive(Debug)]
pub struct LoadedAreas {
	loaded: HashMap<PlayerId, Area>,
	fresh: HashMap<PlayerId, Area>
}

impl LoadedAreas {
	pub fn new() -> Self {
		Self {
			loaded: HashMap::new(),
			fresh: HashMap::new(),
		}
	}

	pub fn update(&mut self, player_positions: &HashMap<PlayerId, Pos>) {
		self.fresh = HashMap::new();
		for (player_id, pos) in player_positions.iter() {
			let old_area = self.loaded.get(player_id);
			let in_view_range = old_area
					.map(|area| area.grow(-EDGE_OFFSET).contains(*pos))
					.unwrap_or(false);
			if !in_view_range {
				let (total_area, new_area) = Self::new_area(*pos, old_area);
				self.loaded.insert(player_id.clone(), total_area);
				self.fresh.insert(player_id.clone(), new_area);
			}
		}
		self.loaded.retain(|player_id, _| player_positions.contains_key(player_id));
	}

	pub fn all_loaded(&self) -> Vec<Area> {
		self.loaded.values().cloned().collect()
	}

	pub fn loaded(&self, player: &PlayerId) -> Option<Area> {
		self.loaded.get(player).copied()
	}

	pub fn all_fresh(&self) -> Vec<Area> {
		self.fresh.values().cloned().collect()
	}

	pub fn fresh(&self, player: &PlayerId) -> Option<Area> {
		self.fresh.get(player).copied()
	}

	pub fn is_loaded(&self, pos: Pos) -> bool {
		self.loaded.values().any(|area| area.grow(DESPAWN_OFFSET).contains(pos))
	}

	fn new_area(body_pos: Pos, view_area: Option<&Area>) -> (Area, Area) {
		let core_area = Area::centered(body_pos, VIEW_AREA_SIZE);
		let Some(old_area) = view_area else {
			return (core_area, core_area);
		};
		if !core_area.overlaps(old_area) {
			return (core_area, core_area);
		}
		if body_pos.x <= old_area.min().x + EDGE_OFFSET {
			let new_min = Pos::new(body_pos.x - VIEW_AREA_SIZE.x / 2, old_area.min().y);
			(Area::new(new_min, VIEW_AREA_SIZE), Area::between(new_min, Pos::new(old_area.min().x, old_area.max().y)))
		} else if body_pos.y <= old_area.min().y + EDGE_OFFSET {
			let new_min = Pos::new(old_area.min().x, body_pos.y - VIEW_AREA_SIZE.y / 2);
			(Area::new(new_min, VIEW_AREA_SIZE), Area::between(new_min, Pos::new(old_area.max().x, old_area.min().y)))
		} else if body_pos.x >= old_area.max().x - EDGE_OFFSET {
			let new_min = Pos::new(body_pos.x - VIEW_AREA_SIZE.x / 2, old_area.min().y);
			let new_area = Area::new(new_min, VIEW_AREA_SIZE);
			(new_area, Area::between(Pos::new(old_area.max().x, old_area.min().y), new_area.max()))
		} else if body_pos.y >= old_area.max().y - EDGE_OFFSET {
			let new_min = Pos::new(old_area.min().x, body_pos.y - VIEW_AREA_SIZE.y / 2);
			let new_area = Area::new(new_min, VIEW_AREA_SIZE);
			(new_area, Area::between(Pos::new(old_area.min().x, old_area.max().y), new_area.max()))
		} else {
			// this function shouldn't get called when this is the case, but let's do something somewhat sensible anyways
			(core_area, core_area)
		}
	}
}
