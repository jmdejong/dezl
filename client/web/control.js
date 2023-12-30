"use strict";

const NORTH = "north";
const SOUTH = "south";
const EAST = "east";
const WEST = "west";

class Control {

	constructor(sender) {
		this.sender = sender;
		this.direction = null;
	}

	send(inp) {
		this.sender(JSON.stringify({input: inp}));
	}

	moveOnce(direction) {
		return () => this.send({move: direction});
	}

	startMoving(direction) {
		return () => {
			this.send({movement: direction});
			this.direction = direction;
		}
	}

	stopMoving(direction) {
		return () => {
			if (direction == this.direction) {
				this.stop();
			}
		}
	}

	stop() {
		this.direction = null;
		this.send("stop");
	}

	interact(direction) {
		return () => this.send({interact: direction});
	}

	selectNext() {
		return () => this.send({select: "next"});
	}

	selectPrevious() {
		return () => this.send({select:  "previous"});
	}

}


