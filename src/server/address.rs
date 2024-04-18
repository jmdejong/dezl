
use std::path::PathBuf;
use std::net::{SocketAddr};
use std::str::FromStr;

use crate::{
	errors::{AError, AnyError},
	err,
};
use super::{
	VarInetServer,
	WebTlsServer,
	StreamTlsServer,
	UnixServer,
	ServerEnum
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Address {
	Inet(SocketAddr),
	TlsWeb(SocketAddr),
	TlsSock(SocketAddr),
	Unix(PathBuf)
}


impl Address {
	pub fn to_server(&self, identity: Option<native_tls::Identity>) -> Result<ServerEnum, AnyError> {
		match self {
			Address::Inet(addr) => Ok(VarInetServer::new(*addr)?.into()),
			Address::TlsWeb(addr) => Ok(WebTlsServer::new(*addr, identity.ok_or(err!("Missing tls identity"))?)?.into()),
			Address::TlsSock(addr) => Ok(StreamTlsServer::new(*addr, identity.ok_or(err!("Missing tls identity"))?)?.into()),
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
			"tlsweb" => Ok(Address::TlsWeb(text.parse().map_err(|e| err!("'{}' is not a valid inet address: {}", text, e))?)),
			"tlssock" => Ok(Address::TlsSock(text.parse().map_err(|e| err!("'{}' is not a valid inet address: {}", text, e))?)),
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
