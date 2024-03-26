

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use enum_assoc::Assoc;
use crate::{
	action::{Action, InteractionType::*, CraftType},
	tile::Structure,
	hashmap,
	crop::Crop,
	sprite::Sprite,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Assoc)]
#[serde(rename_all="snake_case")]
#[func(pub fn actions(&self) -> Vec<Action> {Vec::new()})]
#[func(pub fn description(&self) -> &str)]
#[func(pub fn name(&self) -> &str)]
#[func(pub fn quantified(&self) -> bool {true})]
#[func(pub fn sprite(&self) -> Option<Sprite>)]
pub enum Item {
	#[assoc(name="reed")]
	#[assoc(description="Some cut reeds")]
	Reed,
	
	#[assoc(name="flower")]
	#[assoc(description="A pretty flower")]
	#[assoc(actions=vec![Action::Craft(CraftType::Marker, Item::MarkerStone, hashmap![Item::Stone => 1, Item::Flower => 9])])]
	Flower,
	
	#[assoc(name="pebble")]
	#[assoc(sprite = Sprite::Pebble)]
	#[assoc(description="Pebble. A small stone")]
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
	#[assoc(description="Wooden stick")]
	#[assoc(sprite = Sprite::Stick)]
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
		Action::Craft(CraftType::GardeningTable, Item::Axe, hashmap![Item::Reed => 1, Item::HardwoodStick=> 1]),
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
	
	#[assoc(name="axe")]
	#[assoc(description="A wooden axe")]
	#[assoc(actions=vec![
		Action::interact(Chop, 2, false)
	])]
	Axe,
	
	#[assoc(name="log")]
	#[assoc(description="Wooden log")]
	#[assoc(actions=vec![
		Action::Craft(CraftType::SawTable, Item::Plank, HashMap::new()),
		Action::interact(Fuel, 2, true)
	])]
	Log,
	
	
	#[assoc(name="sawblade")]
	#[assoc(description="Wooden round saw blade")]
	#[assoc(actions=vec![
		Action::interact(BuildSaw, 1, true)
	])]
	SawBlade,
	
	#[assoc(name="plank")]
	#[assoc(description="Wooden plank")]
	#[assoc(actions=vec![
		Action::Build(Structure::PlankWall, HashMap::new()),
		Action::interact(Fuel, 2, true)
	])]
	Plank,
}


#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn stone_has_smash_action() {
		assert_eq!(Item::Stone.actions(), vec![Action::interact(Smash, 1, true)]);
	}
	
}
