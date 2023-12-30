
use std::fmt;
use serde::{Serialize, Deserialize};

use crate::{
	pos::Area,
	controls::Control,
	creature::CreatureId
};

#[derive(Debug, Default, PartialEq, Eq, Clone, Hash, Serialize, Deserialize)]
pub struct PlayerId(pub String);

impl fmt::Display for PlayerId {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.0)
	}
}

#[derive(Debug, Clone)]
pub struct Player {
	pub plan: Option<Control>,
	pub body: CreatureId,
	pub view_area: Option<Area>
}


impl Player {

	pub fn new(body: CreatureId) -> Self {
		Self {
			plan: None,
			body,
			view_area: None
		}
	}

	pub fn view_area(&self) -> Option<Area>{
		self.view_area
	}
}

