"use strict";

const BLOCKING = 1;


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
		while (fringe.heap.length) {
			let node = fringe.remove();
			if (node.pos.equals(to)) {
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
