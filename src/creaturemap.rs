
use std::collections::HashMap;
use core::cell::Ref;

use crate::{
	pos::Pos,
	creature::{Creature, Faction},
	creatures::CreatureId,
};

#[derive(Debug)]
pub struct CreatureMap {
	map: HashMap<Pos, HashMap<CreatureId, CreatureTile>>,
	all: HashMap<CreatureId, CreatureTile>
}

impl CreatureMap {

	pub fn new<'a>(creatures: impl Iterator<Item=Ref<'a, Creature>>) -> Self {
		let mut map = Self { map: HashMap::new(), all: HashMap::new() };
		for creature in creatures {
			map.insert(creature.pos, &creature);
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

	pub fn blocking(&self, pos: Pos, creature: &CreatureTile) -> bool {
		self.map.get(&pos)
			.is_some_and(|creatures| creatures.values().any(|c| c.id != creature.id && (c.blocking || creature.blocking)))
	}


	pub fn move_creature(&mut self, creature: &Creature, from: &Pos, to: Pos) {
		if let Some(creatures) = self.map.get_mut(from) {
			creatures.remove(&creature.id);
			if creatures.is_empty() {
				self.map.remove(from);
			}
		}
		self.insert(to, creature);
	}

	fn insert(&mut self, pos: Pos, creature: &Creature) {
		let mut tile = CreatureTile::new(creature);
		tile.pos = pos;
		self.map.entry(pos).or_default().insert(creature.id, tile);
		self.all.insert(creature.id, tile);
	}
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreatureTile {
	pub id: CreatureId,
	pub faction: Faction,
	pub blocking: bool,
	pub pos: Pos,
}

impl CreatureTile {
	pub fn new(creature: &Creature) -> Self {
		Self {
			id: creature.id,
			pos: creature.pos,
			faction: creature.faction(),
			blocking: creature.blocking(),
		}
	}
}
