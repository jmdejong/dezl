"use strict";


class Client {
	constructor(username, host, display, settings) {
		this.username = username;
		this.host = host;
		this.display = display;
		this.websocket = null;
		this.fps = 10;
		this.keepRunning = true;
		let sender = msg => this.send(msg);
		let delay = settings.delay|0;
		if (delay) {
			sender = msg => setTimeout((() => this.send(msg)), delay);
		}
		this.control = new Control(sender);
		this.model = new Model();
		this.readyToDraw = false;
	}

	start(){
		console.log("connecting to '" + this.host + "' as '" + this.username + "'");
		this.websocket = new WebSocket(this.host);
		this.websocket.addEventListener("open", e => {
			document.getElementById("game").hidden = false;
			e.target.send(JSON.stringify({introduction: this.username}));
		});
		let keymap = {
			KeyW: this.control.startMoving(NORTH),
			ArrowUp: this.control.startMoving(NORTH),
			KeyS: this.control.startMoving(SOUTH),
			ArrowDown: this.control.startMoving(SOUTH),
			KeyA: this.control.startMoving(WEST),
			ArrowLeft: this.control.startMoving(WEST),
			KeyD: this.control.startMoving(EAST),
			ArrowRight: this.control.startMoving(EAST),
			Period: this.control.selectNext(),
			Comma: this.control.selectPrevious(),
			NumpadAdd: this.control.selectNext(),
			NumpadSubtract: this.control.selectPrevious(),
			Equal: this.control.selectNext(),
			Minus: this.control.selectPrevious(),
		};
		let shiftKeymap = {
			KeyW: this.control.interact(NORTH),
			ArrowUp: this.control.interact(NORTH),
			KeyS: this.control.interact(SOUTH),
			ArrowDown: this.control.interact(SOUTH),
			KeyA: this.control.interact(WEST),
			ArrowLeft: this.control.interact(WEST),
			KeyD: this.control.interact(EAST),
			ArrowRight: this.control.interact(EAST),
		}
		document.addEventListener("keydown", e => {
			if (document.activeElement.classList.contains("captureinput")){
				if (e.code == "Escape") {
					document.activeElement.blur();
					this.control.stop();
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
			KeyW: this.control.stopMoving(NORTH),
			ArrowUp: this.control.stopMoving(NORTH),
			KeyS: this.control.stopMoving(SOUTH),
			ArrowDown: this.control.stopMoving(SOUTH),
			KeyA: this.control.stopMoving(WEST),
			ArrowLeft: this.control.stopMoving(WEST),
			KeyD: this.control.stopMoving(EAST),
			ArrowRight: this.control.stopMoving(EAST),
		}
		document.addEventListener("keyup", e => {
			if (document.activeElement.classList.contains("captureinput")){
				this.control.stop();
				return;
			} else {
				let action = upKeyMap[e.code];
				if (action) {
					action();
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

	send(msg) {
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
		this.websocket.send(JSON.stringify({chat: command}));
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
