
use std::collections::HashMap;

use crate::{
	player::{PlayerId},
	pos::{Pos, Area},
	creatures::Creatures
};

const DESPAWN_OFFSET: i32 = 32;

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

	pub fn update(&mut self, creatures: &Creatures) {
		self.fresh = HashMap::new();
		for (player_id, body) in creatures.iter_players() {
			let config = creatures.player_config(player_id).unwrap();
			let old_area = self.loaded.get(player_id);
			let screen_area = Area::centered(body.pos, config.view_size);
			let in_view_range = old_area.is_some_and(|area| area.contains_area(screen_area));
			if !in_view_range {
				let (total_area, new_area) = Self::new_area(screen_area, config.view_offset, old_area);
				self.loaded.insert(*player_id, total_area);
				self.fresh.insert(*player_id, new_area);
			}
		}
		self.loaded.retain(|player_id, _| creatures.get_player(player_id).is_some());
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

	fn new_area(screen_area: Area, offset: i32, view_area: Option<&Area>) -> (Area, Area) {
		let core_area = screen_area.grow(offset);
		let Some(old_area) = view_area else {
			return (core_area, core_area);
		};
		if core_area.size() != old_area.size() || !core_area.overlaps(old_area) {
			return (core_area, core_area);
		}
		if screen_area.min().x < old_area.min().x {
			let new_min = Pos::new(core_area.min().x, old_area.min().y);
			(Area::new(new_min, core_area.size()), Area::between(new_min, Pos::new(old_area.min().x, old_area.max().y)))
		} else if screen_area.min().y < old_area.min().y {
			let new_min = Pos::new(old_area.min().x, core_area.min().y);
			(Area::new(new_min, core_area.size()), Area::between(new_min, Pos::new(old_area.max().x, old_area.min().y)))
		} else if screen_area.max().x > old_area.max().x {
			let new_min = Pos::new(core_area.min().x, old_area.min().y);
			let new_area = Area::new(new_min, core_area.size());
			(new_area, Area::between(Pos::new(old_area.max().x, old_area.min().y), new_area.max()))
		} else if screen_area.max().y > old_area.max().y{
			let new_min = Pos::new(old_area.min().x, core_area.min().y);
			let new_area = Area::new(new_min, core_area.size());
			(new_area, Area::between(Pos::new(old_area.min().x, old_area.max().y), new_area.max()))
		} else {
			// this function shouldn't get called when this is the case, but let's do something somewhat sensible anyways
			(core_area, core_area)
		}
	}
}
