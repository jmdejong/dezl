

use serde::{Serialize, Deserialize};
use crate::{
	PlayerId,
	pos::{Direction, Pos},
};


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all="lowercase")]
pub enum Plan {
	Move(Direction),
	Use(usize, Option<Direction>),
	Take(Option<Direction>),
	Inspect(Option<Direction>),
	Fight(Option<Direction>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all="lowercase")]
pub enum DirectChange {
	MoveItem(usize, usize),
	Movement(Option<Direction>),
	Path(Vec<Pos>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Control {
	Plan(Plan),
	Direct(DirectChange)
}

#[derive(Debug, Clone)]
pub enum Action {
	Join(PlayerId, String),
	Leave(PlayerId),
	Input(PlayerId, Control)
}

