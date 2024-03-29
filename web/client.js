"use strict";


const NORTH = "north";
const SOUTH = "south";
const EAST = "east";
const WEST = "west";

class Client {
	constructor(username, host, display, settings) {
		this.username = username;
		this.host = host;
		this.display = display;
		this.websocket = null;
		this.fps = 10;
		this.keepRunning = true;
		this.send = msg => this.sendRaw(JSON.stringify(msg));
		let delay = settings.delay|0;
		if (delay) {
			this.send = msg => setTimeout((() => this.sendRaw(JSON.stringify(msg))), delay);
		}
		this.model = new Model();
		this.readyToDraw = false;
		this.actionBar = new ActionBar();
	}

	start(){
		console.log("connecting to '" + this.host + "' as '" + this.username + "'");
		this.websocket = new WebSocket(this.host);
		this.websocket.addEventListener("open", e => {
			document.getElementById("game").hidden = false;
			e.target.send(JSON.stringify({introduction: this.username}));
		});
		let keymap = {
			KeyW: () => this.startMoving(NORTH),
			ArrowUp: () => this.startMoving(NORTH),
			KeyS: () => this.startMoving(SOUTH),
			ArrowDown: () => this.startMoving(SOUTH),
			KeyA: () => this.startMoving(WEST),
			ArrowLeft: () => this.startMoving(WEST),
			KeyD: () => this.startMoving(EAST),
			ArrowRight: () => this.startMoving(EAST),
			Period: () => this.actionBar.selectRel(1),
			Comma: () => this.actionBar.selectRel(-1),
			NumpadAdd: () => this.actionBar.selectRel(1),
			NumpadSubtract: () => this.actionBar.selectRel(-1),
			Equal: () => this.actionBar.selectRel(1),
			Minus: () => this.actionBar.selectRel(-1),
		};
		let shiftKeymap = {
			KeyW: () => this.act(NORTH),
			ArrowUp: () => this.act(NORTH),
			KeyS: () => this.act(SOUTH),
			ArrowDown: () => this.act(SOUTH),
			KeyA: () => this.act(WEST),
			ArrowLeft: () => this.act(WEST),
			KeyD: () => this.act(EAST),
			ArrowRight: () => this.act(EAST),
		}
		document.addEventListener("keydown", e => {
			if (document.activeElement.classList.contains("captureinput")){
				if (e.code == "Escape") {
					document.activeElement.blur();
					this.stop();
				}
				return;
			}
			let action = (e.shiftKey && shiftKeymap[e.code]) || keymap[e.code];
			if (action){
				e.preventDefault();
				action();
			} else {
				if (e.code == "Enter" || e.code == "KeyT") {
					e.preventDefault();
					document.getElementById("textinput").focus()
				}
			}
		});
		let upKeyMap = {
			KeyW: () => this.stopMoving(NORTH),
			ArrowUp: () => this.stopMoving(NORTH),
			KeyS: () => this.stopMoving(SOUTH),
			ArrowDown: () => this.stopMoving(SOUTH),
			KeyA: () => this.stopMoving(WEST),
			ArrowLeft: () => this.stopMoving(WEST),
			KeyD: () => this.stopMoving(EAST),
			ArrowRight: () => this.stopMoving(EAST),
		}
		document.addEventListener("keyup", e => {
			if (document.activeElement.classList.contains("captureinput")){
				this.stop();
				return;
			} else {
				let action = upKeyMap[e.code];
				if (action) {
					action();
				}
			}
		});
		document.getElementById("control-up").addEventListener("click", e => {
			this.moveOnce(NORTH);
		});
		document.getElementById("control-left").addEventListener("click", e => {
			this.moveOnce(WEST);
		});
		document.getElementById("control-right").addEventListener("click", e => {
			this.moveOnce(EAST);
		});
		document.getElementById("control-down").addEventListener("click", e => {
			this.moveOnce(SOUTH);
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

	moveOnce(direction) {
		this.sendInput({move: direction});
	}

	startMoving(direction) {
		this.sendInput({movement: direction});
		this.direction = direction;
	}

	stopMoving(direction) {
		if (direction == this.direction) {
			this.stop();
		}
	}

	stop() {
		this.direction = null;
		this.sendInput("stop");
	}

	act(direction) {
		this.sendInput(this.actionBar.selectedAction(direction));
	}

	sendInput(input) {
		this.send({input: input});
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
		this.model.setTime(m.t);
		if (m.viewarea) {
			this.readyToDraw = true;
			this.display.setViewArea(m.viewarea.area);
		}
		if (m.section) {
			this.display.drawSection(m.section.area.w, m.section.area.h, m.section.area.x, m.section.area.y, m.section.field, m.section.mapping);
		}
		if (m.changecells) {
			this.display.changeTiles(m.changecells);
		}
		if (m.dynamics) {
			this.model.setEntities(m.dynamics);
		}
		if (m.playerpos) {
			this.model.setCenter(m.playerpos);
			document.getElementById("coordinates").textContent = `${m.playerpos.pos[0]}, ${m.playerpos.pos[1]}`;
		}
		if (m.inventory) {
			this.actionBar.setInventory(m.inventory[0]);
		}
		if (m.sounds) {
			for (let sound of m.sounds) {
				this.print(sound[1], sound[0]);
			}
		}
	}

	sendRaw(msg) {
		if (this.websocket.readyState === WebSocket.OPEN){
			this.websocket.send(msg);
		} else {
			console.error("can't send message: websocket not open", this.websocket.readyState,  msg);
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
		this.send({chat: command});
	}

	resize() {
		this.zooms = this.zooms || 0
		this.zooms += 1
		this.display.resize(window.innerWidth, window.innerHeight);
	}

	draw() {
		let [cx, cy] = this.model.currentCenter();
		this.display.setCenter(cx, cy);
		this.display.drawDynamics(this.model.currentEntities());
		this.display.redraw();
	}

	update(t, duration) {
		this.model.stepTime(duration / 1000 * this.fps);
		if (this.readyToDraw) {
			this.draw();
		}
		if (this.keepRunning) {
			requestAnimationFrame(newTime => this.update(newTime, newTime - t));
		}
	}
}
