
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use enum_assoc::Assoc;
use crate::{
	sprite::Sprite,
	item::Item,
	action::{Action, InteractionType, CraftType, Interactable, InteractionResult},
	timestamp::Timestamp,
	hashmap,
	crop::Crop,
	creature::CreatureType as Npc,
};


#[derive(Debug, Clone, Copy, PartialEq, Eq, Assoc, Serialize, Deserialize)]
#[func(fn sprite(self) -> Option<Sprite>)]
#[func(fn accessible(self) -> bool {true})]
#[func(fn clear(self) -> Option<Ground>)]
#[func(fn describe(&self) -> Option<&str>)]
#[func(fn craft(self) -> Option<CraftType>)]
#[func(fn buildable(self) -> bool {false})]
#[func(pub fn restoring(self) -> bool {false})]
pub enum Ground {
	#[assoc(sprite = Sprite::Dirt)]
	#[assoc(describe = "Dirt")]
	#[assoc(buildable = true)]
	#[assoc(restoring = true)]
	Dirt,
	
	#[assoc(clear = Ground::Dirt)]
	#[assoc(sprite = Sprite::Grass1)]
	#[assoc(describe = "Grass")]
	Grass1,
	
	#[assoc(clear = Ground::Dirt)]
	#[assoc(sprite = Sprite::Grass2)]
	#[assoc(describe = "Grass")]
	Grass2,
	
	#[assoc(clear = Ground::Dirt)]
	#[assoc(sprite = Sprite::Grass3)]
	#[assoc(describe = "Grass")]
	Grass3,
	
	#[assoc(clear = Ground::Dirt)]
	#[assoc(sprite = Sprite::Moss)]
	#[assoc(describe = "Moss")]
	Moss,
	
	#[assoc(clear = Ground::Dirt)]
	#[assoc(sprite = Sprite::DeadLeaves)]
	#[assoc(describe = "Old leaves")]
	DeadLeaves,
	
	#[assoc(sprite = Sprite::Sanctuary)]
	#[assoc(describe = "Ornate stone floor")]
	Sanctuary,
	
	#[assoc(sprite = Sprite::Water)]
	#[assoc(has_water = true)]
	#[assoc(accessible = false)]
	#[assoc(describe = "Water")]
	#[assoc(craft = CraftType::Water)]
	Water,
	
	#[assoc(sprite = Sprite::RockFloor)]
	#[assoc(buildable = true)]
	#[assoc(describe = "Rock floor")]
	RockFloor,
	
	#[assoc(sprite = Sprite::StoneFloor)]
	#[assoc(buildable = true)]
	#[assoc(describe = "Stone floor")]
	StoneFloor,
	
	#[assoc(sprite = Sprite::WoodFloor)]
	#[assoc(describe = "Wooden plank floor")]
	WoodFloor,
	
	#[assoc(accessible = false)]
	Empty
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Assoc, Serialize, Deserialize)]
#[func(fn sprite(self) -> Option<Sprite>)]
#[func(fn blocking(self) -> bool {false})]
#[func(pub fn is_open(self) -> bool {false})]
#[func(fn interactions(self) -> Vec<Interactable> {Vec::new()})]
#[func(fn take(self) -> Option<Item>)]
#[func(fn describe(&self) -> Option<&str>)]
#[func(fn description(self) -> Option<String> { self.describe().map(|s| s.to_string())})]
#[func(fn craft(self) -> Option<CraftType>)]
#[func(fn grow(self) -> Option<(i64, Structure, Option<Structure>)>)]
#[func(fn join(self, other: Structure) -> Option<Structure>)]
#[func(fn spawn(self) -> Option<Npc>)]
pub enum Structure {
	#[assoc(is_open = true)]
	Air,
	
	#[assoc(sprite = Sprite::Wall)]
	#[assoc(blocking = true)]
	#[assoc(describe = "Stone wall")]
	Wall,
	
	#[assoc(sprite = Sprite::WoodWall)]
	#[assoc(blocking = true)]
	#[assoc(describe = "Wooden wall")]
	WoodWall,
	
	#[assoc(sprite = Sprite::Rock)]
	#[assoc(blocking = true)]
	#[assoc(describe = "Natural rock wall")]
	Rock,
	
	#[assoc(sprite = Sprite::RockMid)]
	#[assoc(blocking = true)]
	#[assoc(describe = "Natural rock wall")]
	RockMid,
	
	#[assoc(sprite = Sprite::Sapling)]
	#[assoc(describe = "Sapling")]
	Sapling,
	
	#[assoc(sprite = Sprite::YoungTree)]
	#[assoc(blocking = true)]
	#[assoc(describe = "Young tree")]
	#[assoc(interactions = vec![Interactable::harvest(InteractionType::Chop, 1, &[1.0], &[Item::Stick])])]
	YoungTree,
	
	#[assoc(sprite = Sprite::Tree)]
	#[assoc(blocking = true)]
	#[assoc(describe = "Tree")]
	#[assoc(interactions = vec![Interactable::harvest(InteractionType::Chop, 1, &[1.0], &[Item::Log])])]
	Tree,
	
	#[assoc(sprite = Sprite::OldTree)]
	#[assoc(blocking = true)]
	#[assoc(describe = "Dead tree")]
	#[assoc(interactions = vec![Interactable::harvest(InteractionType::Chop, 1, &[1.0], &[Item::Stick])])]
	OldTree,
	
	#[assoc(sprite = Sprite::OldTreeTinder)]
	#[assoc(blocking = true)]
	#[assoc(interactions = vec![Interactable::new(InteractionType::Cut, 1, &[0.5, 1.0], Some(Structure::OldTree), &[Item::Tinder])])]
	#[assoc(describe = "Dead tree with tinder fungus on it")]
	OldTreeTinder,
	
	#[assoc(sprite = Sprite::DenseGrass)]
	#[assoc(interactions = vec![Interactable::harvest(InteractionType::Take, 0, &[0.1], &[Item::GreenSeed])])]
	#[assoc(describe = "Dense grass")]
	DenseGrassGrn,
	
	#[assoc(sprite = Sprite::DenseGrass)]
	#[assoc(interactions = vec![Interactable::harvest(InteractionType::Take, 0, &[0.1], &[Item::BrownSeed])])]
	#[assoc(describe = "Dense grass")]
	DenseGrassBrn,
	
	#[assoc(sprite = Sprite::DenseGrass)]
	#[assoc(interactions = vec![Interactable::harvest(InteractionType::Take, 0, &[0.1], &[Item::YellowSeed])])]
	#[assoc(describe = "Dense grass")]
	DenseGrassY,
	
	#[assoc(sprite = Sprite::Heather)]
	#[assoc(describe = "Heather")]
	Heather,
	
	#[assoc(sprite = Sprite::Rush)]
	#[assoc(describe = "Rush")]
	Rush,
	
	#[assoc(sprite = Sprite::Shrub)]
	#[assoc(describe = "Some shrub")]
	Shrub,
	
	#[assoc(sprite = Sprite::Bush)]
	#[assoc(describe = "Just a bush")]
	Bush,
	
	#[assoc(sprite = Sprite::Reed)]
	#[assoc(interactions = vec![Interactable::harvest(InteractionType::Cut, 1, &[0.5, 1.0], &[Item::Reed])])]
	#[assoc(describe = "Reeds. Can be cut")]
	Reed,
	
	#[assoc(sprite = Sprite::PitcherPlant)]
	#[assoc(interactions = vec![Interactable::harvest(InteractionType::Cut, 1, &[0.5, 1.0], &[Item::Pitcher])])]
	#[assoc(describe = "Pitcher plant. Can be cut")]
	PitcherPlant,
	
	#[assoc(sprite = Sprite::Flower)]
	#[assoc(take = Item::Flower)]
	#[assoc(describe = "Flower")]
	Flower,
	
	#[assoc(sprite = Sprite::Stone)]
	#[assoc(take = Item::Stone)]
	#[assoc(interactions = vec![
		Interactable::new(
			InteractionType::Smash,
			1,
			&[0.4, 1.0],
			Some(Structure::Gravel),
			&[Item::SharpStone],
		)
	])]
	#[assoc(describe = "Stone. A medium-size cobble. Can be smashed to try to get smaller stones")]
	Stone,
	
	#[assoc(sprite = Sprite::Gravel)]
	#[assoc(describe = "Gravel. Small stone rocks")]
	Gravel,
	
	#[assoc(sprite = Sprite::Sage)]
	#[assoc(blocking = true)]
	#[assoc(describe = "Sage. An old wise person with grey hair. This sage is too busy meditating to talk")]
	Sage,
	
	#[assoc(sprite = Sprite::Fireplace)]
	#[assoc(blocking = true)]
	#[assoc(describe = "Fireplace. Safe place to have a fire")]
	#[assoc(interactions = vec![Interactable::transform(InteractionType::Fuel, 1, Structure::Fire)])]
	Fireplace,
	
	#[assoc(sprite = Sprite::Fire)]
	#[assoc(blocking = true)]
	#[assoc(describe = "Fire. Safely contained in fireplace.")]
	#[assoc(grow = (1, Structure::AshPlace, None))]
	Fire,
	
	#[assoc(sprite = Sprite::AshPlace)]
	#[assoc(blocking = true)]
	#[assoc(describe = "Fireplace. Filled with ash")]
	#[assoc(interactions = vec![Interactable::new(InteractionType::Take, 0, &[], Some(Structure::Fireplace), &[Item::Ash])])]
	AshPlace,
	
	#[assoc(sprite = Sprite::WorkTable)]
	#[assoc(blocking = true)]
	#[assoc(describe = "Gardening worktable. Build a crude hoe with a stick, some reed rope and a sharp stone")]
	#[assoc(craft = CraftType::GardeningTable)]
	GardeningTable,
	
	#[assoc(sprite = Sprite::Altar)]
	#[assoc(blocking = true)]
	#[assoc(describe = "Marker Altar. Bring 10 flowers and a stone to create a marker stone")]
	#[assoc(craft = CraftType::Marker)]
	MarkerAltar,
	
	#[assoc(sprite = Sprite::MarkStone)]
	#[assoc(blocking = true)]
	#[assoc(describe = "Mark stone. Center of a land claim")]
	MarkStone,
	
	#[assoc(sprite = Sprite::SeedingHardPlant)]
	#[assoc(describe = "Seeding Hardwood")]
	#[assoc(interactions = vec![Interactable::take(&[Item::BrownSeed, Item::BrownSeed])])]
	SeedingHardPlant,
	
	#[assoc(sprite = Sprite::SeedingDiscPlant)]
	#[assoc(describe = "Seeding Disc plant")]
	#[assoc(interactions = vec![Interactable::take(&[Item::GreenSeed, Item::GreenSeed])])]
	SeedingDiscPlant,
	
	#[assoc(sprite = Sprite::SeedingKnifePlant)]
	#[assoc(describe = "Seeding Knife plant")]
	#[assoc(interactions = vec![Interactable::take(&[Item::YellowSeed, Item::YellowSeed])])]
	SeedingKnifePlant,
	
	#[assoc(sprite = Sprite::DiscLeaf)]
	#[assoc(describe = "DiscLeaf")]
	#[assoc(interactions = vec![Interactable::take(&[Item::DiscLeaf])])]
	DiscLeaf,
	
	#[assoc(sprite = Sprite::KnifeLeaf)]
	#[assoc(describe = "KnifeLeaf")]
	#[assoc(interactions = vec![Interactable::take(&[Item::KnifeLeaf])])]
	KnifeLeaf,
	
	#[assoc(sprite = Sprite::HardwoodStick)]
	#[assoc(describe = "Hardwood stick")]
	#[assoc(interactions = vec![Interactable::take(&[Item::HardwoodStick])])]
	HardwoodStick,
	
	#[assoc(sprite = Sprite::HardwoodKnife)]
	#[assoc(describe = "Hardwood knife")]
	#[assoc(interactions = vec![Interactable::take(&[Item::HardwoodKnife])])]
	HardwoodKnife,
	
	#[assoc(sprite = Sprite::HardwoodTable)]
	#[assoc(describe = "Hardwood Table. Can be used for crafting")]
	#[assoc(interactions = vec![
		Interactable::take(&[Item::HardwoodTable]),
		Interactable::transform(InteractionType::BuildSaw, 1, Structure::SawTable)
	])]
	#[assoc(craft = CraftType::GardeningTable)]
	HardwoodTable,
	
	#[assoc(sprite = Sprite::SawTable)]
	#[assoc(describe = "Saw table. Can cut planks from logs")]
	#[assoc(craft = CraftType::SawTable)]
	SawTable,
	
	#[assoc(sprite = Sprite::SawBlade)]
	#[assoc(describe = "Saw blade")]
	#[assoc(interactions = vec![Interactable::take(&[Item::SawBlade])])]
	SawBlade,
	
	#[assoc(sprite = Sprite::WoodWall)]
	#[assoc(blocking = true)]
	#[assoc(describe = "Wooden wall")]
	#[assoc(interactions = vec![Interactable::harvest(InteractionType::Chop, 1, &[1.0], &[Item::Plank])])]
	PlankWall,
	
	#[assoc(sprite = _0.sprite())]
	#[assoc(description = _0.description())]
	#[assoc(interactions = _0.all_interactions())]
	#[assoc(grow = _0.grow()?)]
	#[assoc(join = _0.join(other)?)]
	Crop(Crop),

	#[assoc(spawn = _0)]
	Spawn(Npc),

	#[assoc(take = _0)]
	#[assoc(describe = _0.description())]
	#[assoc(sprite = _0.sprite()?)]
	Item(Item),
}


impl Structure {
	fn interactables(self) -> Vec<Interactable> {
		let mut interactions = self.interactions();
		if let Some(item) = self.take() {
			interactions.push(Interactable::take(&[item]));
		}
		interactions
	}
	
	pub fn joined(self, other: Structure) -> Option<Structure> {
		self.join(other).or_else(|| other.join(self))
	}
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tile {
	pub ground: Ground,
	pub structure: Structure
}

impl Tile {
	pub fn empty() -> Tile{
		Self{ground: Ground::Empty, structure: Structure::Air}
	}

	pub fn ground(ground: Ground) -> Tile{
		Self{ground, structure: Structure::Air}
	}
	
	pub fn structure(ground: Ground, structure: Structure) -> Tile {
		Self{ground, structure}
	}
	
	pub fn view(self) -> TileView {
		let sprites = [self.structure.sprite(), self.ground.sprite()]
			.into_iter()
			.flatten();
		TileView::new(sprites, self.blocking())
	}
	
	pub fn blocking(self) -> bool {
		!self.ground.accessible() || self.structure.blocking()
	}
	
	fn can_build(self) -> bool {
		self.structure.is_open() && self.ground.buildable()
	}

	pub fn inspect(self) -> String {
		let mut text = self.ground.describe().unwrap_or("").to_string();
		if let Some(s) = self.structure.description() {
			text = format!("{} | {}", text, s);
		}
		text
	}
	
	pub fn interact(self, item: Item, time: Timestamp) -> Option<InteractionResult> {
		item.actions().into_iter().find_map(|action| self.act(action, item, time))
	}
	
	pub fn act(self, action: Action, item: Item, time: Timestamp) -> Option<InteractionResult> {
		match action {
			Action::Interact(interact) => {
				let mut result = self.structure.interactables()
					.into_iter()
					.find_map(|interactable| interactable.apply(interact, time))?;
				if interact.use_item {
					result.cost.insert(item, 1);
				}
				Some(result)
			}
			Action::Clear =>
				if self.structure.is_open() {
					Some(InteractionResult {
						remains_ground: Some(self.ground.clear()?),
						..Default::default()
					})
				} else {
					None
				}
			Action::BuildClaim(structure) =>
				if self.can_build() {
					Some(InteractionResult {
						remains: Some(structure),
						cost: hashmap!{item => 1},
						claim: true,
						..Default::default()
					})
				} else {
					None
				}
			Action::Build(structure, mut cost) =>
				if self.can_build() {
					cost.entry(item).and_modify(|n| {*n += 1;}).or_insert(1);
					Some(InteractionResult {
						remains: Some(structure),
						cost,
						build: true,
						..Default::default()
					})
				} else {
					None
				}
			Action::Craft(typ, product, mut cost) => {
				cost.entry(item).and_modify(|n| {*n += 1;}).or_insert(1);
				if Some(typ) == self.structure.craft() || Some(typ) == self.ground.craft() {
					Some(InteractionResult {
						items: vec![product],
						cost,
						..Default::default()
					})
				} else {
					None
				}
			}
		}
	}
	
	pub fn grow(self) -> Option<(i64, Structure, Option<Structure>)> {
		self.structure.grow()
	}

	pub fn spawn(self) -> Option<Npc> {
		self.structure.spawn()
	}

	pub fn take(self) -> Option<(Tile, Item)> {
		Some((Self::ground(self.ground), self.structure.take()?))
	}
}

impl Default for Tile {
	fn default() -> Tile {
		Tile::ground(Ground::Empty)
	}
}

impl Serialize for Tile {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where S: Serializer {
		(self.ground, self.structure).serialize(serializer)
	}
}
impl<'de> Deserialize<'de> for Tile {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where D: Deserializer<'de> {
		let (ground, structure) = <(Ground, Structure)>::deserialize(deserializer)?;
		Ok(Self{ground, structure})
	}
}


#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct TileView(Vec<TileProperty>);

impl TileView {
	fn new(sprites: impl Iterator<Item=Sprite>, blocking: bool) -> Self {
		let mut properties: Vec<TileProperty> = sprites.map(TileProperty::Sprite).collect();
		if blocking {
			properties.push(TileProperty::Blocking);
		}
		Self(properties)
	}
}


#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq, Serialize)]
pub enum TileProperty {
	#[serde(rename="!b")]
	Blocking,
	#[serde(untagged)]
	Sprite(Sprite),
}
