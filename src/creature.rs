

use serde::{Serialize, Deserialize};
use crate::{
	sprite::Sprite,
	Pos,
	PlayerId,
	timestamp::Duration,
	util::HolderId,
	inventory::{Inventory, InventorySave},
	worldmessages::SoundType,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mind {
	Player(PlayerId)
}

#[derive(Debug, Clone)]
pub struct Creature {
	pub mind: Mind,
	pub pos: Pos,
	pub cooldown: Duration,
	pub walk_cooldown: Duration,
	pub sprite: Sprite,
	pub inventory: Inventory,
	pub heard_sounds: Vec<(SoundType, String)>,
	is_dead: bool,
}

impl Creature {
	
	pub fn player(&self) -> Option<PlayerId> {
		match &self.mind {
			Mind::Player(id) => Some(id.clone())
		}
	}
	
	
	pub fn load_player(playerid: PlayerId, saved: PlayerSave) -> Self {
		Self {
			mind: Mind::Player(playerid),
			pos: saved.pos,
			cooldown: Duration(0),
			walk_cooldown: Duration(0),
			sprite: Sprite::PlayerDefault,
			inventory: Inventory::load(saved.inventory),
			heard_sounds: Vec::new(),
			is_dead: false
		}
	}
	
	pub fn kill(&mut self) {
		self.is_dead = true;
	}
	
	pub fn save(&self) -> PlayerSave {
		PlayerSave {
			pos: self.pos,
			inventory: self.inventory.save()
		}
	}

	pub fn view(&self) -> CreatureView {
		CreatureView {
			pos: self.pos,
			sprite: self.sprite
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CreatureId(pub usize);

impl HolderId for CreatureId {
	fn next(&self) -> Self { Self(self.0 + 1) }
	fn initial() -> Self { Self(1) }
}



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerSave {
	pub inventory: InventorySave,
	pub pos: Pos
}

impl PlayerSave {
	pub fn new(pos: Pos) -> Self {
		Self {
			pos,
			inventory: Vec::new()
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreatureView {
	#[serde(rename = "s")]
	pub sprite: Sprite,
	#[serde(rename = "p")]
	pub pos: Pos
}

