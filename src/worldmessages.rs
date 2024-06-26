
use std::collections::HashMap;
use serde::Serialize;
use crate::{
	pos::Pos,
	pos::Area,
	player::PlayerId,
	timestamp::Timestamp,
	creature::CreatureView,
	map::SectionView,
	tile::TileView,
};


#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorldMessage {
	#[serde(rename = "t")]
	pub tick: Timestamp,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub sounds: Vec<(SoundType, String)>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub me: Option<CreatureView>,
	#[serde(rename="changecells", skip_serializing_if = "Option::is_none")]
	pub change: Option<ChangeMessage>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub inventory: Option<InventoryMessage>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub viewarea: Option<ViewAreaMessage>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub section: Option<SectionView>,
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

worldmessages!(me, change,  inventory, viewarea, section, dynamics);

pub type ChangeMessage = Vec<(Pos, TileView)>;
pub type InventoryMessage = (Vec<(String, Option<usize>)>, Option<usize>);
pub type DynamicMessage = Vec<CreatureView>;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct ViewAreaMessage {
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
			self.cache.insert(*player, msg.clone());
		}
	}
	
	pub fn remove(&mut self, player: &PlayerId){
		self.cache.remove(player);
	}
}

