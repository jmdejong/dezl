"use strict";

const BLOCKING = 1;
const DIAGONAL_COST = 1.99;

class GameMap {


	constructor() {
		this.grid = new GridU32();
	}

	blocking(pos) {
		return (this.grid.getVal(pos) & BLOCKING) !== 0;
	}

	path(from, to) {
		if (!this.grid.area.contains(to)) {
			return null;
		}
		let visited = new GridU32(this.grid.area);
		let fringe = new PriorityQueue(node => node.cost + node.pos.distanceTo(to));
		fringe.add({pos: from, path: [], cost: 0});
		let steps = 0;
		while (fringe.heap.length) {
			++steps;
			let node = fringe.remove();
			if (node.pos.equals(to)) {
				console.log("steps", steps);
				console.log("path", node);
				return node.path;
			}
			if (visited.getVal(node.pos) === 1) {
				continue;
			}
			visited.setVal(node.pos, 1);
			for (let d of [vec2(0,-1), vec2(-1,0), vec2(0,1), vec2(1,0)]) {
				let pos = node.pos.add(d);
				if (this.grid.area.contains(pos) && !this.blocking(pos) && visited.getVal(pos) === 0) {
					fringe.add({pos: pos, path: node.path.concat(pos), cost: node.cost + 1});
				}
			}
			for (let d of [vec2(-1,-1), vec2(-1,1), vec2(1,-1), vec2(1,1)]) {
				let pos = node.pos.add(d);
				if (this.grid.area.contains(pos) && !this.blocking(pos) && !this.blocking(pos.sub(vec2(d.x, 0))) && !this.blocking(pos.sub(vec2(0, d.y))) && visited.getVal(pos) === 0) {
					fringe.add({pos: pos, path: node.path.concat(pos), cost: node.cost + 1.99});
				}
			}
		}
		console.log("path failed");
		return null;
	}

	jumpPath(from, to) {
		if (!this.grid.area.contains(to)) {
			return null;
		}
		let visited = new GridU32(this.grid.area);
		let fringe = new PriorityQueue(node => node.cost + node.pos.distanceTo(to));
		fringe.add({pos: from, path: [], cost: 0, directions: [vec2(0,-1), vec2(-1,0), vec2(0,1), vec2(1,0), vec2(-1,-1), vec2(-1,1), vec2(1,-1), vec2(1,1)]});
		let a = pos => this.grid.area.contains(pos) && !this.blocking(pos);
		let steps=0;
		while (fringe.heap.length) {
			steps++;
			let node = fringe.remove();
			if (node.pos.equals(to)) {
				console.log("steps", steps);
				console.log("path", node);
				return node.path;
			}
			if (visited.getVal(node.pos) === 1) {
				continue;
			}
			visited.setVal(node.pos, 1);
			let points = [];
			for (let d of node.directions) {
				if (d.mLength() === 1) {
					let point = this.jumpOrthagonal(node.pos, d, a, to);
					if (point) {
						points.push(point);
					}
				} else if (d.mLength() === 2) {
					let point = this.jumpDiagonal(node.pos, d, a, to);
					if (point) {
						points.push(point);
					}
				} else {
					console.error("Unknown direction ", d);
				}
			}
			for (let point of points) {
				if (!point.pos) {
					console.error(point);
				}
				fringe.add({pos: point.pos, path: node.path.concat(point.pos), cost: node.cost + point.cost, directions: point.directions});
			}
		}
		console.log("path failed");
		return null;
	}

	jumpOrthagonal(start, d, a, target) {
		let dl = vec2(d.y, d.x);
		let dr = vec2(-d.y, -d.x);
		let oldPos = start;
		let pos = oldPos.add(d);
		let cost = 1;
		while (a(pos)) {
			if (pos.equals(target)) {
				return {pos: pos, cost: cost, directions: []};
			}
			let directions = [d];
			if (a(pos.add(dl)) && !a(oldPos.add(dl))) {
				directions.push(dl, d.add(dl));
			}
			if (a(pos.add(dr)) && !a(oldPos.add(dr))) {
				directions.push(dr, d.add(dr));
			}
			if (directions.length !== 1) {
				return {pos: pos, cost: cost, directions: directions};
			}

			oldPos = pos;
			pos = oldPos.add(d);
			++cost;
		}
		return null;
	}

	jumpDiagonal(start, d, a, target) {
		let pos = start.add(d);
		let cost = DIAGONAL_COST;
		let dl = vec2(d.x, 0);
		let dr = vec2(0, d.y);
		while (a(pos) && a(pos.sub(dl)) && a(pos.sub(dr))) {
			if (pos.equals(target)) {
				return {pos: pos, cost: cost, directions: []};
			}
			if (this.jumpOrthagonal(pos, dl, a, target) || this.jumpOrthagonal(pos, dr, a, target)) {
				return {pos: pos, cost: cost, directions: [d, dl, dr]};
			}

			cost += DIAGONAL_COST;
			pos = pos.add(d);
		}
		return null;
	}

	setArea(area) {
		let oldGrid = this.grid;
		this.grid = new GridU32(area);
		this.grid.copyFrom(oldGrid);
	}

	setSection(area, field, mapping) {
		let flags = mapping.map(t => this.parseTile(t));
		area.forEach((pos, index) => {
			this.grid.setVal(pos, flags[field[index]])
		});
	}

	setTiles(tiles) {
		for (let tile of tiles) {
			let pos = vec2(...tile[0]);
			this.grid.setVal(pos, this.parseTile(tile[1]));
		}
	}

	parseTile(sprites) {
		return BLOCKING * (sprites.indexOf("!b") >= 0);
	}
}
