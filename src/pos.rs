

use std::ops::{Add, Sub, Neg, Mul, Div, Rem, AddAssign};
use serde::{Serialize, Serializer, Deserialize, Deserializer};


#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all="lowercase")]
pub enum Direction {
	North,
	South,
	East,
	West
}

impl Direction {
	
	pub fn to_position(self) -> Pos {
		match self {
			Direction::North => Pos::new(0, -1),
			Direction::South => Pos::new(0, 1),
			Direction::East => Pos::new(1, 0),
			Direction::West => Pos::new(-1, 0)
		}
	}
	
	#[allow(dead_code)]
	pub const DIRECTIONS: [Direction; 4] = [Direction::North, Direction::South, Direction::East, Direction::West];
}


#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Default)]
pub struct Pos {
	pub x: i32,
	pub y: i32
}


impl Pos {
	
	pub const fn new(x: i32, y: i32) -> Pos {
		Pos {x, y}
	}
	
	pub fn zero() -> Pos {
		Pos {x: 0, y: 0}
	}
	
	#[allow(dead_code)]
	pub fn from_tuple(p: (i32, i32)) -> Pos {
		let (x, y) = p;
		Pos {x, y}
	}
	
	pub fn abs(&self) -> Pos {
		Pos{x: self.x.abs(), y: self.y.abs()}
	}

	#[allow(dead_code)]
	pub fn max(&self) -> i32 {
		if self.x > self.y {
			self.x
		} else {
			self.y
		}
	}
	#[allow(dead_code)]
	pub fn min(&self) -> i32 {
		if self.x < self.y {
			self.x
		} else {
			self.y
		}
	}
	
	pub fn size(&self) -> i32{
		self.x.abs() + self.y.abs()
	}
	
	#[allow(dead_code)]
	pub fn is_zero(&self) -> bool {
		self.x == 0 && self.y == 0
	}
	
	pub fn distance_to(&self, other: Pos) -> i32 {
		(other - *self).size()
	}
	
	#[allow(dead_code)]
	pub fn directions_to(&self, other: Pos) -> Vec<Direction> {
		let mut directions = Vec::new();
		let d = other - *self;
		if d.x > 0 {
			directions.push(Direction::East);
		}
		if d.x < 0 {
			directions.push(Direction::West);
		}
		if d.y > 0 {
			directions.push(Direction::South);
		}
		if d.y < 0 {
			directions.push(Direction::North);
		}
		directions
	}
}


impl Serialize for Pos {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where S: Serializer {
		(self.x, self.y).serialize(serializer)
	}
}
impl<'de> Deserialize<'de> for Pos {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where D: Deserializer<'de> {
		let (x, y) = <(i32, i32)>::deserialize(deserializer)?;
		Ok(Self{x, y})
	}
}


impl Add<Pos> for Pos {
	type Output = Pos;
	fn add(self, other: Pos) -> Pos {
		Pos {
			x: self.x + other.x,
			y: self.y + other.y
		}
	}
}

impl Add<(i32, i32)> for Pos {
	type Output = Pos;
	fn add(self, other: (i32, i32)) -> Pos {
		Pos {
			x: self.x + other.0,
			y: self.y + other.1
		}
	}
}

impl Add<Direction> for Pos {
	type Output = Pos;
	fn add(self, dir: Direction) -> Pos {
		let other = dir.to_position();
		Pos {
			x: self.x + other.x,
			y: self.y + other.y
		}
	}
}

impl Add<Option<Direction>> for Pos {
	type Output = Pos;
	fn add(self, dir: Option<Direction>) -> Pos {
		let other = dir.map_or_else(Pos::zero, Direction::to_position);
		Pos {
			x: self.x + other.x,
			y: self.y + other.y
		}
	}
}

impl Sub<Pos> for Pos {
	type Output = Pos;
	fn sub(self, other: Pos) -> Pos {
		Pos {
			x: self.x - other.x,
			y: self.y - other.y
		}
	}
}

impl Neg for Pos {
    type Output = Pos;
    fn neg(self) -> Pos {
		Pos {x: -self.x, y: -self.y}
    }
}

impl Mul<i32> for Pos {
	type Output = Pos;
	fn mul(self, n: i32) -> Pos {
		Pos {
			x: self.x * n,
			y: self.y * n
		}
	}
}

impl Div<i32> for Pos {
	type Output = Pos;
	fn div(self, n: i32) -> Pos {
		Pos {
			x: self.x.div_euclid(n),
			y: self.y.div_euclid(n)
		}
	}
}


impl Rem<i32> for Pos {
	type Output = Pos;
	fn rem(self, n: i32) -> Pos {
		Pos {
			x: self.x.rem_euclid(n),
			y: self.y.rem_euclid(n)
		}
	}
}

impl AddAssign for Pos {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            x: self.x + other.x,
            y: self.y + other.y,
        };
    }
}


#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Default)]
pub struct Area {
	min: Pos,
	size: Pos
}

impl Area {
	
	pub fn new(min: Pos, size: Pos) -> Self {
		Self {min, size}
	}
	
	pub fn centered(center: Pos, size: Pos) -> Self {
		Self::new(center - size / 2, size)
	}

	pub fn between(min: Pos, max: Pos) -> Self {
		Self::new(min, max - min)
	}
	
	pub fn min(&self) -> Pos {
		self.min
	}
	
	pub fn size(&self) -> Pos {
		self.size
	}

	pub fn surface(&self) -> i32 {
		self.size.x * self.size.y
	}
	
	pub fn max(&self) -> Pos {
		self.min + self.size
	}
	
	pub fn iter(&self) -> AreaIter {
		AreaIter {
			area: *self,
			x: self.min.x,
			y: self.min.y
		}
	}
	
	pub fn random_pos(&self, rind: u32) -> Pos {
		let seed = rind as i32;
		let x = seed % self.size.x;
		let y = (seed / self.size.x) % self.size.y;
		Pos::new(x, y) + self.min
	}
	
	pub fn grow(&self, n: i32) -> Area {
		let nn = Pos::new(n, n);
		Area::new(self.min - nn, self.size + nn * 2)
	}
	
	pub fn contains(&self, pos: Pos) -> bool {
		pos.x >= self.min().x
			&& pos.x < self.max().x
			&& pos.y >= self.min().y
			&& pos.y < self.max().y
	}

	pub fn overlaps(&self, other: &Area) -> bool {
		! (self.min().x >= other.max().x || self.max().x <= other.min().x || self.min().y >= other.max().y || self.max().y <= other.min().y)
	}
}


impl Serialize for Area {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where S: Serializer {
		AreaSave {
			x: self.min.x,
			y: self.min.y,
			w: self.size.x,
			h: self.size.y
		}.serialize(serializer)
	}
}
impl<'de> Deserialize<'de> for Area {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where D: Deserializer<'de> {
		let s = AreaSave::deserialize(deserializer)?;
		Ok(Self::new(Pos::new(s.x, s.y), Pos::new(s.w, s.h)))
	}
}

pub struct AreaIter{
	area: Area,
	x: i32,
	y: i32
}

impl Iterator for AreaIter {
	type Item = Pos;
	fn next(&mut self) -> Option<Self::Item> {
		if self.x >= self.area.max().x {
			self.x = self.area.min().x;
			if self.x >= self.area.max().x {
				return None;
			}
			self.y += 1;
		}
		if self.y >= self.area.max().y {
			None
		} else {
			self.x += 1;
			Some(Pos::new(self.x-1, self.y))
		}
	}
}

#[derive(Serialize, Deserialize)]
pub struct AreaSave {
	x: i32,
	y: i32,
	w: i32,
	h: i32
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::collections::HashSet;
	#[test]
	fn division_rounds_to_negative_infinity() {
		assert_eq!(Pos::new(-3, -3) / 2, Pos::new(-2, -2));
	}
	#[test]
	fn creates_centered_area(){
		let area = Area::centered(Pos::new(10, 20), Pos::new(4, 6));
		assert_eq!(area, Area::new(Pos::new(8, 17), Pos::new(4, 6)));
	}
	#[test]
	fn iterates_over_zero_width_area(){
		let area = Area::new(Pos::new(10, 10), Pos::new(0, 10));
		assert_eq!(area.iter().next(), None);
	}
	#[test]
	fn iterates_over_zero_height_area(){
		let area = Area::new(Pos::new(10, 10), Pos::new(10, 0));
		assert_eq!(area.iter().next(), None);
	}
	#[test]
	fn iterates_over_area(){
		let area = Area::new(Pos::new(8, 10), Pos::new(4, 1));
		let mut set = HashSet::new();
		set.insert(Pos::new(8, 10));
		set.insert(Pos::new(9, 10));
		set.insert(Pos::new(10, 10));
		set.insert(Pos::new(11, 10));
		assert_eq!(area.iter().collect::<HashSet<Pos>>(), set);
	}
	#[test]
	fn iterates_over_centered_area(){
		let area = Area::centered(Pos::new(10, 10), Pos::new(4, 1));
		let mut set = HashSet::new();
		set.insert(Pos::new(8, 10));
		set.insert(Pos::new(9, 10));
		set.insert(Pos::new(10, 10));
		set.insert(Pos::new(11, 10));
		assert_eq!(area.iter().collect::<HashSet<Pos>>(), set);
	}
}
