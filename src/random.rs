
use crate::Pos;

const MULTIPLIER: u64 = 0x5DEECE66D;
const ADDEND: u64 = 0xB;
const MASK: u64 = (1 << 48) - 1;


pub fn randomize_u32(seed: u32) -> u32 {
	let num: u64 = (seed as u64 ^ MULTIPLIER) & MASK;
	(((num.wrapping_mul(MULTIPLIER).wrapping_add(ADDEND)) & MASK) >> 16) as u32
}

pub fn random_float(seed: u32) -> f32 {
	(randomize_u32(seed) & 0xffff) as f32 / (0x10000 as f32)
}

#[inline]
pub fn randomize_pos(pos: Pos) -> u32 {
	randomize_u32(pos.x as u32 ^ randomize_u32(pos.y as u32))
}

pub fn randomize_str(s: &str) -> u32 {
	let mut seed: u32 = 3829;
	for byte in s.bytes() {
		seed *= 31;
		seed += byte as u32;
	}
	seed
}

pub fn pick<T>(seed: u32, choices: &[T]) -> &T {
	&choices[seed as usize % choices.len()]
}


pub fn pick_weighted<T>(seed: u32, choices: &[(T, u32)]) -> &T {
	let total: u32 = choices.iter().map(|(_v, c)| c).sum();
	let mut rind = (seed % total) as i32;
	for (value, chance) in choices {
		rind -= *chance as i32;
		if rind < 0 {
			return value;
		}
	}
	panic!("weighted picking exceeds index");
}




#[derive(Debug, Clone)]
pub struct WhiteNoise {
	seed: u32
}

impl WhiteNoise {
	pub fn new(seed: u32) -> Self {
		Self{seed: randomize_u32(seed)}
	}
	
	pub fn gen(&self, pos: Pos) -> u32 {
		randomize_u32(self.seed ^ randomize_pos(pos))
	}
	
	pub fn gen_f(&self, pos: Pos) -> f32 {
		random_float(self.gen(pos))
	}
}


#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn pick_weighted_can_pick_last() {
		for i in 0..15 {
			let v = if i%6 < 5 { 10 } else { 20 };
			assert_eq!(*pick_weighted(i, &[(10, 5), (20, 1)]), v);
		}
	}
}
