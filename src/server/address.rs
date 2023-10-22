
use std::path::PathBuf;
use std::net::{SocketAddr};
use std::str::FromStr;

use crate::{
	Result,
	err,
	errors::AError
};
use super::{
	VarInetServer,
	UnixServer,
	ServerEnum
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Address {
	Inet(SocketAddr),
	Unix(PathBuf)
}

impl Address {
	pub fn to_server(&self) -> Result<ServerEnum> {
		match self {
			Address::Inet(addr) => Ok(VarInetServer::new(*addr)?.into()),
			Address::Unix(path) => Ok(UnixServer::new(path)?.into())
		}
	}
}



impl FromStr for Address {
	type Err = AError;
	fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
		let parts: Vec<&str> = s.splitn(2, ':').collect();
		if parts.len() != 2 {
			return Err(err!("Address string should consist of 2 parts separated by the first colon, but consists of {:?}", parts));
		}
		let typename = parts[0];
		let text = parts[1];
		match typename {
			"inet" => Ok(Address::Inet(text.parse().map_err(|e| err!("'{}' is not a valid inet address: {}", text, e))?)),
			"unix" => Ok(Address::Unix(PathBuf::new().join(text))),
			"abstract" => {
					if cfg!(target_os = "linux") {
						Ok(Address::Unix(PathBuf::new().join(format!("\0{}", text))))
					} else {
						Err(err!("abstract adresses are only for linux"))
					}
				}
			_ => Err(err!("'{}' is not a valid address type", typename))
		}
	}
}
