
use std::cmp::{min, max};


pub fn clamp<T: Ord>(val: T, lower: T, upper: T) -> T{
	max(min(val, upper), lower)
}

#[allow(dead_code)]
pub fn strip_prefix<'a>(txt: &'a str, prefix: &'a str) -> Option<&'a str> {
	if txt.starts_with(prefix) {
		Some(txt.split_at(prefix.len()).1)
	} else {
		None
	}
}

use std::fs;
use std::path::Path;
use crate::{
	errors::AnyError,
	aerr
};

pub fn randomize (mut seed: u32) -> u32 {
	seed ^= seed << 13;
	seed ^= seed >> 17;
	seed ^= seed << 5;
	seed
}

#[allow(dead_code)]
pub fn partition_by(s: &str, pat: &str) -> (String, String) {
	let mut parts: Vec<String> = s.splitn(2, pat).map(String::from).collect();
	while parts.len() < 2 {
		parts.push("".to_string())
	}
	(parts.remove(0), parts.remove(0))
}


#[allow(dead_code)]
pub fn write_file_safe<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> Result<(), AnyError> {
	let temppath = path
		.as_ref()
		.with_file_name(
			format!(
				"tempfile_{}_{}.tmp",
				path.as_ref().file_name().ok_or(aerr!("writing to directory"))?.to_str().unwrap_or("invalid"),
				rand::random::<u64>()
			)
		);
	fs::write(&temppath, contents)?;
	fs::rename(&temppath, path)?;
	Ok(())
}


#[macro_export]
macro_rules! hashmap {
	( $($key:expr => $value:expr ),* ) => {{
		#[allow(unused_mut)]
		let mut h = std::collections::HashMap::new();
		$(
			h.insert($key, $value);
		)*
		h
	}}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Percentage(pub i64);

pub struct Tuple2;

#[allow(dead_code)]
impl Tuple2 {
	pub fn first<T, U>((a, _): &(T, U)) -> &T {
		a
	}
	pub fn second<T, U>((_, b): &(T, U)) -> &U {
		b
	}
}



#[cfg(test)]
mod tests {
	use std::collections::HashMap;
	#[test]
	fn test_hashmap_macro() {
		let mut h = hashmap!("hello" => 1, "world" => 2);
		assert_eq!(h.remove("hello"), Some(1));
		assert_eq!(h.remove("world"), Some(2));
		assert!(h.is_empty());
		let h2: HashMap<i32, usize> = hashmap!();
		assert!(h2.is_empty());
		assert_eq!(h2, HashMap::new());
		
	}
}
