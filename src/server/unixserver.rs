

use std::io;
use std::path::Path;
use std::os::fd::AsRawFd;
use mio::net::{UnixListener, UnixStream};
use nix::sys::socket::getsockopt;
use nix::sys::socket::sockopt;
use nix::unistd::{Uid, User};
use crate::util::Holder;

use super::{
	connection::{Connection, StreamConnection},
	Server,
	ConnectionId,
	Message,
	MessageUpdates,
	ServerError
};


pub struct UnixServer {
	listener: UnixListener,
	connections: Holder<ConnectionId, StreamConnection<UnixStream>>
}

impl UnixServer {

	pub fn new(addr: &Path) -> Result<UnixServer, io::Error> {
		let listener = UnixListener::bind(addr)?;
		Ok( UnixServer {
			listener,
			connections: Holder::new()
		})
	}
}

impl Server for UnixServer {

	fn accept_pending_connections(&mut self) -> Vec<ConnectionId> {
		let mut new_connections = Vec::new();
		while let Ok((stream, _address)) = self.listener.accept() {
			println!("new connection omg!!! {:?}", _address);
			let con = StreamConnection::new(stream).unwrap();
			let id = self.connections.insert(con);
			new_connections.push(id);
		}
		new_connections
	}


	fn recv_pending_messages(&mut self) -> MessageUpdates{
		let mut messages: Vec<Message> = Vec::new();
		let mut to_remove: Vec<ConnectionId> = Vec::new();
		for (connection_id, connection) in self.connections.iter_mut(){
			match connection.read() {
				Err(_e) => {
					to_remove.push(*connection_id);
				}
				Ok((con_messages, closed)) => {
					for message in con_messages {
						messages.push(Message{connection: *connection_id, content: message})
					}
					if closed {
						to_remove.push(*connection_id);
					}
				}
			}
		}
		for key in to_remove.iter() {
			self.connections.remove(key);
		}
		MessageUpdates{messages, to_remove}
	}

	fn broadcast(&mut self, text: &str) {
		for (_id, conn) in self.connections.iter_mut() {
			let _ = conn.send(text);
		}
	}
	
	fn send(&mut self, id: ConnectionId, text: &str) -> Result<(), ServerError> {
		match self.connections.get_mut(&id){
			Some(conn) => {
				conn.send(text).map_err(ServerError::Connection)
			}
			None => Err(ServerError::InvalidIndex(id))
		}
	}
	
	#[cfg(any(target_os = "linux", target_os = "android"))]
	fn get_name(&self, id: ConnectionId) -> Option<String> {
		let conn = self.connections.get(&id)?;
		let peercred = getsockopt(conn.stream().as_raw_fd(), sockopt::PeerCredentials).ok()?;
		let uid = Uid::from_raw(peercred.uid());
		let user: User = User::from_uid(uid).ok()??;
		Some(user.name)
	}
	
	#[cfg(not(any(target_os = "linux", target_os = "android")))]
	fn get_name(&self, id: ConnectionId) -> Option<String> {
		None
	}
}

