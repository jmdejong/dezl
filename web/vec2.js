"use strict";

class Vec2 {

	constructor(x, y) {
		this.x = x;
		this.y = y;
	}

	surface() {
		return this.x * this.y;
	}

	length() {
		return Math.hypot(this.x, this.y);
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
		return this.div(this.length());
	}

	lerp(to, d) {
		return new Vec2(this.x*(1-d) + to.x*d, this.y*(1-d) + to.y*d);
	}

	clone() {
		return new Vec2(this.x, this.y);
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

	origin() {
		return new Vec2(this.x, this.y);
	}

	size() {
		return new Vec2(this.w, this.h);
	}

	max() {
		return new Vec2(this.x + this.w, this.y + this.h);
	}

	grow(s) {
		return new Area(this.x-s, this.y-s, this.w+s*2, this.h+s*2);
	}

	forEach(fn) {
		for (let x=this.x; x<this.x+this.w; ++x) {
			for (let y=this.y; y<this.y+this.h; ++y) {
				fn(new Vec2(x, y));
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
}

