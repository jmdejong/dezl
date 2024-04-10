#![allow(clippy::expect_fun_call)]
use std::thread;
use std::time::{Instant, Duration};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use clap::Parser;
use time::OffsetDateTime;

mod action;
mod basemap;
mod config;
mod controls;
mod creature;
mod creaturemap;
mod creatures;
mod crop;
mod errors;
mod gameserver;
mod grid;
mod heightmap;
mod infinitemap;
mod inventory;
mod item;
mod loadedareas;
mod map;
mod persistence;
mod player;
mod pos;
mod random;
mod randomtick;
mod server;
mod sprite;
mod tile;
mod tiledmap;
mod timestamp;
mod util;
mod world;
mod worldmessages;

use self::{
	pos::Pos,
	player::PlayerId,
	errors::Result,
	
	gameserver::{GameServer, ErrTyp, ServerMessage, WelcomeMsg},
	server::ServerEnum,
	controls::Action,
	world::World,
	worldmessages::MessageCache,
	persistence::{PersistentStorage, file::FileStorage, LoaderError},
	config::{Config, WorldAction, WorldConfig, MapDef},
	basemap::BaseMapImpl,
	creature::PlayerSave,
};


#[allow(clippy::expect_fun_call)]
fn main(){
	
	let config = Config::parse();
	
	match config.world_action {
		WorldAction::New{conf, mapdef} => {
			let persistence = FileStorage::initialize(&conf.name).unwrap();
			if let Err(LoaderError::MissingResource(_)) = persistence.load_world() {
				let basemap = BaseMapImpl::from_mapdef(mapdef.clone()).expect(&format!("Can't load base map {:?}", &mapdef));
				start_world(World::new(conf.name.clone(), basemap, mapdef), persistence, conf);
			} else {
				panic!("World '{}' already exists", &conf.name);
			}
		}
		WorldAction::Load(conf) => {
			let persistence = FileStorage::initialize(&conf.name).unwrap();
			let saved = persistence.load_world().expect("Can't load world");
			let mapdef = &saved.mapdef;
			let basemap = BaseMapImpl::from_mapdef(mapdef.clone()).expect(&format!("Can't load base map {:?}", &mapdef));
			start_world(World::load(saved, basemap), persistence, conf);
		}
		WorldAction::Bench{iterations} => {
			bench_view(iterations);
		}
	}
}

fn start_world(mut world: World, persistence: FileStorage, config: WorldConfig) {

	// eprintln!("stucture size: {}", std::mem::size_of::<crate::tile::Structure>());
	// eprintln!("tile size: {}", std::mem::size_of::<crate::tile::Tile>());
	eprintln!("Server admin(s): {}", config.admins);

	let adresses = config.address
		.unwrap_or_else(||
			(if cfg!(target_os = "linux") {
				vec!["abstract:dezl", "inet:0.0.0.0:9231"]
			} else {
				vec!["inet:127.0.0.1:9231"]
			})
			.iter()
			.map(|a| a.parse().unwrap())
			.collect()
		);
	eprintln!("adresses: {:?}", adresses);
	let servers: Vec<ServerEnum> =
		adresses
		.iter()
		.map(|a| a.to_server().unwrap())
		.collect();

	let mut gameserver = GameServer::new(servers);


	let mut message_cache = MessageCache::default();
	
	// close handler
	// todo: don't let the closing wait on sleep (using a timer thread or recv_timeout)
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
	ctrlc::set_handler(move || {
		eprintln!("shutting down");
		r.store(false, Ordering::SeqCst);
	}).expect("can't set close handler");
	
	
	eprintln!("dezl started world {} on {}", config.name, OffsetDateTime::now_utc());
	
	while running.load(Ordering::SeqCst) {
		let update_start = Instant::now();
		let actions = gameserver.update();
		for action in actions {
			match action {
				Action::Input(player, control) => {
					if let Err(err) = world.control_player(&player, control){
						eprintln!("error controlling player {:?}: {:?}", player, err);
					}
				}
				Action::Configure(player, config) => {
					if let Err(err) = world.configure_player(&player, config){
						eprintln!("error configuring player {:?}: {:?}", player, err);
					}
				}
				Action::Join{player, name, config: player_config} => {
					let playersave = match persistence.load_player(&player) {
						Ok(save) => save,
						Err(LoaderError::MissingResource(_)) => world.default_player(name),
						Err(err) => {
							eprintln!("Error loading save for player {:?}: {:?}", player, err);
							gameserver.send_or_log(&player, ServerMessage::Error(ErrTyp::LoadError, "could not load saved player data"));
							continue
						}
					};
					if let Err(err) = world.add_player(&player, playersave, player_config) {
						eprintln!("Error: can not add player {:?}: {:?}", player, err);
						gameserver.send_or_log(&player, ServerMessage::Error(ErrTyp::WorldError, "invalid room or savefile"));
					}
					gameserver.send_or_log(&player, ServerMessage::Welcome(WelcomeMsg{tick_millis: config.step_duration}));
				}
				Action::Leave(player) => {
					if let Some(saved) = world.save_player(&player) {
						persistence.save_player(&player, saved).unwrap();
						if let Err(err) = world.remove_player(&player) {
							eprintln!("Error: can not remove player {:?}: {:?}", player, err);
						}
					}
					message_cache.remove(&player);
				}
			}
		}

		let read_done = Instant::now();
		world.update();
		let update_done = Instant::now();
		let messages = world.view();
		let view_done = Instant::now();
		for (player, mut message) in messages {
			message_cache.trim(&player, &mut message);

// 			eprintln!("m {}", message.to_json());
			gameserver.send_or_log(&player, ServerMessage::World(Box::new(message)));
		}
		let send_done = Instant::now();
		world.clear_step();
		if world.time.0 % 100 == 1 {
			save(&world, &persistence);
		}
		let save_done = Instant::now();
		let elapsed_time = update_start.elapsed();
		if elapsed_time >= Duration::from_millis(5) {
			eprintln!(
				"Step {} took {} milliseconds. read: {}, update: {}, view: {}, send: {}, save: {}",
				world.time.0,
				elapsed_time.as_millis(),
				read_done.duration_since(update_start).as_millis(),
				update_done.duration_since(read_done).as_millis(),
				view_done.duration_since(update_done).as_millis(),
				send_done.duration_since(view_done).as_millis(),
				save_done.duration_since(send_done).as_millis(),
			);
		}
		thread::sleep(Duration::from_millis(config.step_duration).saturating_sub(elapsed_time));
	}
	save(&world, &persistence);
	eprintln!("shutting down on {}", OffsetDateTime::now_utc());
}

fn save(world: &World, persistence: &impl PersistentStorage) {
	persistence.save_world(world.save()).unwrap();
	for player in world.list_players() {
		persistence.save_player(&player, world.save_player(&player).unwrap()).unwrap();
	}
	eprintln!("saved world {} on step {}", world.name, world.time.0);
}

fn bench_view(iterations: usize) {
	let mapdef = MapDef::Infinite{seed: 9876};
	let basemap = BaseMapImpl::from_mapdef(mapdef.clone()).expect(&format!("Can't load base map {:?}", &mapdef));
	let mut world = World::new("bench".to_string(), basemap, mapdef);
	let player_id = PlayerId::create("Player").unwrap();
	let now = Instant::now();
	for i in 0..iterations {
		let player_save = PlayerSave::new("Player".to_string(), Pos::new(i as i32 * 121 - 22, i as i32 * 8 - 63));
		world.add_player(&player_id, player_save, Default::default()).unwrap();
		world.update();
		world.view();
		world.remove_player(&player_id).unwrap();
		world.update();
	}
	eprintln!("millis: {}", now.elapsed().as_millis());
}
