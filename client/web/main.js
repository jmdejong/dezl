"use strict";


window.addEventListener("load", main);

function main(){
	let loginForm = document.getElementById("login");
	loginForm.hidden = false;
	loginForm.addEventListener("submit", start);
	let hostInput = document.getElementById("hostinput");
	if (hostInput.value === hostInput.defaultValue) {
		hostInput.value = `ws://${window.location.hostname || "localhost"}:9231`;
	}
}



function start(e) {
	let form = e.target;
	let username = form.username.value;
	let host = form.host.value;
	
	let canvas = document.getElementById("canvas");

	let fuzzTemplate = new FuzzTemplate(document.getElementById("fuzz-template"), 1, 1);

	let spritemap = new SpriteMap();
	spritemap.addSprites(
		document.getElementById("spritemap"),
		{
			player: {x: 0, y: 0, layer: "creatures"},
			sage: {x: 1, y: 0},
			worktable: {x: 6, y: 0},
			altar: {x: 7, y: 0},
			grass1: {x: 0, y: 1, layer: "ground"},
			grass2: {x: 1, y: 1, layer: "ground"},
			grass3: {x: 2, y: 1, layer: "ground"},
			dirt: {x: 3, y: 1, layer: "ground"},
			rockmid: {x: 4, y: 1, border: "#222", layer: "base"},
			" ": {x: 4, y: 1},
			rock: {x: 5, y: 1, border: "#222", layer: "base"},
			water: {x: 6, y: 1, border: "#004", layer: "base"},
			moss: {x: 7, y: 1, layer: "ground"},
			deadleaves: {x: 0, y: 2, layer: "ground"},
			densegrass: {x: 1, y: 2, layer: "ground"},
			wall: {x: 2, y: 2, border: "#222", layer: "base"},
			woodwall: {x: 3, y: 2, border: "#220", layer: "base"},
			stonefloor: {x: 4, y: 2, layer: "base"},
			rockfloor: {x: 5, y: 2, layer: "ground"},
			rush: {x: 0, y: 3},
			pitcherplant: {x: 1, y: 3},
			tree: {x: 2, y: 5, ho: true},
			oldtree: {x: 3, y: 5},
			oldtreetinder: {x: 4, y: 5, ho: true},
			youngtree: {x: 1, y: 5},
			sapling: {x: 0, y: 5},
			shrub: {x: 6, y: 3},
			bush: {x: 7, y: 3},
			reed: {x: 2, y: 3},
			gravel: {x: 5, y: 3},
			pebble: {x: 0, y: 6},
			stone: {x: 1, y: 6},
			stick: {x: 2, y: 6},
		},
		8,
		fuzzTemplate
	);
	let client = new Client(username, host, new Display(canvas, spritemap, fuzzTemplate.asSprite()));
	client.start()
	form.hidden = true;
	window.game_client_debug = client;
}


class Client {
	constructor(username, host, display) {
		this.username = username;
		this.host = host;
		this.display = display;
		this.websocket = null;
		this.delay = parseParameters().delay|0;
		this.tick = 0;
		this.fps = 10;
	}
	
	start(){
		console.log("connecting to '" + this.host + "' as '" + this.username + "'");
		this.websocket = new WebSocket(this.host);
		this.websocket.addEventListener("open", e => {
			document.getElementById("game").hidden = false;
			e.target.send(JSON.stringify({introduction: this.username}));
		});
		let keymap = {
			KeyW: {move: "north"},
			ArrowUp: {move: "north"},
			KeyS: {move: "south"},
			ArrowDown: {move: "south"},
			KeyA: {move: "west"},
			ArrowLeft: {move: "west"},
			KeyD: {move: "east"},
			ArrowRight: {move: "east"},
			Period: {select: "next"},
			Comma: {select: "previous"},
			NumpadAdd: {select: "next"},
			NumpadSubtract: {select: "previous"},
			Equal: {select: "next"},
			Minus: {select: "previous"},
		};
		let shiftKeymap = {
			KeyW: {interact: "north"},
			ArrowUp: {interact: "north"},
			KeyS: {interact: "south"},
			ArrowDown: {interact: "south"},
			KeyA: {interact: "west"},
			ArrowLeft: {interact: "west"},
			KeyD: {interact: "east"},
			ArrowRight: {interact: "east"},
		}
		document.addEventListener("keydown", e => {
			if (document.activeElement.classList.contains("captureinput")){
				if (e.code == "Escape") {
					document.activeElement.blur();
				}
				return;
			}
			let action = (e.shiftKey && shiftKeymap[e.code]) || keymap[e.code];
			if (action){
				e.preventDefault();
				this.sendInput(action);
			} else {
				if (e.code == "Enter" || e.code == "KeyT") {
					e.preventDefault();
					document.getElementById("textinput").focus()
				}
			}
		});
		document.getElementById("control-up").addEventListener("click", e => {
			this.sendInput({move: "north"});
		});
		document.getElementById("control-left").addEventListener("click", e => {
			this.sendInput({move: "west"});
		});
		document.getElementById("control-right").addEventListener("click", e => {
			this.sendInput({move: "east"});
		});
		document.getElementById("control-down").addEventListener("click", e => {
			this.sendInput({move: "south"});
		});
		this.websocket.addEventListener("error", console.error);
		if (this.delay) {
			this.websocket.addEventListener("message", msg => setTimeout(() => this.handleMessage(msg), this.delay));
		} else {
			this.websocket.addEventListener("message", msg => this.handleMessage(msg));
		}
		document.getElementById("chatinput").addEventListener("submit", e => {
			let inp = e.target.command;
			this.onCommand(inp.value)
			inp.value = "";
			document.activeElement.blur();
		});
		this.resize();
		window.addEventListener('resize', e => this.resize());
		this.update(0);
	}
	
	handleMessage(msg) {
		let data = JSON.parse(msg.data)
		let type = data[0];
		if (type === "message") {
			this.print(data[1]);
		} else if (type === "messages") {
			for (let mesg of data[1]) {
				this.print(data[1], data[0]);
			}
		} else if (type === "world") {
			this.handleWorldMessage(data[1]);
			this.draw();
			this.display.redraw();
		} else if (type == "welcome") {
			if (data[1].tick_millis) {
				this.fps = 1000 / data[1].tick_millis;
			}
		} else {
			console.log("unknown", data);
		}
	}

	handleWorldMessage(m){
		this.tick = m.t;
		if (m.viewarea) {
			this.display.setViewArea(m.viewarea.area);
		}
		if (m.section) {
			this.display.drawSection(m.section.area.w, m.section.area.h, m.section.area.x, m.section.area.y, m.section.field, m.section.mapping);
		}
		if (m.changecells) {
			this.display.changeTiles(m.changecells);
		}
		if (m.dynamics) {
			this.entities = m.dynamics;
		}
		if (m.playerpos) {
			this.position = m.playerpos;
			document.getElementById("coordinates").textContent = `${m.playerpos.pos[0]}, ${m.playerpos.pos[1]}`;
		}
		if (m.inventory) {
			this.setInventory(m.inventory[0], m.inventory[1]);
		}
		if (m.sounds) {
			for (let sound of m.sounds) {
				this.print(sound[1], sound[0]);
			}
		}
	}

	setInventory(items, selected) {
		let table = document.getElementById("inventory");

		let rows = table.querySelectorAll("li");
		rows.forEach(function(row) {
			row.remove();
		});

		for (let i=0; i<items.length; ++i) {
			let item = items[i];
			let name = item[0];
			let quantity = item[1];
			let row = document.createElement("li");
			row.onclick = e => {
				this.sendInput({select: {idx: i | 0}});
			}
			row.className = "inv-row";

			let nm = document.createElement("span");
			nm.className = "inventory-name";
			nm.innerText = name;
			row.appendChild(nm);

			let am = document.createElement("span");
			am.className = "inventory-amount";
			if (quantity !== null && quantity !== undefined) {
				am.innerText = quantity;
			}
			row.appendChild(am);

			if (i === selected) {
				// nm.className += " inv-selected";
				// am.className += " inv-selected";
				row.className += " inv-selected";
			};
			table.appendChild(row);
			if (Math.abs(i - selected) <= 1) {
				row.scrollIntoView();
			}
		}
	}

	sendInput(msg) {
		let f = () => {
			if (this.websocket.readyState === WebSocket.OPEN){
				this.websocket.send(JSON.stringify({input: msg}));
			} else {
				console.error("can't send input: websocket not open", this.websocket.readyState,  msg);
			}
		};
		if (this.delay) {
			setTimeout(f, this.delay);
		} else {
			f();
		}
	}
	
	print(msg, type) {
		console.log("msg", msg);
		let li = document.createElement("li");
		li.innerText = msg;
		let messages = document.getElementById("messages");
		let isAtBottom = messages.lastElementChild && messages.scrollTop + messages.clientHeight >= messages.scrollHeight - messages.lastElementChild.scrollHeight;
		messages.appendChild(li);
		if (isAtBottom){
			li.scrollIntoView();
		}
	}
	
	onCommand(command) {
		this.websocket.send(JSON.stringify({chat: command}));
	}

	resize() {
		this.zooms = this.zooms || 0
		this.zooms += 1
		this.display.resize(window.innerWidth, window.innerHeight);
	}

	draw() {
		if (this.position) {
			let [cx, cy] = this.position.pos;
			if (this.position.movement && this.tick < this.position.movement.e) {
				let start = this.position.movement.s;
				let progress = (this.tick - start) / (this.position.movement.e - start);
				cx += (this.position.movement.f[0] - cx) * (1-progress)
				cy += (this.position.movement.f[1] - cy) * (1-progress)
			}
			this.display.setCenter(cx, cy);
		}
		if (this.entities) {
			this.display.drawDynamics(this.entities.map(entity => {
				let [x, y] = entity.p;
				if (entity.m && this.tick < entity.m.e) {
					let start = entity.m.s;
					let progress = (this.tick - start) / (entity.m.e - start);
					x += (entity.m.f[0] - x) * (1 - progress);
					y += (entity.m.f[1] - y) * (1 - progress);
				}
				return {x: x, y: y, sprite: entity.s};
			}));
		}
		this.display.redraw();
	}

	update(t, duration) {
		this.tick += duration / 1000 * this.fps;
		this.draw();
		requestAnimationFrame(newTime => this.update(newTime, newTime - t));
	}
}

function parseParameters(){
	let ps = new URLSearchParams(window.location.search)
	let parameters = {};
	for (let p of ps){
		parameters[p[0]] = p[1];
	}
	return parameters;
}
