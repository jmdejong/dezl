
mod holder;

pub use holder::{Holder, HolderId};

use std::cmp::{min, max};

pub fn clamp<T: Ord>(val: T, lower: T, upper: T) -> T{
	max(min(val, upper), lower)
}

use std::fs;
use std::path::Path;
use crate::{
	errors::AnyError,
	aerr
};

pub fn write_file_safe<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> Result<(), AnyError> {
	let temppath = path
		.as_ref()
		.with_file_name(
			format!(
				"tempfile_{}_{}.tmp",
				path.as_ref().file_name().ok_or_else(|| aerr!("writing to directory"))?.to_str().unwrap_or("invalid"),
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
