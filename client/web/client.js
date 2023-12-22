"use strict";


class Client {
	constructor(username, host, display, settings) {
		this.username = username;
		this.host = host;
		this.display = display;
		this.websocket = null;
		this.delay = settings.delay|0;
		this.tick = 0;
		this.fps = 10;
		this.keepRunning = true;
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
		this.websocket.addEventListener("close", e => {
			this.print("Connection lost");
			this.keepRunning = false;
		});
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
			this.display.drawDynamics(Object.entries(this.entities).map(([id, entity]) => {
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
		if (this.keepRunning) {
			requestAnimationFrame(newTime => this.update(newTime, newTime - t));
		}
	}
}
