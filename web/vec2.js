"use strict";

const NORTH = "north";
const SOUTH = "south";
const EAST = "east";
const WEST = "west";

class Vec2 {

	constructor(x, y) {
		this.x = x;
		this.y = y;
	}

	equals(other) {
		return this.x === other.x && this.y === other.y;
	}

	surface() {
		return this.x * this.y;
	}

	length() {
		return Math.hypot(this.x, this.y);
	}

	mLength() {
		return Math.abs(this.x) + Math.abs(this.y);
	}

	add(v) {
		return new Vec2(this.x + v.x, this.y + v.y);
	}

	sub(v) {
		return new Vec2(this.x - v.x, this.y - v.y);
	}

	mul(n) {
		return new Vec2(this.x * n, this.y * n);
	}

	div(n) {
		return new Vec2(this.x / n, this.y / n);
	}

	floor() {
		return new Vec2(Math.floor(this.x), Math.floor(this.y));
	}

	round() {
		return new Vec2(Math.round(this.x), Math.round(this.y));
	}

	ceil() {
		return new Vec2(Math.ceil(this.x), Math.ceil(this.y));
	}

	normalize() {
		let l = this.length();
		if (l !== 0) {
			return this.div(l);
		} else {
			return this;
		}
	}

	lerp(to, d) {
		return new Vec2(this.x*(1-d) + to.x*d, this.y*(1-d) + to.y*d);
	}

	distanceTo(other) {
		return this.sub(other).length();
	}

	mDistanceTo(other) {
		return this.sub(other).mLength();
	}

	clone() {
		return new Vec2(this.x, this.y);
	}

	arr() {
		return [this.x, this.y];
	}

	moved(direction) {
		if (direction === NORTH) {
			return new Vec2(this.x, this.y-1);
		} else if (direction === SOUTH) {
			return new Vec2(this.x, this.y+1);
		} else if (direction === EAST) {
			return new Vec2(this.x+1, this.y);
		} else if (direction === WEST) {
			return new Vec2(this.x-1, this.y);
		} else if (!direction) {
			return this;
		} else {
			console.error("invalid direction", direction);
		}
	}
}

function vec2(x, y) {
	return new Vec2(x, y);
}

class Area {
	constructor(x, y, w, h) {
		this.x = x;
		this.y = y;
		this.w = w;
		this.h = h;
	}

	static parse(o) {
		return new Area(o.x, o.y, o.w, o.h);
	}
	static fromVecs(pos, size) {
		return new Area(pos.x, pos.y, size.x, size.y);
	}
	static fromCorners(pos, max) {
		return new Area(pos.x, pos.y, max.x - pos.x, max.y - pos.y);
	}
	static centered(center, size) {
		return new Area(center.x - size.x / 2, center.y - size.y / 2, size.x, size.y);
	}

	origin() {
		return new Vec2(this.x, this.y);
	}

	size() {
		return new Vec2(this.w, this.h);
	}

	surface() {
		return this.w * this.h;
	}

	max() {
		return new Vec2(this.x + this.w, this.y + this.h);
	}

	center() {
		return new Vec2(this.x + this.w / 2, this.y + this.h / 2);
	}

	grow(s) {
		return new Area(this.x-s, this.y-s, this.w+s*2, this.h+s*2);
	}

	forEach(fn) {
		for (let x=this.x; x<this.x+this.w; ++x) {
			for (let y=this.y; y<this.y+this.h; ++y) {
				fn(new Vec2(x, y), (x - this.x) + (y - this.y) * this.w);
			}
		}
	}

	mul(m) {
		return new Area(this.x * m, this.y * m, this.w * m, this.h * m);
	}
	div(m) {
		return new Area(this.x / m, this.y / m, this.w / m, this.h / m);
	}

	add(v) {
		return new Area(this.x + v.x, this.y + v.y, this.w, this.h);
	}
	sub(v) {
		return new Area(this.x - v.x, this.y - v.y, this.w, this.h);
	}
	round() {
		return Area.fromCorners(this.origin().round(), this.max().round());
	}
	floor() {
		return Area.fromCorners(this.origin().floor(), this.max().floor());
	}
	intersection(other) {
		let x = Math.max(this.x, other.x);
		let y = Math.max(this.y, other.y);
		let x_ = Math.min(this.x + this.w, other.x + other.w);
		let y_ = Math.min(this.y + this.h, other.y + other.h);
		let w = Math.max(x_ - x, 0);
		let h = Math.max(y_ - y, 0);
		return new Area(x, y, w, h);
	}
	contains(pos) {
		return pos.x >= this.x && pos.y >= this.y && pos.x < this.x + this.w && pos.y < this.y + this.h;
	}
}

