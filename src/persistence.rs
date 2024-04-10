
use crate::{
	player::PlayerId,
	world::WorldSave,
	creature::PlayerSave,
	errors::AnyError,
};

#[derive(Debug)]
pub enum LoaderError {
	MissingResource(AnyError),
	InvalidResource(AnyError)
}


#[derive(Debug)]
pub enum InitializeError {
	NoDataHome
}

macro_rules! inv {
	($code:expr) => {($code).map_err(|err| LoaderError::InvalidResource(Box::new(err)))}
}


pub trait PersistentStorage {

	fn initialize(world_name: &str) -> Result<Self, InitializeError> where Self: Sized;
	
	fn load_world(&self) -> Result<WorldSave, LoaderError>;
	fn load_player(&self, id: &PlayerId) -> Result<PlayerSave, LoaderError>;
	
	fn save_world(&self, state: WorldSave) -> Result<(), AnyError>;
	fn save_player(&self, id: &PlayerId, state: PlayerSave) -> Result<(), AnyError>;
}


pub mod file {

	use super::*;

	use std::path::{Path, PathBuf};
	use std::fs;
	use std::env;
	use std::io::ErrorKind;

	use crate::{
		aerr,
	};


	pub struct FileStorage {
		directory: PathBuf
	}

	impl FileStorage {


		fn default_save_dir(world_name: &str) -> Option<PathBuf> {
			if let Some(pathname) = env::var_os("XDG_DATA_HOME") {
				let mut path = PathBuf::from(pathname);
				path.push("dezl");
				path.push("saves");
				path.push(world_name);
				Some(path)
			} else if let Some(pathname) = env::var_os("HOME") {
				let mut path = PathBuf::from(pathname);
				path.push(".dezl");
				path.push("saves");
				path.push(world_name);
				Some(path)
			} else {
				None
			}
		}
	}

	impl PersistentStorage for FileStorage {

		fn initialize(world_name: &str) -> Result<Self, InitializeError> {
			let path = Self::default_save_dir(world_name).ok_or(InitializeError::NoDataHome)?;
			Ok(Self {
				directory: path
			})
		}

		fn load_world(&self) -> Result<WorldSave, LoaderError> {
			let mut path = self.directory.clone();
			path.push("world.save.json");
			let text = fs::read_to_string(path).map_err(|err| {
				if err.kind() == ErrorKind::NotFound {
					LoaderError::MissingResource(Box::new(err))
				} else {
					LoaderError::InvalidResource(Box::new(err))
				}
			})?;
			let state = inv!(serde_json::from_str(&text))?;
			Ok(state)
		}

		fn load_player(&self, id: &PlayerId) -> Result<PlayerSave, LoaderError> {
			let mut path = self.directory.clone();
			path.push("players");
			let fname = id.to_string() + ".save.json";
			path.push(fname);
			let text = fs::read_to_string(path).map_err(|err| {
				if err.kind() == ErrorKind::NotFound {
					LoaderError::MissingResource(Box::new(err))
				} else {
					LoaderError::InvalidResource(Box::new(err))
				}
			})?;
			let state = inv!(serde_json::from_str(&text))?;
			Ok(state)
		}


		fn save_world(&self, state: WorldSave) -> Result<(), AnyError> {
			let mut path = self.directory.clone();
			fs::create_dir_all(&path)?;
			path.push("world.save.json");
			let text = serde_json::to_string(&state).unwrap();
			write_file_safe(path, text)?;
			Ok(())
		}

		fn save_player(&self, id: &PlayerId, state: PlayerSave) -> Result<(), AnyError> {
			let mut path = self.directory.clone();
			path.push("players");
			fs::create_dir_all(&path)?;
			let fname = id.to_string() + ".save.json";
			path.push(fname);
			let text = serde_json::to_string(&state).unwrap();
			write_file_safe(path, text)?;
			Ok(())
		}
	}

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

}

