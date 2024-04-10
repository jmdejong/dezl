

class GridU32 {

	constructor(area) {
		this.area = area;
		this.data = new Uint32Array(area.surface());
	}

	put(pos, val) {
		let x = pos.x - this.area.x;
		let y = pos.y - this.area.y;
		if (x >=0 && x < this.area.w && y >= 0 && y < this.area.h) {
			this.data[x + this.area.w * y] = val;
		}
	}

	getVal(pos) {
		let x = pos.x - this.area.x;
		let y = pos.y - this.area.y;
		if (x >=0 && x < this.area.w && y >= 0 && y < this.area.h) {
			return this.data[x + this.area.w * y];
		} else {
			return -1;
		}
	}

	copyFrom(other) {
		this.area.intersection(other.area).forEach(pos => this.put(pos, other.getVal(pos)));
	}
}
