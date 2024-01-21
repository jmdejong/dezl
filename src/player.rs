
use std::fmt;
use serde::{Serialize, Deserialize, Serializer, Deserializer};

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy, Hash)]
pub struct PlayerId{
	len: u8,
	bytes: [u8; 15],
}

impl PlayerId {
	pub fn new(name: &str) -> Self {
		let len = name.as_bytes().len();
		if len > 15 {
			panic!("player name {} is too long (max 15 bytes allowed)", name);
		}
		let mut id = Self {
			len: len as u8,
			bytes: Default::default()
		};
		id.bytes[..len].copy_from_slice(name.as_bytes());
		id
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
		Ok(Self::new(<&str>::deserialize(deserializer)?))
	}
}

