
use std::collections::HashMap;
use crate::{
	item::Item,
	worldmessages::InventoryMessage,
	controls::Selector,
};

const FIXED_ENTRIES: usize = 2;

#[derive(Debug, Clone)]
pub struct Inventory {
	items: Vec<(Item, usize)>,
	selector: usize
}

impl Inventory {
	
	pub fn add(&mut self, item: Item) {
		for entry in self.items.iter_mut() {
			if entry.0 == item {
				entry.1 += 1;
				return;
			}
		}
		self.items.push((item, 1));
	}
	
	pub fn view(&self) -> InventoryMessage {
		let view = [(Item::Eyes, 1), (Item::Hands, 1)].iter()
			.chain(self.items.iter())
			.map(|(item, count)| (item.name().to_string(), if item.quantified() { Some(*count) } else {None}))
			.collect();
		(view, self.selector)
	}
	
	pub fn save(&self) -> InventorySave {
		self.items.clone()
	}
	
	pub fn load(saved: InventorySave) -> Self {
		Self {
			items: saved,
			selector: 0
		}
	}
	
	fn count(&self) -> usize {
		self.items.len() + FIXED_ENTRIES
	}
	
	pub fn select(&mut self, selector: Selector) {
		self.selector = match selector {
			Selector::Next =>
				(self.selector + 1).rem_euclid(self.count()),
			Selector::Previous =>
				(self.selector + self.count() - 1).rem_euclid(self.count()),
			Selector::Idx(idx) =>
				idx.min(self.count() - 1),
		}
	}
	
	pub fn move_selected(&mut self, selector: Selector) {
		if self.selector < FIXED_ENTRIES {
			return;
		}
		let target = match selector {
			Selector::Next => self.selector + 1,
			Selector::Previous => self.selector - 1,
			Selector::Idx(idx) => idx,
		};
		if target < FIXED_ENTRIES || target >= self.count() {
			return;
		}
		let item = self.items.remove(self.selector - FIXED_ENTRIES);
		self.items.insert(target - FIXED_ENTRIES, item);
		self.select(selector);
	}
	
	pub fn selected(&self) -> Item {
		self.get_item(self.selector)
	}

	pub fn get_item(&self, index: usize) -> Item {
		if index == 0 {
			Item::Eyes
		} else if index == 1 {
			Item::Hands
		} else {
			self.items[index - FIXED_ENTRIES].0
		}
	}
	
	pub fn pay(&mut self, mut cost: HashMap<Item, usize>) -> bool {
		if cost.is_empty() {
			return true;
		}
		if let Some(items) = self.items.iter()
				.map(|(item, n)| {
					let amount = cost.remove(item).unwrap_or(0);
					if amount > *n {
						None
					} else {
						Some((*item, *n - amount))
					}
				})
				.collect::<Option<Vec<(Item, usize)>>>() {
			if !cost.is_empty() {
				false
			} else {
				self.items = items;
				self.items.retain(|(_, n)| *n > 0);
				self.selector = self.selector.min(self.count() - 1);
				true
			}
		} else {
			false
		}
	}
}

pub type InventorySave = Vec<(Item, usize)>;


#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn selects_eyes() {
		let inv = Inventory::load(vec![]);
		assert_eq!(inv.selected(), Item::Eyes);
	}
	#[test]
	fn selects_take_hands() {
		let mut inv = Inventory::load(vec![]);
		inv.select(Selector::Next);
		assert_eq!(inv.selected(), Item::Hands);
	}
	#[test]
	fn selects_stone() {
		let mut inv = Inventory::load(vec![(Item::Stone, 1)]);
		inv.select(Selector::Idx(2));
		assert_eq!(inv.selected(), Item::Stone);
	}
}

