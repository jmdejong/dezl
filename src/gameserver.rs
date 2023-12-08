

use std::collections::HashMap;

use serde_json::{Value, json};
use serde::{Serialize, Deserialize};
use unicode_categories::UnicodeCategories;
use time::OffsetDateTime;
use crate::util::{HolderId, Holder};

use crate::{
	controls::{Control, Action},
	server::{
		Server,
		ServerEnum,
		ConnectionId,
		ServerError
	},
	PlayerId
};



#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all="lowercase")]
enum Message {
	Introduction(String),
	Chat(String),
	Input(Value)
}

struct MessageError {
	typ: String,
	text: String
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ServerId(usize);

impl HolderId for ServerId {
	fn next(&self) -> Self { Self(self.0 + 1) }
	fn initial() -> Self { Self(1) }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ClientId(ServerId, ConnectionId);


macro_rules! merr {
	(name, $text: expr) => {merr!("invalidname", $text)};
	(action, $text: expr) => {merr!("invalidaction", $text)};
	(msg, $text: expr) => {merr!("invalidmessage", $text)};
	($typ: expr, $text: expr) => {MessageError{typ: $typ.to_string(), text: $text.to_string()}};
}


pub struct GameServer {
	players: HashMap<ClientId, PlayerId>,
	connections: HashMap<PlayerId, ClientId>,
	servers: Holder<ServerId, ServerEnum>,
}

impl GameServer {
	pub fn new(raw_servers: Vec<ServerEnum>) -> GameServer {
		let mut servers = Holder::new();
		for server in raw_servers.into_iter() {
			servers.insert(server);
		}
		GameServer {
			players: HashMap::new(),
			connections: HashMap::new(),
			servers
		}
	}
	
	pub fn update(&mut self) -> Vec<Action>{
		for (_serverid, server) in self.servers.iter_mut(){
			let _ = server.accept_pending_connections();
		}
		
		let mut actions: Vec<Action> = Vec::new();
		
		let mut raw_messages: Vec<(ClientId, String)> = Vec::new();
		let mut to_remove: Vec<ClientId> = Vec::new();
		
		for (serverid, server) in self.servers.iter_mut() {
			let message_updates = server.recv_pending_messages();
			for connectionid in message_updates.to_remove {
				to_remove.push(ClientId(*serverid, connectionid));
			}
			for raw_message in message_updates.messages{
				raw_messages.push((ClientId(*serverid, raw_message.connection), raw_message.content));
			}
		}
		for (clientid, content) in raw_messages {
			match serde_json::from_str(&content) {
				Ok(msg) => {
					match self.handle_message(clientid, msg){
						Ok(Some(action)) => {actions.push(action);}
						Ok(None) => {}
						Err(err) => {let _ = self.send_error(clientid, &err.typ, &err.text);}
					}
				}
				Err(_err) => {
					let _ = self.send_error(
						clientid,
						"invalidmessage",
						&format!("Invalid message structure: {}", &content)
					);
				}
			}
		}
		for clientid in to_remove {
			if let Some(player) = self.players.remove(&clientid){
				self.connections.remove(&player);
				self.broadcast_message(&format!("{} disconnected", player));
				actions.push(Action::Leave(player.clone()));
			}
		}
		actions
	}
	
	fn send_error(&mut self, clientid: ClientId, errname: &str, err_text: &str) -> Result<(), ServerError>{
		self.servers.get_mut(&clientid.0)
			.unwrap()
			.send(clientid.1, json!(["error", errname, err_text]).to_string().as_str())
	}
	
	fn broadcast_message(&mut self, text: &str){
		println!("m {}      {}", text, OffsetDateTime::now_utc());
		self.broadcast_json(json!(["message", text, ""]));
	}
	
	fn broadcast_json(&mut self, value: Value){
		self.broadcast(value.to_string().as_str());
	}
	
	fn broadcast(&mut self, txt: &str){
		for ClientId(serverid, id) in self.players.keys() {
			let _ = self.servers.get_mut(serverid)
				.unwrap()
				.send(*id, txt);
		}
	}
	
	pub fn send(&mut self, player: &PlayerId, value: Value) -> Result<(), ServerError> {
		match self.connections.get(player) {
			Some(ClientId(serverid, id)) => {
				self.servers.get_mut(serverid)
					.unwrap()
					.send(*id, value.to_string().as_str())
			}
			None => Err(ServerError::Custom(format!("unknown player name {}", player)))
		}
	}
	
	pub fn send_player_error(&mut self, player: &PlayerId, errname: &str, err_text: &str) -> Result<(), ServerError> {
		self.send(player, json!(["error", errname, err_text]))
	}
	
	fn handle_message(&mut self, clientid: ClientId, msg: Message) -> Result<Option<Action>, MessageError> {
		let id = clientid;
		match msg {
			Message::Introduction(name) => {
				if name.len() > 60 {
					return Err(merr!(name, "A name can not be longer than 60 bytes"));
				}
				if name.is_empty() {
					return Err(merr!(name, "A name must have at least one character"));
				}
				for chr in name.chars() {
					if !(chr.is_letter() || chr.is_number() || chr.is_punctuation_connector()){
						return Err(merr!(name, "A name can only contain letters, numbers and underscores"));
					}
				}
				if self.players.contains_key(&id) {
					return Err(merr!(action, "You can not change your name"));
				}
				let player = PlayerId(name);
				if self.connections.contains_key(&player) {
					return Err(merr!("nametaken", "Another connection to this player exists already"));
				}
				self.broadcast_message(&format!("{} connected", player));
				self.players.insert(id, player.clone());
				self.connections.insert(player.clone(), id);
				let confirmation_message = json!(["connected", format!("successfully connected as {}", player)]);
				if self.send(&player, confirmation_message).is_err() {
					return Err(merr!("server", "unable to send connected message"))
				}
				Ok(Some(Action::Join(player)))
			}
			Message::Chat(text) => {
				let player = self.players.get(&id).ok_or(merr!(action, "Set a valid name before you send any other messages"))?.clone();
				self.broadcast_message(&format!("{}: {}", player, text));
				Ok(None)
			}
			Message::Input(inp) => {
				let player = self.players.get(&id).ok_or(merr!(action, "Set a name before you send any other messages"))?;
				let control = Control::deserialize(&inp).map_err(|err| merr!(action, &format!("unknown action {} {}", inp, err)))?;
				Ok(Some(Action::Input(player.clone(), control)))
			}
		}
	}
}



