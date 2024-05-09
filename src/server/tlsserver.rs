

use std::net::SocketAddr;
use mio::net::{TcpListener, TcpStream};
use native_tls::{
	Identity,
	TlsAcceptor,
	TlsStream,
	MidHandshakeTlsStream,
	HandshakeError,
};
use crate::{
	util::Holder,
	errors::{AnyError},
};

use super::{
	connection::Connection,
	Server,
	ConnectionId,
	Message,
	MessageUpdates,
	ServerError
};


pub struct TlsServer<T: Connection<TlsStream<TcpStream>>> {
	listener: TcpListener,
	acceptor: TlsAcceptor,
	connections: Holder<ConnectionId, T>,
	partial_connections: Vec<MidHandshakeTlsStream<TcpStream>>,
}

impl <T: Connection<TlsStream<TcpStream>>> TlsServer<T> {

	pub fn new(addr: SocketAddr, identity: Identity) -> Result<Self, AnyError> {
		let listener = TcpListener::bind(addr)?;
		let acceptor = TlsAcceptor::new(identity)?;
		Ok( Self {
			listener,
			acceptor,
			connections: Holder::new(),
			partial_connections: Vec::new(),
		})
	}
}

impl <T: Connection<TlsStream<TcpStream>>> Server for TlsServer<T> {

	fn accept_pending_connections(&mut self) -> Vec<ConnectionId> {

		let mut new_connections = Vec::new();
		let partial_connections = std::mem::replace(&mut self.partial_connections, Vec::new());
		let mut results: Vec<Result<TlsStream<TcpStream>, HandshakeError<TcpStream>>> = partial_connections.into_iter()
			.map(MidHandshakeTlsStream::handshake)
			.collect();
		while let Ok((stream, _address)) = self.listener.accept() {
			results.push(self.acceptor.accept(stream));
		}

		for result in results {
			match result {
				Ok(tls_stream) => {
					let con = Connection::new(tls_stream).unwrap();
					let id = self.connections.insert(con);
					new_connections.push(id);
				}
				Err(HandshakeError::Failure(err)) => panic!("Failed tls handshake: {}", err),
				Err(HandshakeError::WouldBlock(mid_stream)) => self.partial_connections.push(mid_stream),
			}
		}
		new_connections
	}


	fn recv_pending_messages(&mut self) -> MessageUpdates {
		let mut messages: Vec<Message> = Vec::new();
		let mut to_remove: Vec<ConnectionId> = Vec::new();
		for (connection_id, connection) in self.connections.iter_mut(){
			match connection.read() {
				Err(_e) => {
					to_remove.push(*connection_id);
				}
				Ok((con_messages, closed)) => {
					for message in con_messages {
						messages.push(Message{connection: *connection_id, content: message});
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
}

