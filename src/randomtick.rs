
use crate::{
	pos::Pos,
	timestamp::Timestamp
};

pub const CHUNK_SIZE: i32 = 16;
const STEP: i64 = 541;
const STEP_INVERSE: i64 = 53;
pub const CHUNK_AREA: i64 = (CHUNK_SIZE * CHUNK_SIZE) as i64;
const XOR: i64 = 0b1101110110_i64.rem_euclid(CHUNK_AREA);

pub fn tick_position(time: Timestamp) -> Pos {
	let ind = ((time.0^XOR) * STEP).rem_euclid(CHUNK_AREA) as i32;
	Pos::new(ind % CHUNK_SIZE, ind / CHUNK_SIZE)
}

fn tick_time(pos: Pos) -> i64 {
	((pos.x.rem_euclid(CHUNK_SIZE) + pos.y.rem_euclid(CHUNK_SIZE) * CHUNK_SIZE) as i64 * STEP_INVERSE).rem_euclid(CHUNK_AREA) ^ XOR
}

pub fn tick_num(pos: Pos, time: Timestamp) -> i64 {
	time.0.div_euclid(CHUNK_AREA)
		+ i64::from(tick_time(pos) <= time.0.rem_euclid(CHUNK_AREA))
}

#[cfg(test)]
mod tests {
	use super::*;
	
	#[test]
	fn step_inverse_is_inverse_of_step() {
		assert_eq!(STEP * STEP_INVERSE % CHUNK_AREA, 1);
	}
	
	#[test]
	fn tick_time_reverses_tick_position() {
		for i in 0..5000.min(CHUNK_AREA as i64) {
			assert_eq!(tick_time(tick_position(Timestamp(i))) as i64, i);
		}
	}
	
	#[test]
	fn same_positions_in_different_chunks_give_same_tick_time() {
		let pos = Pos::new(3, 4);
		let larger_pos = pos + Pos::new(CHUNK_SIZE * 3, CHUNK_SIZE * 7);
		let neg_pos = pos - Pos::new(CHUNK_SIZE * 6, CHUNK_SIZE * 2);
		let time = tick_time(pos);
		assert_eq!(time, tick_time(larger_pos), "larger pos gives different tick time");
		assert_eq!(time, tick_time(neg_pos), "negative pos gives different tick time");
	}
	
	#[test]
	fn times_with_multiples_of_chunk_area_in_between_give_save_tick_pos() {
		let time = Timestamp(5);
		let larger_time = Timestamp(5+CHUNK_AREA * 9);
		let neg_time = Timestamp(5 - CHUNK_AREA *11);
		let pos = tick_position(time);
		assert_eq!(pos, tick_position(larger_time));
		assert_eq!(pos, tick_position(neg_time));
	}
	
	#[test]
	fn tick_num_updates_on_tick_time(){
		let time = Timestamp(12300);
		let pos = tick_position(time);
		let tick = tick_num(pos, time);
		assert_eq!(
			tick,
			tick_num(pos, Timestamp(time.0 - 1 + CHUNK_AREA as i64))
		);
		assert_eq!(
			tick - 1,
			tick_num(pos, Timestamp(time.0 - 1))
		);
		assert_eq!(
			tick - 1,
			tick_num(pos, Timestamp(time.0 - CHUNK_AREA as i64))
		);
		assert_eq!(
			tick + 1,
			tick_num(pos, Timestamp(time.0 + CHUNK_AREA as i64))
		);
		assert_eq!(
			tick - 2,
			tick_num(pos, Timestamp(time.0 - 1 - CHUNK_AREA as i64))
		);
	}
}
