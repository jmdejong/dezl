
use std::fmt;
use serde::{Serialize, Deserialize, Serializer, Deserializer, de};
use crate::{
	pos::Pos,
};

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy, Hash)]
pub struct PlayerId{
	len: u8,
	bytes: [u8; 14],
}

impl PlayerId {
	pub fn create(name: &str) -> Result<Self, String> {
		let len = name.as_bytes().len();
		if len > 14 {
			return Err(format!("player name {} is too long. Max 14 bytes allowed while length is {}", name, len));
		}
		let mut id = Self {
			len: len as u8,
			bytes: Default::default()
		};
		id.bytes[..len].copy_from_slice(name.as_bytes());
		Ok(id)
	}

	pub fn name(&self) -> &str {
		std::str::from_utf8(&self.bytes[..self.len as usize]).unwrap()
	}
}

impl fmt::Display for PlayerId {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.name())
	}
}

impl Serialize for PlayerId {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where S: Serializer {
		self.name().serialize(serializer)
	}
}
impl<'de> Deserialize<'de> for PlayerId {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where D: Deserializer<'de> {
		Self::create(<&str>::deserialize(deserializer)?).map_err(de::Error::custom)
	}
}


#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(rename_all="lowercase")]
pub struct PlayerConfigMsg {
	pub view_size: Option<Pos>,
	pub view_offset: Option<i32>,
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PlayerConfig {
	pub view_size: Pos,
	pub view_offset: i32,
}
impl Default for PlayerConfig {
	fn default() -> Self {
		Self {
			view_size: Pos::new(64, 64),
			view_offset: 16,
		}
	}
}

impl PlayerConfig {
	pub fn update(&mut self, message: PlayerConfigMsg) {
		if let Some(view_offset) = message.view_offset {
			self.view_offset = view_offset.clamp(8, 32);
		}
		if let Some(view_size) = message.view_size {
			self.view_size = Pos::new(view_size.x.clamp(8, 64), view_size.y.clamp(8, 64));
		}
	}
}
