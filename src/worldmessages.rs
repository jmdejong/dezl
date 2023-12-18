
use std::collections::HashMap;
use serde::Serialize;
use crate::{
	Pos,
	pos::Area,
	Sprite,
	PlayerId,
	timestamp::Timestamp,
	creature::{CreatureView, Movement},
};


#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorldMessage {
	#[serde(rename = "t")]
	pub tick: Timestamp,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub sounds: Vec<(SoundType, String)>,
	#[serde(rename="playerpos", skip_serializing_if = "Option::is_none")]
	pub pos: Option<PositionMessage>,
	#[serde(rename="changecells", skip_serializing_if = "Option::is_none")]
	pub change: Option<ChangeMessage>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub inventory: Option<InventoryMessage>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub viewarea: Option<ViewAreaMessage>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub section: Option<SectionMessage>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub dynamics: Option<DynamicMessage>,

}


macro_rules! worldmessages {
	($($name: ident),*) => {
	
		impl WorldMessage {

			pub fn new(tick: Timestamp) -> Self {
				Self {
					tick,
					sounds: Vec::new(),
					$(
						$name: None,
					)*
				}
			}
			pub fn remove_old(&mut self, previous: &WorldMessage){
				$(
					if self.$name == previous.$name {
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
		}
	}
}

worldmessages!(pos, change,  inventory, viewarea, section, dynamics);

pub type ChangeMessage = Vec<(Pos, Vec<Sprite>)>;
pub type InventoryMessage = (Vec<(String, Option<usize>)>, usize);
pub type DynamicMessage = Vec<CreatureView>;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct PositionMessage {
	pub pos: Pos,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub movement: Option<Movement>,
}

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



