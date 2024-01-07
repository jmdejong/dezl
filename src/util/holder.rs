

use std::collections::HashMap;
use std::collections::hash_map::{Iter, IterMut, Values};
use std::hash::Hash;

pub trait HolderId: Copy + Eq + Hash {
	fn next(&self) -> Self;
	fn initial() -> Self;
}


#[derive(Debug, Clone)]
pub struct Holder <K: HolderId, V> {
	storage: HashMap<K, V>,
	next_key: K
}

impl <K: HolderId, V> Holder<K, V> {
	pub fn new() -> Self {
		Self {
			next_key: K::initial(),
			storage: HashMap::new()
		}
	}
	
	pub fn insert(&mut self, value: V) -> K {
		let key = self.next_key;
		self.next_key = key.next();
		self.storage.insert(key, value);
		key
	}
	
	#[allow(dead_code)]
	#[inline]
	pub fn contains_key(&self, key: &K) -> bool {
		self.storage.contains_key(key)
	}
	
	#[inline]
	pub fn get(&self, key: &K) -> Option<&V> {
		self.storage.get(key)
	}
	
	#[inline]
	pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
		self.storage.get_mut(key)
	}
	
	#[inline]
	pub fn remove(&mut self, key: &K) -> Option<V> {
		self.storage.remove(key)
	}
	
	#[allow(dead_code)]
	#[inline]
	pub fn iter(&self) -> Iter<K, V> {
		self.storage.iter()
	}
	
	#[inline]
	pub fn iter_mut(&mut self) -> IterMut<K, V> {
		self.storage.iter_mut()
	}
	
	#[allow(dead_code)]
	#[inline]
	pub fn values(&self) -> Values<K, V> {
		self.storage.values()
	}
}

impl HolderId for usize {
	fn next(&self) -> Self { self + 1 }
	fn initial() -> Self { 1 }
}



