"use strict";


class Model {

	constructor() {
		this.entities = {};
		this.tick = 0;
		this.me = {p: [0, 0]};
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

	setMe(me) {
		this.me = me;
	}

	currentEntities() {
		return Object.values(this.entities).map(entity => {
			let pos = vec2(...entity.p);
			if (entity.a && this.tick < entity.a.e) {
				let start = entity.a.s;
				let progress = (this.tick - start) / (entity.a.e - start);
				if (entity.a.M) {
					let origin = vec2(...entity.a.M);
					pos = origin.lerp(pos, progress);
				} else if (entity.a.F) {
					let d = Math.max(0, 0.25 - Math.abs(progress-0.25))*2;
					let target = vec2(...entity.a.F.t)
					pos = pos.lerp(target, d);
				}
			}
			let wounds = entity.w.map(wound => ({damage: wound.d, age: this.tick - wound.t, rind: wound.r}));
			return {x: pos.x, y: pos.y, sprite: entity.s, health: entity.h, maxHealth: entity.hh, wounds};
		});
	}


	currentCenter() {
		let center = vec2(...this.me.p);
		if (this.me.a && this.me.a.M && this.tick < this.me.a.e) {
			let start = this.me.a.s;
			let progress = (this.tick - start) / (this.me.a.e - start);
			let origin = vec2(...this.me.a.M);
			center = origin.lerp(center, progress);
		}
		return [center.x, center.y];
	}
}
