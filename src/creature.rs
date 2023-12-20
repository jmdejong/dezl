

use serde::{Serialize, Deserialize};
use crate::{
	sprite::Sprite,
	Pos,
	PlayerId,
	timestamp::Duration,
	util::HolderId,
	inventory::{Inventory, InventorySave},
	worldmessages::SoundType,
	timestamp::Timestamp,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mind {
	Player(PlayerId)
}

#[derive(Debug, Clone)]
pub struct Creature {
	pub mind: Mind,
	pub pos: Pos,
	walk_cooldown: Duration,
	pub sprite: Sprite,
	pub inventory: Inventory,
	pub heard_sounds: Vec<(SoundType, String)>,
	is_dead: bool,
	movement: Option<Movement>,
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
			walk_cooldown: Duration(4),
			sprite: Sprite::PlayerDefault,
			inventory: Inventory::load(saved.inventory),
			heard_sounds: Vec::new(),
			is_dead: false,
			movement: None,
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
			sprite: self.sprite,
			movement: self.movement.clone(),
		}
	}

	pub fn move_to(&mut self, newpos: Pos, time: Timestamp) {
		self.movement = Some(Movement {
			from: self.pos,
			start: time,
			end: time + self.walk_cooldown
		});
		self.pos = newpos;
	}

	pub fn current_movement(&self, time: Timestamp) -> Option<Movement> {
		if time < self.movement.as_ref()?.end {
			self.movement.clone()
		} else {
			None
		}
	}

	pub fn can_move(&self, time: Timestamp) -> bool {
		if let Some(movement) = &self.movement {
			time >= movement.end
		} else {
			true
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
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
	pub pos: Pos,
	#[serde(skip_serializing_if = "Option::is_none", rename="m")]
	pub movement: Option<Movement>,
}


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct Movement {
	#[serde(rename = "f")]
	pub from: Pos,
	#[serde(rename = "s")]
	pub start: Timestamp,
	#[serde(rename = "e")]
	pub end: Timestamp,
}

