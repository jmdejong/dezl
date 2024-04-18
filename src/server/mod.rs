mod tcpserver;
mod tlsserver;
mod unixserver;
mod address;
mod connection;

use enum_dispatch::enum_dispatch;

use crate::util::HolderId;
use unixserver::UnixServer;
pub use address::Address;
pub use connection::ConnectionError;
use native_tls::TlsStream;
use mio::net::TcpStream;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConnectionId(pub usize);

impl HolderId for ConnectionId {
	fn next(&self) -> Self { Self(self.0 + 1) }
	fn initial() -> Self { Self(1) }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
	pub connection: ConnectionId,
	pub content: String
}

#[derive(Debug, Clone)]
pub struct MessageUpdates {
	pub messages: Vec<Message>,
	pub to_remove: Vec<ConnectionId>
}


#[derive(Debug)]
pub enum ServerError {
	InvalidIndex(ConnectionId),
	Connection(ConnectionError),
	Custom(String),
}

#[enum_dispatch]
pub trait Server {
	
	fn accept_pending_connections(&mut self) -> Vec<ConnectionId>;
	
	fn recv_pending_messages(&mut self) -> MessageUpdates;
	
	fn send(&mut self, id: ConnectionId, text: &str) -> Result<(), ServerError>;
	
	fn broadcast(&mut self, text: &str);
	
	fn get_name(&self, _id: ConnectionId) -> Option<String> {
		None
	}
}

type VarInetServer = tcpserver::TcpServer<connection::DynCon<TcpStream>>;
type WebTlsServer = tlsserver::TlsServer<connection::WebSocketConnection<TlsStream<TcpStream>>>;
type StreamTlsServer = tlsserver::TlsServer<connection::StreamConnection<TlsStream<TcpStream>>>;

#[enum_dispatch(Server)]
pub enum ServerEnum {
	VarInetServer,
	WebTlsServer,
	StreamTlsServer,
	UnixServer,
}


