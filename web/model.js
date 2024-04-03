"use strict";


class Model {

	constructor() {
		this.entities = {};
		this.tick = 0;
		this.position = {pos: [0, 0]};
	}

	setTime(tick) {
		this.tick = tick;
	}

	stepTime(difference) {
		this.tick += difference;
	}

	setEntities(dynamics) {
		this.entities = dynamics;
	}

	setCenter(position) {
		this.position = position;
	}

	currentEntities() {
		return Object.entries(this.entities).map(([id, entity]) => {
			let [x, y] = entity.p;
			if (entity.a && entity.a.M && this.tick < entity.a.e) {
				let start = entity.a.s;
				let progress = (this.tick - start) / (entity.a.e - start);
				x += (entity.a.M[0] - x) * (1 - progress);
				y += (entity.a.M[1] - y) * (1 - progress);
			}
			return {x: x, y: y, sprite: entity.s, health: entity.h, maxHealth: entity.hh};
		});
	}


	currentCenter() {
		let [cx, cy] = this.position.pos;
		if (this.position.activity && this.position.activity.M && this.tick < this.position.activity.e) {
			let start = this.position.activity.s;
			let progress = (this.tick - start) / (this.position.activity.e - start);
			cx += (this.position.activity.M[0] - cx) * (1-progress)
			cy += (this.position.activity.M[1] - cy) * (1-progress)
		}
		return [cx, cy];
	}
}
