
use std::path::{PathBuf};
use std::fs;
use std::env;
use std::io::ErrorKind;
use serde_json;
use crate::{
	PlayerId,
	world::WorldSave,
	player::PlayerSave,
	errors::AnyError,
	util::write_file_safe
};

#[derive(Debug)]
pub enum LoaderError {
	MissingResource(AnyError),
	InvalidResource(AnyError)
}

macro_rules! inv {
	($code:expr) => {($code).map_err(|err| LoaderError::InvalidResource(Box::new(err)))}
}


pub trait PersistentStorage {
	
	fn load_world(&self) -> Result<WorldSave, LoaderError>;
	fn load_player(&self, id: PlayerId) -> Result<PlayerSave, LoaderError>;
	
	fn save_world(&self, state: WorldSave) -> Result<(), AnyError>;
	fn save_player(&self, id: PlayerId, state: PlayerSave) -> Result<(), AnyError>;
}


pub struct FileStorage {
	directory: PathBuf
}

impl FileStorage {
	pub fn new(path: PathBuf) -> Self {
		Self {
			directory: path
		}
	}
	
	pub fn default_save_dir(world_name: String) -> Option<PathBuf> {
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
	
	fn load_player(&self, id: PlayerId) -> Result<PlayerSave, LoaderError> {
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
	
	fn save_player(&self, id: PlayerId, state: PlayerSave) -> Result<(), AnyError> {
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

