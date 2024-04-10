"use strict";

function clamp(v, lo, hi) {
	return Math.max(lo, Math.min(hi, v));
}


class Activity {

	constructor(start, end) {
		this.start = start;
		this.end = end;
		this.duration = end - start;
		this.id = start;
		this.started = false;
	}

	doStart(time) {
		if (this.started) {
			return;
		}
		this.started = true;
		let delay = time - this.start
		if (delay <= 0) {
			return;
		}
		this.start = time;
		this.duration = Math.max(this.duration/2, this.duration - delay);
		this.end = this.start + this.duration;
	}


	static parse(a, pos) {
		if (!a) {
			return null;
		} else if (a.M) {
			return new WalkActivity(a.s, a.e, vec2(...a.M), pos);
		} else if (a.F) {
			return new FightActivity(a.s, a.e, vec2(...a.F.t));
		} else if (a.D) {
			return new DieActivity(a.s, a.e);
		} else {
			console.error("Unknown activity", a);
			return null;
		}
	}

	isFinished(time) {
		return time >= this.end;
	}
	progress(time, max) {
		return clamp((time - this.start) / Math.min(this.duration, max || Infinity), 0, 1);
	}
	corePosition(time, pos) {
		return pos;
	}
	currentPosition(time, pos) {
		return this.corePosition(time, pos);
	}
	opacity(time) {
		return 1;
	}
}
class NoActivity extends Activity {
	constructor() {
		super(-1, 0);
	}
	isFinished(_time) {
		return true;
	}
	progress(_time) {
		return 1;
	}
}
class WalkActivity extends Activity {
	constructor(start, end, origin, to) {
		super(start, end);
		this.origin = origin;
		this.to = to
	}
	corePosition(time) {
		return this.origin.lerp(this.to, this.progress(time));
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
	constructor(id, pos, sprite, activity, health, wounds, blocking, previous){
		this.id = id;
		this.pos = pos;
		this.sprite = sprite;
		this.activities = []
		if (previous) {
			this.activities = previous.activities.concat();
		}
		if (activity && !this.activities.some(a => a.id === activity.id)) {
			this.activities.push(activity);
		}
		this.health = health;
		this.wounds = wounds;
		this.blocking = blocking;
	}

	static parse(e, previous) {
		let pos = vec2(...e.p);
		let wounds = (e.w || []).map(wound => ({damage: wound.d, time: wound.t, rind: wound.r}));
		return new Creature(e.i, pos, e.s, Activity.parse(e.a, pos), e.h, wounds, e.b, previous);
	}

	activity(time) {
		while (this.activities.length > 1 && this.activities[0].isFinished(time)) {
			this.activities.shift();
		}
		if (this.activities.length) {
			this.activities[0].doStart(time);
			return this.activities[0];
		} else {
			return new NoActivity();
		}
	}


	snapshot(time) {
		let wounds = this.wounds.map(wound => ({damage: wound.damage, age: time - wound.time, rind: wound.rind}));
		return {
			pos: this.activity(time).currentPosition(time, this.pos),
			sprite: this.sprite,
			health: this.health,
			opacity: this.activity(time).opacity(time),
			wounds
		};
	}

	corePosition(time) {
		return this.activity(time).corePosition(time, this.pos);
	}

	isPlayer() {
		this.id[0] === "p";
	}
}

class Model {

	constructor() {
		this.entities = [];
		this.tick = 0;
		this.shownTick = 0;
		this.me = null;
	}

	setTime(tick) {
		this.tick = tick;
		if (Number.isNaN(this.shownTick) || this.shownTick < tick - 10 || this.shownTick > tick + 10) {
			console.log("time jump", tick, this.shownTick);
			this.shownTick = tick;
		}
	}

	stepTime(difference) {
		this.tick += difference;
		this.shownTick += difference;
		if (this.shownTick > this.tick) {
			this.shownTick = Math.max(this.tick, this.shownTick - difference / 4);
		} else if (this.shownTick <= this.tick) {
			this.shownTick = Math.min(this.tick, this.shownTick + difference / 32);
		}
	}

	setEntities(entities) {
		let oldEntities = {};
		for (let entity of this.entities) {
			oldEntities[entity.id] = entity;
		}
		this.entities = entities.map(e => Creature.parse(e, oldEntities[e.i]));
		this.entities.sort((a, b) => a.pos.y - b.pos.y || a.isPlayer() - b.isPlayer() || (a.id === this.me.id) - (b.id === this.me.id));
	}

	setMe(rawMe) {
		let me = Creature.parse(rawMe, this.me);
		this.me = me;
	}

	currentEntities() {
		return this.entities.map(entity => {
			let s = entity.snapshot(this.shownTick);
			return s;
		});
	}

	currentCenter() {
		if (this.me) {
			let cp = this.me.corePosition(this.shownTick);
			return cp;
		} else {
			return vec2(0, 0);
		}
	}
}
