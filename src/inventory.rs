
use std::collections::HashMap;
use crate::{
	item::Item,
	worldmessages::InventoryMessage,
};

const FIXED_ENTRIES: usize = 2;

#[derive(Debug, Clone, PartialEq)]
pub struct Inventory {
	items: Vec<(Item, usize)>
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
		(view, None)
	}
	
	pub fn save(&self) -> InventorySave {
		self.items.clone()
	}
	
	pub fn load(saved: InventorySave) -> Self {
		Self {
			items: saved,
		}
	}
	
	fn count(&self) -> usize {
		self.items.len() + FIXED_ENTRIES
	}
	
	pub fn move_item(&mut self, from: usize, target: usize) {
		if from < FIXED_ENTRIES || target < FIXED_ENTRIES || from > self.count() || target > self.count() || from == target{
			return;
		}
		let item = self.items.remove(from - FIXED_ENTRIES);
		self.items.insert(target - FIXED_ENTRIES, item);
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
	fn has_default_entries() {
		let inv = Inventory::load(vec![]);
		assert_eq!(inv.get_item(0), Item::Eyes);
		assert_eq!(inv.get_item(1), Item::Hands);
	}
	#[test]
	fn has_item() {
		let inv = Inventory::load(vec![(Item::Stone, 1)]);
		assert_eq!(inv.get_item(2), Item::Stone);
	}
	#[test]
	fn moves_item() {
		let mut inv = Inventory::load(vec![(Item::Stone, 1), (Item::Stick, 1), (Item::Ash, 1), (Item::Log, 1), (Item::Hoe, 1)]);
		inv.move_item(3, 5);
		let expected = Inventory::load(vec![(Item::Stone, 1), (Item::Ash, 1), (Item::Log, 1), (Item::Stick, 1), (Item::Hoe, 1)]);
		assert_eq!(inv, expected);

	}
}

