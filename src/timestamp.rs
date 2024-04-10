
use std::ops::{Add, Sub};
use serde::{Serialize, Deserialize};
use crate::random;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Timestamp(pub i64);

impl Timestamp {
	pub fn zero() -> Self {
		Self(0)
	}

	pub fn increment(&mut self) {
		self.0 += 1;
	}
	
	pub fn random_seed(self) -> u32 {
		random::randomize_u32(self.0 as u32 ^ 12345)
	}
}

impl Add<Duration> for Timestamp {
	type Output = Self;
	fn add(self, other: Duration) -> Self {
		Self(self.0 + other.0)
	}
}

impl Sub<Duration> for Timestamp {
	type Output = Self;
	fn sub(self, other: Duration) -> Self {
		Self(self.0 - other.0)
	}
}

impl Sub<Self> for Timestamp {
	type Output = Duration;
	fn sub(self, other: Self) -> Duration {
		Duration(self.0 - other.0)
	}
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Duration(pub i64);

impl Sub<Self> for Duration {
	type Output = Self;
	fn sub(self, other: Self) -> Self {
		Duration(self.0 - other.0)
	}
}
impl Add<Self> for Duration {
	type Output = Self;
	fn add(self, other: Self) -> Self {
		Duration(self.0 + other.0)
	}
}
