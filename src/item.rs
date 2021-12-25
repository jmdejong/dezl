
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all="lowercase")]
pub enum ItemRef {
	Axe,
	OakWood,
	OakNut,
	RadishSeed,
	Radish,
}

pub struct ItemDef {
	is_tool: bool,
}

impl ItemRef {
	pub fn properties(&self) -> ItemDef {
		match self {
			ItemRef::Axe => ItemDef{is_tool: true},
			ItemRef::OakWood => ItemDef{is_tool: false},
			ItemRef::OakNut => ItemDef{is_tool: false},
			ItemRef::RadishSeed => ItemDef{is_tool: false},
			ItemRef::Radish => ItemDef{is_tool: false}
		}
	}
}
