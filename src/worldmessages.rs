
use std::collections::HashMap;
use serde_json::{Value, json};
use serde::Serialize;
use crate::{
	Pos,
	Sprite,
	PlayerId,
	inventory::Item
};

macro_rules! worldmessages {
	($($name: ident, $typ: ident, $strname: expr, $filter: expr);*;) => {
	
		#[derive(Debug, Clone, Default, PartialEq)]
		pub struct WorldMessage {
			$(
				pub $name: Option<$typ>,
			)*
		}

		impl WorldMessage {
			
			pub fn remove_old(&mut self, previous: &WorldMessage){
				$(
					if $filter && self.$name == previous.$name {
						self.$name = None;
					}
				)*
			}
			
			pub fn add(&mut self, other: &WorldMessage){
				$(
					if other.$name.is_some() {
						self.$name = other.$name.clone();
					}
				)*
			}
			
			pub fn is_empty(&self) -> bool {
				true $( && self.$name.is_none())*
			}
			
			pub fn to_json(&self) -> Value {
				let mut updates: Vec<Value> = Vec::new();
				$(
					if let Some(update) = &self.$name {
						updates.push(json!([$strname, update]));
					}
				)*
				json!(["world", updates])
			}
		}
	}
}

worldmessages!(
	field, FieldMessage, "field", true;
	pos, Pos, "playerpos", true;
	change, ChangeMessage, "changecells", true;
	inventory, InventoryMessage, "inventory", true;
	ground, GroundMessage, "ground", true;
	sounds, SoundMessage, "messages", false;
);


pub type ChangeMessage = Vec<(Pos, Vec<Sprite>)>;
pub type InventoryMessage = (Vec<(Item, usize)>, usize);
pub type GroundMessage = Vec<String>;
pub type SoundMessage = Vec<(String, String, Option<Value>)>;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct FieldMessage {
	pub width: i32,
	pub height: i32,
	pub field: Vec<usize>,
	pub mapping: Vec<Vec<Sprite>>,
	pub offset: Pos
}

#[derive(Debug, Clone, Default)]
pub struct MessageCache {
	cache: HashMap<PlayerId, WorldMessage>
}

impl MessageCache {
	
	pub fn trim(&mut self, player: &PlayerId, msg: &mut WorldMessage){
		if let Some(cached) = self.cache.get_mut(player){
			msg.remove_old(cached);
			cached.add(msg);
		} else {
			self.cache.insert(player.clone(), msg.clone());
		}
	}
	
	pub fn remove(&mut self, player: &PlayerId){
		self.cache.remove(player);
	}
}



