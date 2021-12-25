

use std::collections::HashMap;
use std::io;

use serde_json::{Value, json};
use serde::{Serialize, Deserialize};
use unicode_categories::UnicodeCategories;
use chrono::Utc;

use crate::{
	controls::{Control, Action},
	server::Server,
	sprite::Sprite,
	PlayerId
};



#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all="lowercase")]
enum Message {
	Introduction(String, String),
	Chat(String),
	Input(Value)
}

struct MessageError {
	typ: String,
	text: String
}

macro_rules! merr {
	(name, $text: expr) => {merr!("invalidname", $text)};
	(action, $text: expr) => {merr!("invalidaction", $text)};
	(msg, $text: expr) => {merr!("invalidmessage", $text)};
	($typ: expr, $text: expr) => {MessageError{typ: $typ.to_string(), text: $text.to_string()}};
}


pub struct GameServer {
	players: HashMap<(usize, usize), PlayerId>,
	connections: HashMap<PlayerId, (usize, usize)>,
	servers: Vec<Box<dyn Server>>,
	#[allow(dead_code)]
	admins: String
}

impl GameServer {
	pub fn new(servers: Vec<Box<dyn Server>>, admins: String) -> GameServer {
		GameServer {
			players: HashMap::new(),
			connections: HashMap::new(),
			servers,
			admins
		}
	}
	
	pub fn update(&mut self) -> Vec<Action>{
		for server in self.servers.iter_mut(){
			let _ = server.accept_pending_connections();
		}
		
		let mut actions: Vec<Action> = Vec::new();
		let mut input = Vec::new();
		for (serverid, server) in self.servers.iter_mut().enumerate(){
			let (messages, left) = server.recv_pending_messages();
			input.push((serverid, messages, left));
		}
		for (serverid, messages, left) in input {
			for (id, message) in messages {
				match serde_json::from_str(&message) {
					Ok(msg) => {
						match self.handle_message((serverid, id), msg){
							Ok(Some(action)) => {actions.push(action);}
							Ok(None) => {}
							Err(err) => {let _ = self.send_error((serverid, id), &err.typ, &err.text);}
						}
					}
					Err(_err) => {
						{let _ = self.send_error((serverid, id), "invalidmessage", &format!("Invalid message structure: {}", &message));}
					}
				}
			}
			for id in left {
				if let Some(player) = self.players.remove(&(serverid, id)){
					self.connections.remove(&player);
					self.broadcast_message(&format!("{} disconnected", player));
					actions.push(Action::Leave(player.clone()));
				}
			}
		}
		actions
	}
	
	fn send_error(&mut self, (serverid, connectionid): (usize, usize), errname: &str, err_text: &str) -> Result<(), io::Error>{
		self.servers[serverid].send(connectionid, &json!(["error", errname, err_text]).to_string().as_str())
	}
	
	pub fn broadcast_message(&mut self, text: &str){
		println!("m {}      {}", text, Utc::now());
		self.broadcast_json(json!(["message", text, ""]));
	}
	
	pub fn broadcast_json(&mut self, value: Value){
		self.broadcast(value.to_string().as_str());
	}
	
	pub fn broadcast(&mut self, txt: &str){
		for (serverid, id) in self.players.keys() {
			let _ = self.servers[*serverid].send(*id, txt);
		}
	}
	
	pub fn send(&mut self, player: &PlayerId, value: Value) -> Result<(), io::Error> {
		match self.connections.get(player) {
			Some((serverid, id)) => {
				self.servers[*serverid].send(*id, value.to_string().as_str())
			}
			None => Err(io::Error::new(io::ErrorKind::Other, "unknown player name"))
		}
	}
	
	pub fn send_player_error(&mut self, player: &PlayerId, errname: &str, err_text: &str) -> Result<(), io::Error> {
		self.send(player, json!(["error", errname, err_text]))
	}
	
	fn handle_message(&mut self, (serverid, connectionid): (usize, usize), msg: Message) -> Result<Option<Action>, MessageError> {
		let id = (serverid, connectionid);
		match msg {
			Message::Introduction(name, spritename) => {
				let sprite = Sprite::player_sprite(&spritename).ok_or(merr!(name, format!("Invalid player sprite: {}", spritename)))?;
				if name.len() > 99 {
					return Err(merr!(name, "A name can not be longer than 99 bytes"));
				}
				if name.len() == 0 {
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
				if let Err(_) = self.send(&player, json!(["connected", format!("successfully connected as {}", player)])){
					return Err(merr!("server", "unable to send connected message"))
				}
				Ok(Some(Action::Join(player, sprite)))
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



