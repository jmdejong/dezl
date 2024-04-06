


use serde::{Serialize, Deserialize};
use enum_assoc::Assoc;
use crate::{
	sprite::Sprite,
	action::{InteractionType, Interactable},
	tile::Structure,
};


#[derive(Debug, Clone, Copy, PartialEq, Eq, Assoc, Serialize, Deserialize)]
#[func(fn sprite(self) -> Sprite)]
#[func(fn describe(&self) -> &str)]
#[func(fn interactions(self) -> Vec<Interactable> {Vec::new()})]
#[func(fn next(self) -> Option<(i64, CropType)>)]
#[func(fn grow(self) -> Option<(i64, Structure)>)]
#[func(fn fertilized_grow(self) -> Option<CropType>)]
#[func(fn inosculate(self) -> Vec<(CropType, CropType)> {Vec::new()})]
enum CropType {
	
	#[assoc(sprite = Sprite::PlantedSeed)]
	#[assoc(describe = "Planted seed")]
	#[assoc(next = (1, CropType::GreenSeedling))]
	GreenSeed,
	
	#[assoc(sprite = Sprite::Seedling)]
	#[assoc(describe = "Seedling")]
	#[assoc(next = (1, CropType::YoungDiscPlant))]
	GreenSeedling,
	
	#[assoc(sprite = Sprite::YoungDiscPlant)]
	#[assoc(describe = "A small plant with big round leaves")]
	#[assoc(next = (1, CropType::DiscPlant))]
	#[assoc(fertilized_grow = CropType::DiscShoot)]
	YoungDiscPlant,
	
	#[assoc(sprite = Sprite::DiscPlant)]
	#[assoc(describe = "A plant with big round leaves")]
	#[assoc(grow = (1, Structure::SeedingDiscPlant))]
	#[assoc(fertilized_grow = CropType::DiscShoot)]
	DiscPlant,
	
	#[assoc(sprite = Sprite::DiscShoot)]
	#[assoc(describe = "A shoot of a plant with big round leaves")]
	#[assoc(grow = (1, Structure::DiscLeaf))]
	#[assoc(inosculate = vec![
		(CropType::HardShoot, CropType::HardDiscPlant),
		(CropType::KnifeShoot, CropType::DiscKnifePlant)
	])]
	DiscShoot,
	
	
	#[assoc(sprite = Sprite::PlantedSeed)]
	#[assoc(describe = "Planted seed")]
	#[assoc(next = (1, CropType::YellowSeedling))]
	YellowSeed,
	
	#[assoc(sprite = Sprite::Seedling)]
	#[assoc(describe = "Seedling")]
	#[assoc(next = (1, CropType::YoungKnifePlant))]
	YellowSeedling,
	
	#[assoc(sprite = Sprite::YoungKnifePlant)]
	#[assoc(describe = "A small plant with sharp leaves")]
	#[assoc(next = (1, CropType::KnifePlant))]
	#[assoc(fertilized_grow = CropType::KnifeShoot)]
	YoungKnifePlant,
	
	#[assoc(sprite = Sprite::KnifePlant)]
	#[assoc(describe = "A plant with sharp leaves")]
	#[assoc(grow = (1, Structure::SeedingKnifePlant))]
	#[assoc(fertilized_grow = CropType::KnifeShoot)]
	KnifePlant,
	
	#[assoc(sprite = Sprite::KnifeShoot)]
	#[assoc(describe = "A shoot of a plant with sharp leaves")]
	#[assoc(grow = (1, Structure::KnifeLeaf))]
	#[assoc(inosculate = vec![
		(CropType::DiscShoot, CropType::DiscKnifePlant),
		(CropType::HardShoot, CropType::HardKnifePlant)
	])]
	KnifeShoot,
	
	
	#[assoc(sprite = Sprite::PlantedSeed)]
	#[assoc(describe = "Planted seed")]
	#[assoc(next = (1, CropType::BrownSeedling))]
	BrownSeed,
	
	#[assoc(sprite = Sprite::Seedling)]
	#[assoc(describe = "Seedling")]
	#[assoc(next = (1, CropType::YoungHardPlant))]
	BrownSeedling,
	
	#[assoc(sprite = Sprite::YoungHardPlant)]
	#[assoc(describe = "A small plant with a hard stem")]
	#[assoc(next = (1, CropType::HardPlant))]
	#[assoc(fertilized_grow = CropType::HardShoot)]
	YoungHardPlant,
	
	#[assoc(sprite = Sprite::HardPlant)]
	#[assoc(describe = "Plant with a very hard stem")]
	#[assoc(grow = (1, Structure::SeedingHardPlant))]
	#[assoc(fertilized_grow = CropType::HardShoot)]
	HardPlant,
	
	#[assoc(sprite = Sprite::HardShoot)]
	#[assoc(describe = "A shoot of a plant with hard branches")]
	#[assoc(grow = (1, Structure::HardwoodStick))]
	#[assoc(inosculate = vec![
		(CropType::DiscShoot, CropType::HardDiscPlant),
		(CropType::KnifeShoot, CropType::HardKnifePlant)
	])]
	HardShoot,
	
	#[assoc(sprite = Sprite::HardKnifePlant)]
	#[assoc(describe = "A shoot of a hardwood plant inosculated with a shoot of a knife plant")]
	#[assoc(grow = (1, Structure::HardwoodKnife))]
	HardKnifePlant,
	
	#[assoc(sprite = Sprite::HardDiscPlant)]
	#[assoc(describe = "A shoot of a hardwood plant inosculated with a shoot of a disc plant")]
	#[assoc(grow = (1, Structure::HardwoodTable))]
	HardDiscPlant,
	
	#[assoc(sprite = Sprite::DiscKnifePlant)]
	#[assoc(describe = "A shoot of a knife plant inosculated with a shoot of a disc plant")]
	#[assoc(grow = (1, Structure::SawBlade))]
	DiscKnifePlant,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Crop {
	typ: CropType,
	flags: u8
}

const WATERED: u8 = 1<<7;
const FERTILIZED: u8 = 1<<6;

impl Crop {
	pub fn all_interactions(self) -> Vec<Interactable> {
		let mut interactions = self.typ.interactions();
		if self.flags & WATERED == 0 {
			interactions.push(Interactable::transform(InteractionType::Water, 1, Structure::Crop(self.water())));
		}
		if self.flags & FERTILIZED == 0 && self.typ.fertilized_grow().is_some() {
			interactions.push(Interactable::transform(InteractionType::Fertilize, 1, Structure::Crop(self.fertilize())));
		}
		interactions
	}
	
	fn water(self) -> Self {
		Self { typ: self.typ, flags: self.flags | WATERED }
	}
	
	fn fertilize(self) -> Self {
		Self { typ: self.typ, flags: self.flags | FERTILIZED }
	}
	
	pub fn description(self) -> String {
		let mut description = self.typ.describe().to_string();
		if self.flags & WATERED == 0 {
			description = format!("{}. Needs water", description);
		}
		if self.flags & FERTILIZED == 0 && self.typ.fertilized_grow().is_some() {
			description = format!("{}. Can be fertilized", description);
		}
		description
	}
	
	pub fn grow(self) -> Option<(i64, Structure, Option<Structure>)> {
		if self.flags & WATERED == 0 {
			return None;
		}
		let shoot =
			if self.flags & FERTILIZED != 0 {
				self.typ.fertilized_grow().map(|typ| Structure::Crop(Self { typ, flags: 0 }))
			} else {
				None
			};
		if let Some((steps, typ)) = self.typ.next() {
			let crop = Self { typ, flags: 0 };
			Some((steps, Structure::Crop(crop), shoot))
		} else if let Some((steps, typ)) = self.typ.grow() {
			Some((steps, typ, shoot))
		} else {
			None
		}
	}
	
	pub fn join(self, other: Structure) -> Option<Structure> {
		if let Structure::Crop(crop) = other {
			for (with, product) in self.typ.inosculate() {
				if with == crop.typ {
					return Some(Structure::Crop(Self{ typ: product, flags: 0 }))
				}
			}
		}
		None
	}
	
	pub fn sprite(self) -> Sprite {
		self.typ.sprite()
	}
	
	fn new(typ: CropType) -> Self {
		Self { typ, flags: 0 }
	}
	
	pub fn greenseed() -> Self {
		Self::new(CropType::GreenSeed)
	}
	
	pub fn yellowseed() -> Self {
		Self::new(CropType::YellowSeed)
	}
	
	pub fn brownseed() -> Self {
		Self::new(CropType::BrownSeed)
	}
	
	
}

