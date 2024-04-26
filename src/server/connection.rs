
use std::io;
use std::io::{Read, Write};
use tungstenite::{
	WebSocket,
	Message,
	handshake::MidHandshake,
	handshake::server::{ServerHandshake, NoCallback},
};


#[derive(Debug)]
pub enum ConnectionError {
	IO(io::Error),
	Tungstenite(tungstenite::Error),
	NotReadyYet,
	Custom(String),
	UnknownProtocol
}

pub trait Connection<T: Read+Write>: Sized {

	fn new(stream: T) -> Result<Self, ConnectionError>;
	
	fn read(&mut self) -> Result<(Vec<String>, bool), ConnectionError>;
	
	fn send(&mut self, text: &str) -> Result<(), ConnectionError>;

}


pub struct StreamConnection<T: Read+Write> {
	stream: T,
	buffer: Vec<u8>
}

impl <T: Read+Write> StreamConnection<T> {
	pub fn stream(&self) -> &T {
		&self.stream
	}
}

impl <T: Read+Write>Connection<T> for StreamConnection<T> {

	fn new(stream: T) -> Result<Self, ConnectionError> {
		Ok(Self { stream, buffer: Vec::new()})
	}
	
	fn read(&mut self) -> Result<(Vec<String>, bool), ConnectionError> {
		let mut buf = [0; 2048];
		let mut closed = false;
		loop {
			match self.stream.read(&mut buf) {
				Err(e) => {
					if e.kind() == io::ErrorKind::WouldBlock {
						break;
					} else {
						return Err(ConnectionError::IO(e));
					}
				}
				Ok(0) => {
					closed = true;
					break;
				}
				Ok(i) => {
					self.buffer.extend_from_slice(&buf[..i]);
				}
			}
		}
		let mut messages = Vec::new();
		while self.buffer.len() >= 4 {
			let mut header: [u8; 4] = [0;4];
			header.copy_from_slice(&self.buffer[..4]);
			let mlen: usize = u32::from_be_bytes(header) as usize;
			if self.buffer.len() - 4 < mlen {
				break;
			}
			let rest = self.buffer.split_off(4+mlen);
			let message = String::from_utf8_lossy(&self.buffer[4..]).to_string();
			messages.push(message);
			self.buffer = rest;
		}
		Ok((messages, closed))
	}
	
	fn send(&mut self, text: &str) -> Result<(), ConnectionError> {
		let bytes: &[u8] = text.as_bytes();
		let len: u32 = bytes.len() as u32;
		let header: [u8; 4] = len.to_be_bytes();
		self.stream.write_all(&header).map_err(ConnectionError::IO)?;
		self.stream.write_all(bytes).map_err(ConnectionError::IO)?;
		Ok(())
	}
}

#[allow(clippy::large_enum_variant)]
pub enum WebSocketConnection<T: Read+Write> {
	Ready(WebSocket<T>),
	Handshake(MidHandshake<ServerHandshake<T, NoCallback>>),
	Invalid,
}

fn is_wouldblock_error(error: &tungstenite::Error) -> bool {
	if let tungstenite::Error::Io(io_err) = error {
		io_err.kind() == std::io::ErrorKind::WouldBlock 
	} else {
		false
	}
}

impl <T: Read+Write> Connection<T> for WebSocketConnection<T> {
	
	fn new(stream: T) -> Result<Self, ConnectionError> {
		match tungstenite::accept(stream) {
			Ok(socket) => Ok( Self::Ready(socket)),
			Err(tungstenite::HandshakeError::Interrupted(handshake)) => {
				Ok( Self::Handshake(handshake))
			}
			Err(tungstenite::HandshakeError::Failure(err)) => {
				Err(ConnectionError::Tungstenite(err))
			}
		}
	}
	
	fn read(&mut self) -> Result<(Vec<String>, bool), ConnectionError> {
		let mut messages = Vec::new();
		let mut is_closed = false;
		if matches!(self, Self::Handshake(_)) {
			let Self::Handshake(handshake) = std::mem::replace(self, Self::Invalid)
				else { panic!("Websocket is not in handshake state") };
			match handshake.handshake() {
				Ok(socket) => {
					let _ = std::mem::replace(self, Self::Ready(socket));
				}
				Err(tungstenite::HandshakeError::Interrupted(handshake2)) => {
					let _ = std::mem::replace(self, Self::Handshake(handshake2));
				}
				Err(tungstenite::HandshakeError::Failure(err)) => {
					return Err(ConnectionError::Tungstenite(err));
				}
			}
		}
		if let Self::Ready(websocket) = self {
			loop {
				match websocket.read() {
					Err(err) => {
						if is_wouldblock_error(&err) {
							break;
						}
						eprintln!("error reading websocket message: {:?}", err);
						return Err(ConnectionError::Tungstenite(err))
					}
					Ok(Message::Text(text)) => {
						// println!("websocket text: {}", text.clone());
						messages.push(text);
					}
					Ok(Message::Close(_)) => {
						// println!("websocket close");
						is_closed = true;
					}
					Ok(_) => {
						// println!("websocket other");
					}
				}
			}
		}
		Ok((messages, is_closed))
	}
	
	fn send(&mut self, text: &str) -> Result<(), ConnectionError> {
		match self {
			Self::Ready(websocket) => {
				websocket.send(Message::Text(text.to_string()))
					.map_err(ConnectionError::Tungstenite)
			}
			Self::Handshake(_)  | Self::Invalid => {
				Err(ConnectionError::NotReadyYet)
			}
		}
	}
}

pub trait Peek {
	fn peek(&self, buf: &mut [u8]) -> std::io::Result<usize>;
}

impl Peek for mio::net::TcpStream {
	fn peek(&self, buf: &mut [u8]) -> std::io::Result<usize> {
		mio::net::TcpStream::peek(self, buf)
	}
}

#[allow(clippy::large_enum_variant)]
pub enum DynCon<T: Read+Write+Peek> {
	Web(WebSocketConnection<T>),
	TCon(StreamConnection<T>),
	Unknown(T),
	Invalid
}

impl <T: Read+Write+Peek> DynCon<T> {

	fn handshake(&mut self) -> Result<(), ConnectionError> {
		if matches!(self, Self::Unknown(_stream)) {
			let Self::Unknown(stream) = std::mem::replace(self, Self::Invalid)
				else { panic!("DynCon is not in Unknown state") };
			let mut buf: [u8; 4] = [0; 4];
			let connection = match stream.peek(&mut buf) {
				Ok(0) => Self::Unknown(stream),
				Ok(_) => {
					if buf[0] == 0 {
						Self::TCon(StreamConnection::new(stream)?)
					} else if buf[0] == b'G' || buf[0] == b'P' {
						Self::Web(WebSocketConnection::new(stream)?)
					} else {
						return Err(ConnectionError::Custom(format!("invalid first bytes from connection: {:?}", buf)));
					}
				}
				Err(_) => {
					Self::Unknown(stream)
				}
			};
			let _ = std::mem::replace(self, connection);
		}
		Ok(())
	}
}

impl <T: Read+Write+Peek>Connection<T> for DynCon<T> {
	fn new(stream: T) -> Result<Self, ConnectionError> {
		Ok(Self::Unknown(stream))
	}
	
	fn read(&mut self) -> Result<(Vec<String>, bool), ConnectionError> {
		match self {
			Self::Web(conn) => conn.read(),
			Self::TCon(conn) => conn.read(),
			Self::Unknown(_conn) => {
				self.handshake()?;
				Ok((Vec::new(), false))
			}
			Self::Invalid => {
				Err(ConnectionError::Custom("Tried to read from invalid connection".to_string()))
			}
		}
	}
	
	
	fn send(&mut self, text: &str) -> Result<(), ConnectionError> {
		
		match self {
			Self::Web(conn) =>
				conn.send(text),
			Self::TCon(conn) =>
				conn.send(text),
			Self::Unknown(_conn) => 
				Err(ConnectionError::UnknownProtocol),
			Self::Invalid =>
				Err(ConnectionError::Custom("Tried to send to invalid connection".to_string()))
		}
	}
}
