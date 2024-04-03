"use strict";

class Vec2 {

	constructor(x, y) {
		this.x = x;
		this.y = y;
	}

	hash() {
		// return this.x + "," + this.y;
		return this.x + (1<<15) | (this.y + (1<<15)) <<16;
	}

	surface() {
		return this.x * this.y;
	}

	length() {
		return Math.hypot(this.x, this.y);
	}

	normalize() {
		return this.mult(1/this.length());
	}

	mult(n) {
		return vec2(this.x * n, this.y * n);
	}

	add(v) {
		return vec2(this.x + v.x, this.y + v.y);
	}

	sub(v) {
		return vec2(this.x - v.x, this.y - v.y);
	}

	lerp(to, d) {
		return this.mult(1-d).add(to.mult(d));
	}

	clone() {
		return vec2(this.x, this.y);
	}

}

function vec2(x, y) {
	return new Vec2(x, y);
}
