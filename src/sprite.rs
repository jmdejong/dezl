

use serde::{Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq, Serialize)]
#[serde(rename_all="lowercase")]
#[allow(dead_code)]
pub enum Sprite {
	#[serde(rename="player")]
	PlayerDefault,
	Sage,
	Dirt,
	Grass1,
	Grass2,
	Grass3,
	RockFloor,
	StoneFloor,
	WoodFloor,
	Gravel,
	DenseGrass,
	Heather,
	Shrub,
	Bush,
	Sanctuary,
	Water,
	Wall,
	WoodWall,
	Rock,
	RockMid,
	Sapling,
	YoungTree,
	Tree,
	OldTree,
	OldTreeTinder,
	Stone,
	Pebble,
	Crop,
	Flower,
	Reed,
	Rush,
	Lilypad,
	Moss,
	DeadLeaves,
	PitcherPlant,
	Fireplace,
	Fire,
	AshPlace,
	MarkStone,
	Altar,
	WorkTable,
	GreenStem,
	BrownStem,
	Stick,

	PlantedSeed,
	Seedling,

	YoungDiscPlant,
	DiscPlant,
	SeedingDiscPlant,
	DiscShoot,
	DiscLeaf,

	YoungKnifePlant,
	KnifePlant,
	SeedingKnifePlant,
	KnifeShoot,
	KnifeLeaf,

	YoungHardPlant,
	HardPlant,
	SeedingHardPlant,
	HardShoot,
	HardwoodStick,

	HardKnifePlant,
	HardDiscPlant,
	DiscKnifePlant,

	HardwoodKnife,
	HardwoodTable,
	SawBlade,

	SawTable,

	Frog,
	Worm,
	Unknown,
}
