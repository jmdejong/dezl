"use strict";

const ClearMode = {
	Tile: "Tile",
	None: "None"
}

class Sprite {
	constructor(image, x, y, width, height, area) {
		this.image = image;
		this.x = x || 0;
		this.y = y || 0;
		this.width = width || image.width;
		this.height = height || image.height;
		this.area = area || {
			x: 0,
			y: 0,
			w: 1,
			h: 1,
		};
	}

	drawOn(ctx, x, y) {
		ctx.drawImage(this.image, this.x, this.y, this.width, this.height, x, y, this.width, this.height);
	}

	getImage(resolution) {
		let canvas = document.createElement("canvas");
		canvas.width = this.width * resolution;
		canvas.height = this.height * resolution;
		let ctx = canvas.getContext("2d");
		ctx.drawImage(this.image, this.x, this.y, this.width, this.height, 0, 0, this.width * resolution, this.height * resolution);
	}
}

class LayeredSprite {
	constructor(name, layers, border, big, x, y) {
		this.name = name;
		this.layers = layers;
		this.border = border;
		this.big = big
		this.x = x;
		this.y = y;
	}
}


class SpriteMap {
	constructor() {
		this.sprites = {};
	}
	
	addSprites(image, mapping, size, fuzzTemplate) {
		for (let name in mapping) {
			let entry = mapping[name];
			let layers = {};
			if (entry.wide) {
				layers.hol = new Sprite(image, (entry.x - 1) * size, (entry.y - 1) * size, size, size);
				layers.hom = new Sprite(image, entry.x * size, (entry.y - 1) * size, size, size);
				layers.hor = new Sprite(image, (entry.x + 1) * size, (entry.y - 1) * size, size, size);
				layers.wol = new Sprite(image, (entry.x - 1) * size, entry.y * size, size, size);
				layers.wom = new Sprite(image, entry.x * size, entry.y * size, size, size);
				layers.wor = new Sprite(image, (entry.x + 1) * size, entry.y * size, size, size);
			} else {
				let mainSprite = new Sprite(image, entry.x * size, entry.y * size, size, size)
				layers[entry.layer || "main"] = mainSprite;
				if (entry.layer === "ground") {
					layers.fuzz = fuzzTemplate.fuzz(mainSprite);
				}
			}
			this.sprites[name] = new LayeredSprite(name, layers, entry.border, entry.wide, entry.x, entry.y);
		}
	}

	sprite(name) {
		return this.sprites[name];
	}

	all() {
		return Object.values(this.sprites);
	}
}


function hashpos(x, y) {
	return x + "," + y;
}

class DrawBuffer {

	constructor(area, resolution) {
		this.canvas = document.createElement("canvas");
		this.canvas.width = area.w * resolution;
		this.canvas.height = area.h * resolution;
		this.resolution = resolution;
		this.area = area;
		this.ctx = this.canvas.getContext("2d");
		this.ctx.imageSmoothingEnabled = false;
	}


	drawSprite(sprite, x, y) {
		x = Math.round((x - this.area.x) * this.resolution);
		y = Math.round((y - this.area.y) * this.resolution);
		this.ctx.drawImage(
			sprite.image,
			sprite.x,
			sprite.y,
			sprite.width,
			sprite.height,
			x + sprite.area.x * this.resolution,
			y + sprite.area.x * this.resolution,
			this.resolution * sprite.area.w,
			this.resolution * sprite.area.h
		);
	}

	drawBehind(drawFn) {
		this.ctx.globalCompositeOperation = "destination-out";
		drawFn(this);
		this.ctx.globalCompositeOperation = "source-over";
	}

	drawBuffer(buffer) {
		// todo: what if resolution is different
		this.ctx.drawImage(buffer.canvas, (buffer.area.x - this.area.x) * this.resolution, (buffer.area.y - this.area.y) * this.resolution);
	}

	clear() {
		this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);
	}

	fromWorld(pos) {
		return pos.sub(this.area.origin()).mul(this.resolution)
	}

	text(text, pos, color, outline, width) {
		this.ctx.fillStyle = color;
		this.ctx.strokeStyle = outline;
		this.ctx.textAlign = "center";
		this.ctx.font = "16px mono condensed";
		let bpos = this.fromWorld(pos);
		this.ctx.fillText(text, bpos.x, bpos.y, width);
		this.ctx.strokeText(text, bpos.x, bpos.y, width);
	}
	fillRect(color, area) {
		this.ctx.fillStyle = color;
		let a = this.fromWorld(area);
		this.ctx.fillRect(a.x, a.y, a.w, a.h);
	}

	clearRect(area) {
		let a = this.fromWorld(area);
		this.ctx.clearRect(a.x, a.y, a.w, a.h);
	}

	clearTile(x, y) {
		this.clearRect(new Area(x, y, 1, 1));
	}

	drawBorders(color, x, y, edges, width) {
		let px = (x - this.area.x) * this.resolution;
		let py = (y - this.area.y) * this.resolution;
		this.ctx.strokeStyle = color;
		this.ctx.lineWidth = width * this.resolution;
		let off = width * this.resolution / 2;
		if (edges.left) {
			this.ctx.beginPath();
			this.ctx.moveTo(px+off, py);
			this.ctx.lineTo(px+off, py + this.resolution);
			this.ctx.stroke();
		}
		if (edges.top) {
			this.ctx.beginPath();
			this.ctx.moveTo(px, py+off);
			this.ctx.lineTo(px + this.resolution, py+off);
			this.ctx.stroke();
		}
		if (edges.right) {
			this.ctx.beginPath();
			this.ctx.moveTo(px + this.resolution-off, py);
			this.ctx.lineTo(px + this.resolution-off, py + this.resolution);
			this.ctx.stroke();
		}
		if (edges.bottom) {
			this.ctx.beginPath();
			this.ctx.moveTo(px, py + this.resolution-off);
			this.ctx.lineTo(px + this.resolution, py + this.resolution-off);
			this.ctx.stroke();
		}
		this.ctx.stroke();
		this.ctx.lineWidth = 1;
	}

	move(area) {
		this.area = Area.fromCorners(area.origin().floor(), area.max().ceil());
		// console.log(this.area);
		this.canvas.width = this.area.w * this.resolution;
		this.canvas.height = this.area.h * this.resolution;
		this.ctx.imageSmoothingEnabled = false;
	}
}

class Layer {
	constructor(name, opts) {
		opts = opts || {};
		this.name = name;
		this.clear = opts.clear|| ClearMode.Tile;
		this.offset = vec2(...(opts.offset || [0, 0]));
		this.trueScale = opts.trueScale || false;
	}

	clearMode() {
		return this.clear;
	}

}

class Display {
	tileSize = 8;

	constructor(canvas, spritemap, fuzzSprite) {
		this.canvas = canvas;
		this.outerCtx = canvas.getContext("2d");
		let groundOffset = [0, 1/this.tileSize]
		this.layers = [
			new Layer("ground", {offset: groundOffset}),
			new Layer("fuzz", {offset: groundOffset, clear: ClearMode.None}),
			new Layer("base", {offset: groundOffset}),
			new Layer("borders", {offset: groundOffset, clear: ClearMode.None}),
			new Layer("main"),
			new Layer("creatures", {clear: ClearMode.None, trueScale: true}),
			new Layer("wol", {offset: [-1, 0]}),
			new Layer("wom", {offset: [0, 0]}),
			new Layer("wor", {offset: [1, 0]}),
			new Layer("effect", {clear: ClearMode.None, trueScale: true}),
			new Layer("hol", {offset: [-1, -1]}),
			new Layer("hom", {offset: [0, -1]}),
			new Layer("hor", {offset: [1, -1]}),
		];
		this.buffers = {};
		this.spritemap = spritemap;
		this.area = new Area(0, 0, 0, 0);
		this.center = vec2(0, 0);
		this.borders = new Map();
		this.scale = 4;
		this.init = false;
		this.fuzzSprite = fuzzSprite;
	}

	setViewArea(area){
		for (let layer of this.layers) {
			let resolution = this.tileSize;
			if (layer.trueScale) {
				resolution *= this.scale;
			}
			let buffer = new DrawBuffer(area.grow(1), resolution);
			if (this.buffers[layer.name]) {
				buffer.drawBuffer(this.buffers[layer.name]);
			}
			this.buffers[layer.name] = buffer;
		}
		this.area = area;
		let minX = area.x - 1;
		let minY = area.y - 1;
		let maxX = area.x + area.w;
		let maxY = area.y + area.h;
		this.borders.forEach((border, key, map) => {
			let [x, y] = key.split(",").map(v => v|0)
			if (x < minX || y < minY || x > maxX || y > maxY) {
				map.delete(key);
			}
		});
		this.init = true
	}

	drawSection(area, cells, mapping){
		let borderMap = {};
		for (let key in mapping) {
			borderMap[key] = this._border(mapping[key]);
		}
		for (let layer of this.layers) {
			if (layer.clearMode() === ClearMode.Tile) {
				this.buffers[layer.name].clearRect(area);
			}
		}
		for (let i=0; i<area.w * area.h; ++i){
			let x = (i % area.w) + area.x;
			let y = (i / area.w | 0) + area.y;
			this._drawTile(x, y, mapping[cells[i]]);
			this.borders.set(hashpos(x, y), borderMap[cells[i]]);
		}
		area.grow(1).forEach(pos => this._drawBorder(pos.x, pos.y));
	}

	changeTiles(tiles) {
		if (!this.init) {
			return;
		}
		for (let tile of tiles){
			let x = tile[0][0];
			let y = tile[0][1];
			let sprites = tile[1];

			for (let layer of this.layers) {
				if (layer.clearMode() === ClearMode.Tile) {
					this.buffers[layer.name].clearTile(x, y)
				}
			}
			// this.buffers.fuzz.drawBehind(buffer => buffer.drawSprite(this.fuzzSprite, tileX, tileY));
			this._drawTile(x, y, sprites);
			let border = this._border(sprites);
			let p = hashpos(x, y);
			if (border !== this.borders.get(p)) {
				this.borders.set(p, border);
				this._drawBorder(x, y);
				this._drawBorder(x+1, y);
				this._drawBorder(x-1, y);
				this._drawBorder(x, y+1);
				this._drawBorder(x, y-1);
			}
		}
	}

	drawDynamics(entities) {
		// let visibleArea = this.screenToWorld(new Area(0, 0, this.canvas.width, this.canvas.height));
		// this.buffers.creatures.move(visibleArea);
		// this.buffers.effect.move(visibleArea);
		this.buffers.creatures.clear();
		this.buffers.effect.clear();
		for (let entity of entities) {
			this.drawSprite(entity.sprite, entity.x, entity.y);
			this._drawHealthBar(entity.health, entity.maxHealth, entity.x, entity.y);
			for (let wound of entity.wounds) {
				this._drawWound(wound.damage, wound.age, entity.x, entity.y, wound.rind);
			}
		}
	}

	drawSprite(spritename, x, y) {
		let sprite = this.spritemap.sprite(spritename);
		if (sprite) {
			for (let layer in sprite.layers) {
				this.buffers[layer].drawSprite(sprite.layers[layer], x, y);
			}
		} else {
			this.buffers.base.fillRect(this._getColor(spritename), new Area(x, y, 1, 1));
		}
	}

	_drawHealthBar(health, maxHealth, x, y) {
		if (health === maxHealth) {
			return;
		}
		let ratio = health / maxHealth;
		let height = 1/8;
		let offset = 1/8;
		let area = new Area(x, y-height-offset, 1, height);
		let [green, red] = area.divideY(ratio);
		this.buffers.effect.fillRect("#0f0", green);
		this.buffers.effect.fillRect("#c00", red);
	}

	_drawWound(damage, age, x, y, rind) {
		let rx = rind / 0x1_00_00_00_00;
		let ry = (rind % 0x1_00_00) / 0x1_00_00;
		this.buffers.effect.text(damage, vec2(x+0.3 + 0.4*rx, y+0.4 + 0.4*ry - age/20), "#f77", "#f00")
	}

	_drawTile(tileX, tileY, sprites) {
		for (let i=sprites.length; i --> 0;) {
			let name = sprites[i];
			this.drawSprite(name, tileX, tileY);
		}
	}

	_drawBorder(x, y) {
		this.buffers.borders.clearTile(x, y);
		let border = this._borderAt(x, y);
		if (border) {
			let edges = {
				left: this._borderAt(x - 1, y) !== border,
				right: this._borderAt(x + 1, y) !== border,
				top: this._borderAt(x, y - 1) !== border,
				bottom: this._borderAt(x, y + 1) !== border,
			};
			this.buffers.borders.drawBorders(border, x, y, edges, 1/this.tileSize);
		}
	}

	_borderAt(x, y) {
		return this.borders.get(hashpos(x, y));
	}

	setCenter(x, y) {
		this.center = vec2(x, y);
	}

	_getColor(name){
		var hash = 583;
		for (let i=0; i<name.length; ++i) {
			hash *= 37;
			hash += name.charCodeAt(i);
			hash %= 256 * 256 * 256;
		}
		let color = "#" + hash.toString(16);
		return color;
	}

	_border(spriteNames) {
		for (let spriteName of spriteNames) {
			let sprite = this.spritemap.sprite(spriteName);
			if (sprite && sprite.border) {
				return sprite.border;
			}
		}
		return null;
	}

	screenCenter() {
		return vec2(this.canvas.width, this.canvas.height).div(2).floor();
	}

	worldToScreen(pos) {
		return pos.sub(this.center).mul(this.tileSize * this.scale).add(this.screenCenter())
	}

	screenToWorld(spos) {
		return spos.sub(this.screenCenter()).div(this.tileSize * this.scale).add(this.center);
	}

	redraw(){
		if (!this.init) {
			return;
		}
		let tileSize = this.tileSize * this.scale;
		this.outerCtx.imageSmoothingEnabled = false;
		for (let layer of this.layers) {
			let buffer = this.buffers[layer.name];
			let bufferCanvasSize = vec2(buffer.canvas.width, buffer.canvas.height);
			let area = this.worldToScreen(buffer.area.add(layer.offset));
			this.outerCtx.drawImage(
				buffer.canvas,
				area.x,
				area.y,
				area.w,
				area.h
			);
			// let wa = this.screenToWorld(new Area(0, 0, this.canvas.width, this.canvas.height));
			// let ba = buffer.fromWorld(wa.sub(layer.offset));
			// this.outerCtx.drawImage(
			// 	buffer.canvas,
			// 	ba.x, ba.y, ba.w, ba.h,
			// 	0, 0, this.canvas.width, this.canvas.height
			// );
		}
	}

	resize(width, height) {
		this.canvas.width = width;;
		this.canvas.height = height;
		this.redraw();
	}
}
