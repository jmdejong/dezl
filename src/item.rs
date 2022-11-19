

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use enum_assoc::Assoc;
use crate::{
	action::{Action, InteractionType::*, CraftType},
	tile::Structure,
	hashmap,
	crop::Crop,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Assoc)]
#[serde(rename_all="snake_case")]
#[func(pub fn actions(&self) -> Vec<Action> {Vec::new()})]
#[func(pub fn description(&self) -> Option<&str>)]
#[func(pub fn name(&self) -> &str)]
pub enum Item {
	#[assoc(name="<eyes>")]
	#[assoc(actions=vec![Action::Inspect])]
	#[assoc(description="Inspect things around you")]
	Eyes,
	
	#[assoc(name="<hands>")]
	#[assoc(actions=vec![Action::take()])]
	#[assoc(description="Take items that are laying loose")]
	Hands,
	
	#[assoc(name="reed")]
	#[assoc(description="Some cut reeds")]
	Reed,
	
	#[assoc(name="flower")]
	#[assoc(description="A pretty flower")]
	#[assoc(actions=vec![Action::Craft(CraftType::Marker, Item::MarkerStone, hashmap![Item::Stone => 1, Item::Flower => 9])])]
	Flower,
	
	#[assoc(name="pebble")]
	#[assoc(description="A small stone")]
	Pebble,
	
	#[assoc(name="stone")]
	#[assoc(description="A mid-size stone. Stones can be broken by smashing two together")]
	#[assoc(actions=vec![Action::interact(Smash, 1, true)])]
	Stone,
	
	#[assoc(name="sharp stone")]
	#[assoc(description="A small stone with a sharp edge. It can be used to cut things, though it is very crude and may not always work")]
	#[assoc(actions=vec![Action::interact(Cut, 1, false)])]
	SharpStone,
	
	#[assoc(name="pitcher")]
	#[assoc(description="A pitcher from the pitcher plant. It can function as a bucket")]
	#[assoc(actions=vec![Action::Craft(CraftType::Water, Item::FilledPitcher, HashMap::new())])]
	Pitcher,
	
	#[assoc(name="water pitcher")]
	#[assoc(description="A pitcher from the pitcher plant, filled with water")]
	#[assoc(actions=vec![Action::interact_change(Water, 1, Item::Pitcher)])]
	FilledPitcher,
	
	#[assoc(name="hoe")]
	#[assoc(description="A simple hoe that can be used to clear the ground of small vegetation")]
	#[assoc(actions=vec![Action::Clear])]
	Hoe,
	
	#[assoc(name="green seed")]
	#[assoc(description="Unknown green seed")]
	#[assoc(actions=vec![Action::Build(Structure::Crop(Crop::greenseed()), HashMap::new())])]
	GreenSeed,
	
	#[assoc(name="yellow seed")]
	#[assoc(actions=vec![Action::Build(Structure::Crop(Crop::yellowseed()), HashMap::new())])]
	#[assoc(description="Unknown yellow seed")]
	YellowSeed,
	
	#[assoc(name="brown seed")]
	#[assoc(actions=vec![Action::Build(Structure::Crop(Crop::brownseed()), HashMap::new())])]
	#[assoc(description="Unknown brown seed")]
	BrownSeed,
	
	#[assoc(name="stick")]
	#[assoc(description="Stick")]
	#[assoc(actions=vec![
		Action::Craft(CraftType::GardeningTable, Item::Hoe, hashmap![Item::Reed => 1, Item::SharpStone => 1]),
		Action::interact(Fuel, 1, true)
	])]
	Stick,
	
	#[assoc(name="discleaf")]
	#[assoc(description="Disk leaf")]
	#[assoc(actions=vec![
		Action::interact(Fuel, 1, true)
	])]
	DiscLeaf,
	
	#[assoc(name="knifeleaf")]
	#[assoc(description="Knife leaf")]
	#[assoc(actions=vec![
		Action::interact(Cut, 2, true)
	])]
	KnifeLeaf,
	
	#[assoc(name="hardwood stick")]
	#[assoc(description="A strong stick")]
	#[assoc(actions=vec![
		Action::interact(Fuel, 2, true)
	])]
	HardwoodStick,
	
	#[assoc(name="wood knife")]
	#[assoc(description="A surprisingly effective wooden knife")]
	#[assoc(actions=vec![
		Action::interact(Cut, 2, false)
	])]
	HardwoodKnife,
	
	#[assoc(name="wood table")]
	#[assoc(description="A wooden table")]
	#[assoc(actions=vec![
		Action::Build(Structure::HardwoodTable, HashMap::new())
	])]
	HardwoodTable,
	
	#[assoc(name="tinder")]
	#[assoc(description="Tinder from the tinder fungus. Can be placed with some pebbles on a clear space to create a fireplace")]
	#[assoc(actions=vec![Action::Build(Structure::Fireplace, hashmap![Item::Pebble => 10])])]
	Tinder,
	
	#[assoc(name="marker stone")]
	#[assoc(description="A marker stone that can be placed to create a land claim")]
	#[assoc(actions=vec![Action::BuildClaim(Structure::MarkStone)])]
	MarkerStone,
	
	#[assoc(name="ash")]
	#[assoc(description="Wood ash. Can be used as fertilizer")]
	#[assoc(actions=vec![Action::interact(Fertilize, 1, true)])]
	Ash,
}


#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn hands_has_take_action() {
		assert_eq!(Item::Hands.action(), Some(Action::take()));
	}
	
}
