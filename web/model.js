"use strict";

function clamp(v, lo, hi) {
	return Math.max(lo, Math.min(hi, v));
}


class Activity {

	constructor(start, end) {
		this.start = start;
		this.end = end;
		this.duration = end - start;
	}

	static parse(a) {
		if (!a) {
			return new NoActivity();
		} else if (a.M) {
			return new WalkActivity(a.s, a.e, vec2(...a.M));
		} else if (a.F) {
			return new FightActivity(a.s, a.e, vec2(...a.F.t));
		} else if (a.D) {
			return new DieActivity(a.s, a.e);
		} else {
			console.error("Unknown activity", a);
			return new NoActivity();
		}
	}

	isActive(time) {
		return time >= this.start && time <= this.end;
	}
	progress(time, max) {
		return clamp((time - this.start) / Math.min(this.duration, max || Infinity), 0, 1);
	}
	currentPosition(time, pos) {
		return pos;
	}
	corePosition(time, pos) {
		return pos;
	}
	opacity(time) {
		return 1;
	}
}
class NoActivity extends Activity {
	isActive(_time) {
		return false;
	}
	progress(_time) {
		return 1;
	}
}
class WalkActivity extends Activity {
	constructor(start, end, origin) {
		super(start, end);
		this.origin = origin;
	}
	currentPosition(time, pos) {
		return this.origin.lerp(pos, this.progress(time));
	}
	corePosition(time, pos) {
		return this.currentPosition(time, pos);
	}
}
class FightActivity extends Activity {
	constructor(start, end, target) {
		super(start, end);
		this.target = target;
	}
	currentPosition(time, pos) {;
		let d = Math.max(0, 0.5 - Math.abs(this.progress(time, 5)-0.5));
		return pos.lerp(this.target, d);
	}
}
class DieActivity extends Activity {
	opacity(time) {
		return 1 - this.progress(time, 10);
	}
}

class Creature {
	constructor(id, pos, sprite, activity, health, wounds, blocking){
		this.id = id;
		this.pos = pos;
		this.sprite = sprite
		this.activity = activity;
		this.health = health;
		this.wounds = wounds;
		this.blocking = blocking;
	}

	static parse(e) {
		let wounds = (e.w || []).map(wound => ({damage: wound.d, time: wound.t, rind: wound.r}));
		return new Creature(e.i, vec2(...e.p), e.s, Activity.parse(e.a), e.h, wounds, e.b);
	}

	snapshot(time) {
		let wounds = this.wounds.map(wound => ({damage: wound.damage, age: time - wound.time, rind: wound.rind}));
		return {
			pos: this.activity.currentPosition(time, this.pos),
			sprite: this.sprite,
			health: this.health,
			opacity: this.activity.opacity(time),
			wounds
		};
	}

	corePosition(time) {
		return this.activity.corePosition(time, this.pos);
	}

	isPlayer() {
		this.id[0] === "p";
	}
}

class Model {

	constructor() {
		this.entities = [];
		this.tick = 0;
		this.me = {p: [0, 0]};
	}

	setTime(tick) {
		this.tick = tick;
	}

	stepTime(difference) {
		this.tick += difference;
	}

	setEntities(entities) {
		this.entities = entities.map(Creature.parse);
		this.entities.sort((a, b) => a.pos.y - b.pos.y || a.isPlayer() - b.isPlayer() || (a.id === this.me.id) - (b.id === this.me.id));
	}

	setMe(me) {
		this.me = Creature.parse(me);
	}

	currentEntities() {
		return this.entities.map(entity => entity.snapshot(this.tick));
	}

	currentCenter() {
		return this.me.corePosition(this.tick);
	}
}
