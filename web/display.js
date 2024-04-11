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

	constructor(canvas, area, resolution) {
		this.canvas = canvas;
		this.resolution = resolution;
		this.area = area;
		this.ctx = this.canvas.getContext("2d");
		this.ctx.imageSmoothingEnabled = false;
	}

	static create(area, resolution) {
		let canvas = document.createElement("canvas");
		canvas.width = area.w * resolution;
		canvas.height = area.h * resolution;
		return new DrawBuffer(canvas, area, resolution);
	}

	static centered(canvas, pos, resolution) {
		let area = Area.centered(pos, vec2(canvas.width / resolution, canvas.height / resolution));
		return new DrawBuffer(canvas, area, resolution);
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

	drawBuffer(buffer, offset) {
		offset = offset || vec2(0, 0);
		let overlap = this.area.intersection(buffer.area);
		let dest = this.fromWorld(overlap);
		let src = buffer.fromWorld(overlap.sub(offset));
		this.ctx.drawImage(buffer.canvas, src.x, src.y, src.w, src.h, dest.x, dest.y, dest.w, dest.h);
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
		let a = this.fromWorld(area).round();
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
		this.ctx.strokeStyle = "#" + color.toString(16);
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
		this.dynamic = opts.dynamic || false;
	}

	clearMode() {
		return this.clear;
	}

}

class Display {
	tileSize = 8;

	constructor(canvas, spritemap, fuzzSprite) {
		this.canvas = canvas;
		let groundOffset = [0, 1/this.tileSize]
		this.layers = [
			new Layer("ground", {offset: groundOffset}),
			new Layer("fuzz", {offset: groundOffset, clear: ClearMode.None}),
			new Layer("base", {offset: groundOffset}),
			new Layer("borders", {offset: groundOffset, clear: ClearMode.None}),
			new Layer("main"),
			new Layer("creatures", {clear: ClearMode.None, dynamic: true}),
			new Layer("wol", {offset: [-1, 0]}),
			new Layer("wom", {offset: [0, 0]}),
			new Layer("wor", {offset: [1, 0]}),
			new Layer("effect", {clear: ClearMode.None, dynamic: true}),
			new Layer("hol", {offset: [-1, -1]}),
			new Layer("hom", {offset: [0, -1]}),
			new Layer("hor", {offset: [1, -1]}),
		];
		this.buffers = {};
		this.spritemap = spritemap;
		this.area = new Area(0, 0, 0, 0);
		this.center = vec2(0, 0);
		this.borders = new GridU32(this.area)//new Map();
		this.scale = 4;
		this.init = false;
		this.fuzzSprite = fuzzSprite;
		this.entities = [];
	}

	setViewArea(area){
		for (let layer of this.layers) {
			if (layer.dynamic) {
				continue;
			}
			let resolution = this.tileSize;
			let buffer = DrawBuffer.create(area.grow(1), resolution);
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
		let borders = new GridU32(area);
		borders.copyFrom(this.borders);
		this.borders = borders;
		this.init = true;
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
			this.borders.put(vec2(x, y), borderMap[cells[i]]);
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
			if (border !== this.borders.getVal(p)) {
				this.borders.put(p, border);
				this._drawBorder(x, y);
				this._drawBorder(x+1, y);
				this._drawBorder(x-1, y);
				this._drawBorder(x, y+1);
				this._drawBorder(x, y-1);
			}
		}
	}

	drawCreatures(buffer, entities) {
		for (let entity of entities) {
			if (entity.opacity) {
				let sprite = this.spritemap.sprite(entity.sprite);
				if (!sprite) {
					sprite = this.spritemap.sprite("unknowncreature");
				}
				buffer.drawSprite(sprite.layers.creatures, entity.pos.x, entity.pos.y);
			}
		}
	}

	drawEffects(buffer, entities) {
		for (let entity of entities) {
			if (entity.opacity && entity.health[0] !== entity.health[1]) {
				this._drawHealthBar(buffer, entity.health[0], entity.health[1], entity.pos.x, entity.pos.y);
			}
			for (let wound of entity.wounds) {
				if (wound.age < 10){
					this._drawWound(buffer, wound.damage, wound.age, entity.pos, wound.rind);
				}
			}
		}
	}

	drawDynamics(entities) {
		this.entities = entities;
	}

	drawSprite(spritename, x, y) {
		let sprite = this.spritemap.sprite(spritename);
		if (!sprite) {
			sprite = this.spritemap.sprite("unknown");
		}
		for (let layer in sprite.layers) {
			this.buffers[layer].drawSprite(sprite.layers[layer], x, y);
		}
	}

	_drawHealthBar(buffer, health, maxHealth, x, y) {
		let ratio = health / maxHealth;
		let width = 1;
		let height = 1/8;
		let offset = 1/8;
		let ytop = y - height - offset;
		let res = buffer.resolution;
		let splitX = Math.round((ratio * width) * res) / res + x;
		let green = new Area(x, ytop, splitX - x, height);
		let red = new Area(splitX, ytop, x + width - splitX, height);
		buffer.fillRect("#0f0", green);
		buffer.fillRect("#c00", red);
	}

	_drawWound(buffer, damage, age, pos, rind) {
		let rx = rind / 0x1_00_00_00_00;
		let ry = (rind % 0x1_00_00) / 0x1_00_00;
		let roffset = vec2(0.3 + 0.4*rx, 0.4 + 0.4*ry - age/20);
		buffer.text(damage, pos.add(roffset), "#f77", "#f00");
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
		if (border >= 0 && border < 0x1_00_00_00) {
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
		return this.borders.getVal(vec2(x, y));
	}

	setCenter(pos) {
		this.center = pos;
	}
	setEntities(entities) {
		this.entities = entities;
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
		return -1;
	}

	screenCenter() {
		return vec2(this.canvas.width, this.canvas.height).div(2).floor();
	}

	viewport() {
		return this.screenToWorld(new Area(0, 0, this.canvas.width, this.canvas.height));
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
		let mainBuffer = DrawBuffer.centered(this.canvas, this.center, tileSize);
		for (let layer of this.layers) {
			if (layer.dynamic) {
				if (layer.name === "creatures") {
					this.drawCreatures(mainBuffer, this.entities);
				} else if (layer.name === "effect") {
					this.drawEffects(mainBuffer, this.entities);
				}
			} else {
				let buffer = this.buffers[layer.name];
				mainBuffer.drawBuffer(buffer, layer.offset);
			}
		}
	}

	resize(width, height) {
		this.canvas.width = width;;
		this.canvas.height = height;
		this.redraw();
	}
}
