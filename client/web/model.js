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
			if (entity.m && this.tick < entity.m.e) {
				let start = entity.m.s;
				let progress = (this.tick - start) / (entity.m.e - start);
				x += (entity.m.f[0] - x) * (1 - progress);
				y += (entity.m.f[1] - y) * (1 - progress);
			}
			return {x: x, y: y, sprite: entity.s};
		});
	}


	currentCenter() {
		let [cx, cy] = this.position.pos;
		if (this.position.movement && this.tick < this.position.movement.e) {
			let start = this.position.movement.s;
			let progress = (this.tick - start) / (this.position.movement.e - start);
			cx += (this.position.movement.f[0] - cx) * (1-progress)
			cy += (this.position.movement.f[1] - cy) * (1-progress)
		}
		return [cx, cy];
	}
}
