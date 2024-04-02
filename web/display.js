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
}

class LayeredSprite {
	constructor(layers, border) {
		this.layers = layers;
		this.border = border;
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
				if (entry.layer == "ground") {
					layers.fuzz = fuzzTemplate.fuzz(mainSprite);
				}
			}
			this.sprites[name] = new LayeredSprite(layers, entry.border);
		}
	}

	sprite(name) {
		return this.sprites[name];
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

	fillRect(color, x, y, width, height) {
		this.ctx.fillStyle = color;
		this.ctx.fillRect((x - this.area.x) * this.resolution, (y - this.area.y) * this.resolution, width * this.resolution, height * this.resolution);
	}

	fillTile(color, x, y) {
		this.drawRect(color, x, y, 1, 1);
	}

	clearTile(x, y) {
		this.ctx.clearRect((x - this.area.x) * this.resolution, (y - this.area.y) * this.resolution, this.resolution, this.resolution);
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
}

class Layer {
	constructor(name, opts) {
		opts = opts || {};
		this.name = name;
		this.clear = opts.clear|| ClearMode.Tile;
		this.offset = opts.offset || [0, 0];
		this.trueScale = opts.trueScale || false;
	}

	clearMode() {
		return this.clear;
	}

	offsetX() {
		return this.offset[0];
	}

	offsetY() {
		return this.offset[1];
	}

}

class Display {
	tileSize = 8;

	constructor(canvas, spritemap, fuzzSprite) {
		this.canvas = canvas;
		this.outerCtx = canvas.getContext("2d");
		this.layers = [
			new Layer("ground"),
			new Layer("fuzz", {clear: ClearMode.None}),
			new Layer("base"),
			new Layer("borders", {clear: ClearMode.None}),
			new Layer("main"),
			new Layer("creatures", {clear: ClearMode.None, trueScale: true}),
			new Layer("effect", {clear: ClearMode.None, trueScale: true}),
			new Layer("wol", {offset: [-1, 0]}),
			new Layer("wom", {offset: [0, 0]}),
			new Layer("wor", {offset: [1, 0]}),
			new Layer("hol", {offset: [-1, -1]}),
			new Layer("hom", {offset: [0, -1]}),
			new Layer("hor", {offset: [1, -1]}),
		];
		this.buffers = {};
		this.spritemap = spritemap;
		this.offsetX = 0;
		this.offsetY = 0;
		this.centerX = 0;
		this.centerY = 0;
		this.borders = new Map();
		this.width = 0;
		this.height = 0;
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
			let buffer = new DrawBuffer(area, resolution);
			if (this.buffers[layer.name]) {
				buffer.drawBuffer(this.buffers[layer.name]);
			}
			this.buffers[layer.name] = buffer;
		}
		this.offsetX = area.x;
		this.offsetY = area.y;
		this.width = area.w;
		this.height = area.h;
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
	}

	drawSection(width, height, offsetX, offsetY, cells, mapping){
		let borderMap = {};
		for (let key in mapping) {
			borderMap[key] = this._border(mapping[key]);
		}
		for (let i=0; i<width * height; ++i){
			let x = (i % width) + offsetX;
			let y = (i / width | 0) + offsetY;
			this._drawTile(x, y, mapping[cells[i]]);
			this.borders.set(hashpos(x, y), borderMap[cells[i]]);
		}
		for (let x=offsetX-1; x<width+offsetX+1; ++x) {
			for (let y=offsetY-1; y<height+offsetY+1; ++y) {
				this._drawBorder(x, y);
			}
		}
		this.init = true
	}

	changeTiles(tiles) {
		if (!this.init) {
			return;
		}
		for (let tile of tiles){
			let x = tile[0][0];
			let y = tile[0][1];
			let sprites = tile[1];
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
		this.buffers.creatures.clear();
		this.buffers.effect.clear();
		for (let entity of entities) {
			this._drawSprite(entity.sprite, entity.x, entity.y);
			this._drawHealthBar(entity.health, entity.maxHealth, entity.x, entity.y);
		}
	}

	_drawSprite(spritename, x, y) {
		let sprite = this.spritemap.sprite(spritename);
		if (sprite) {
			for (let layer in sprite.layers) {
				this.buffers[layer].drawSprite(sprite.layers[layer], x, y);
			}
		} else {
			this.buffers.base.fillTile(this._getColor(name), x, y);
		}
	}

	_drawHealthBar(health, maxHealth, x, y) {
		if (health === maxHealth) {
			return;
		}
		let ratio = health / maxHealth;
		let height = 1/8;
		let offset = 1/8;
		this.buffers.effect.fillRect("#0f0", x, y-height-offset, ratio, height);
		this.buffers.effect.fillRect("#c00", x+ratio, y-height-offset, 1-ratio, height);
	}

	_drawTile(tileX, tileY, sprites) {
		for (let layer of this.layers) {
			if (layer.clearMode() === ClearMode.Tile) {
				this.buffers[layer.name].clearTile(tileX, tileY)
			}
		}
		this.buffers.fuzz.drawBehind(buffer => buffer.drawSprite(this.fuzzSprite, tileX, tileY));
		for (let i=sprites.length; i --> 0;) {
			let name = sprites[i];
			this._drawSprite(name, tileX, tileY);
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
		this.centerX = x;
		this.centerY = y;
	}

	_getColor(name){
		var hash = 583;
		for (let i=0; i<name.length; ++i) {
			hash *= 7;
			hash += name.charCodeAt(i);
			hash %= 256 * 256 * 256;
		}
		return "#" + hash.toString(16);
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

	redraw(){
		if (!this.init) {
			return;
		}
		let tileSize = this.tileSize * this.scale;
		let centerX = (this.centerX - this.offsetX) * tileSize;
		let centerY = (this.centerY - this.offsetY) * tileSize;
		this.outerCtx.imageSmoothingEnabled = false;
		for (let layer of this.layers) {
			let buffer = this.buffers[layer.name];
			this.outerCtx.drawImage(
				buffer.canvas,
				this.canvas.width / 2 - centerX + layer.offsetX() * tileSize,
				this.canvas.height / 2 - centerY + layer.offsetY() * tileSize,
				buffer.canvas.width * tileSize / buffer.resolution,
				buffer.canvas.height * tileSize / buffer.resolution
			);
		}
	}

	resize(width, height) {
		this.canvas.width = width;;
		this.canvas.height = height;
		this.redraw();
	}
}
