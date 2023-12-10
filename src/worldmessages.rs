
use std::collections::HashMap;
use serde_json::{Value, json};
use serde::Serialize;
use crate::{
	Pos,
	pos::Area,
	Sprite,
	PlayerId,
	timestamp::Timestamp,
	creature::CreatureView
};

macro_rules! worldmessages {
	($($name: ident, $typ: ident, $strname: expr, $filter: expr);*;) => {
	
		#[derive(Debug, Clone, PartialEq, Eq)]
		pub struct WorldMessage {
			pub tick: Timestamp,
			$(
				pub $name: Option<$typ>,
			)*

		}

		impl WorldMessage {

			pub fn new(tick: Timestamp) -> Self {
				Self {
					tick,
					$(
						$name: None,
					)*
				}
			}
			
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
				json!(["world", updates, self.tick])
			}
		}
	}
}

worldmessages!(
	pos, Pos, "playerpos", true;
	change, ChangeMessage, "changecells", true;
	inventory, InventoryMessage, "inventory", true;
	sounds, SoundMessage, "messages", false;
	viewarea, ViewAreaMessage, "viewarea", true;
	section, SectionMessage, "section", true;
	dynamics, DynamicMessage, "dynamics", true;
);


pub type ChangeMessage = Vec<(Pos, Vec<Sprite>)>;
pub type InventoryMessage = (Vec<(String, Option<usize>)>, usize);
pub type SoundMessage = Vec<(SoundType, String)>;
pub type DynamicMessage = Vec<CreatureView>;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct ViewAreaMessage {
	pub area: Area
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct SectionMessage {
	pub field: Vec<usize>,
	pub mapping: Vec<Vec<Sprite>>,
	pub area: Area
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all="lowercase")]
pub enum SoundType {
	BuildError,
	Explain
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



