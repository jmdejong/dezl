

use std::io;
use std::path::Path;
use std::os::unix::io::AsRawFd;
use mio_uds::{UnixListener, UnixStream};
use slab::Slab;
use nix::sys::socket::getsockopt;
use nix::sys::socket::sockopt;

use super::streamconnection::StreamConnection;
use super::Server;


pub struct UnixServer {
	listener: UnixListener,
	connections: Slab<StreamConnection<UnixStream>>
}

impl UnixServer {

	pub fn new(addr: &Path) -> Result<UnixServer, io::Error> {
		let listener = UnixListener::bind(addr)?;
		Ok( UnixServer {
			listener,
			connections: Slab::new()
		})
	}
	
	
}

impl Server for UnixServer {

	fn accept_pending_connections(&mut self) -> Vec<usize> {
		let mut new_connections = Vec::new();
		loop {
			match self.listener.accept() {
				Ok(Some((stream, _address))) => {
					let con = StreamConnection::new(stream);
					let id = self.connections.insert(con);
					new_connections.push(id);
				}
				Ok(None) => {
					break;
				}
				Err(_e) => {
					break;
				}
			}
		}
		new_connections
	}


	fn recv_pending_messages(&mut self) -> (Vec<(usize, String)>, Vec<usize>){
	// 	let mut buf = [0; 2048];
		let mut messages: Vec<(usize, String)> = Vec::new();
		let mut to_remove = Vec::new();
		for (key, connection) in self.connections.iter_mut(){
			match connection.read() {
				Err(_e) => {
					to_remove.push(key);
				}
				Ok((con_messages, closed)) => {
					for message in con_messages {
						messages.push((key, message));
					}
					if closed {
						to_remove.push(key);
					}
				}
			}
		}
		for key in to_remove.iter() {
			self.connections.remove(*key);
		}
		(messages, to_remove)
	}

	fn broadcast(&mut self, text: &str) {
		for (_id, conn) in self.connections.iter_mut() {
			let _ = conn.send(text);
		}
	}
	
	fn send(&mut self, id: usize, text: &str) -> Result<(), io::Error> {
		match self.connections.get_mut(id){
			Some(conn) => {
				conn.send(text)
			}
			None => Err(io::Error::new(io::ErrorKind::Other, "index is empty"))
		}
	}
	
	#[cfg(any(target_os = "linux", target_os = "android"))]
	fn get_name(&self, id: usize) -> Option<String> {
		let connection = self.connections.get(id)?;
		let fd = connection.stream.as_raw_fd();
		let peercred = getsockopt(fd, sockopt::PeerCredentials).ok()?;
		let uid = peercred.uid();
		let user = users::get_user_by_uid(uid)?;
		let name = user.name();
		Some(name.to_string_lossy().to_string())
	}
	
	#[cfg(not(any(target_os = "linux", target_os = "android")))]
	fn get_name(&self, id: usize) -> Option<String> {
		None
	}
}

