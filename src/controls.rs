

use serde::{Serialize, Deserialize};
use crate::{
	PlayerId,
	Direction
};


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all="lowercase")]
pub enum Control {
	Move(Direction),
	Movement(Direction),
	Stop,
	Suicide,
	Interact(Option<Direction>),
	Use(usize, Option<Direction>),
	Select(Selector),
	MoveSelected(Selector),
}

#[derive(Debug, Clone)]
pub enum Action {
	Join(PlayerId),
	Leave(PlayerId),
	Input(PlayerId, Control)
}


#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all="lowercase")]
pub enum Selector {
	Next,
	Previous,
	Idx(usize),
}
