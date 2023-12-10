"use strict";

class Sprite {
	constructor(image, x, y, width, height, originX, originY) {
		this.image = image;
		this.x = x || 0;
		this.y = y || 0;
		this.width = width || image.width;
		this.height = height || image.height;
		this.originX = originX || 0;
		this.originY = originY || 0;
	}

	drawOn(ctx, x, y) {
		ctx.drawImage(this.image, this.x, this.y, this.width, this.height, x - this.originX, y - this.originY, this.width, this.height);
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
			let mainSprite = new Sprite(image, entry.x * size, entry.y * size, size, size)
			layers[entry.layer || "main"] = mainSprite;
			if (entry.ho) {
				layers.ho = new Sprite(image, entry.x * size, (entry.y - 1) * size, size, size);
			}
			if (entry.layer == "ground") {
				layers.fuzz = fuzzTemplate.fuzz(mainSprite);
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


class Display {
	tileSize = 8;

	constructor(canvas, spritemap, fuzzSprite) {
		this.canvas = canvas;
		this.outerCtx = canvas.getContext("2d");
		this.layers = ["ground", "fuzz", "base", "borders", "main", "creatures", "ho"];
		this.buffers = {};
		this.ctxs = {};
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
			let buffer = document.createElement("canvas");
			buffer.width = area.w * this.tileSize;
			buffer.height = area.h * this.tileSize;
			let ctx = buffer.getContext("2d");
			if (this.buffers[layer]) {
				ctx.drawImage(this.buffers[layer], (this.offsetX - area.x) * this.tileSize, (this.offsetY - area.y) * this.tileSize);
			}
			this.buffers[layer] = buffer;
			this.ctxs[layer] = ctx;
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
		this.ctxs.creatures.clearRect(0, 0, this.width * this.tileSize, this.height * this.tileSize);
		for (let entity of entities) {
			let x = (entity.p[0] - this.offsetX) * this.tileSize;
			let y = (entity.p[1] - this.offsetY) * this.tileSize;
			this._drawSprite(entity.s, x|0, y|0);
		}
	}

	_drawSprite(spritename, x, y) {
		let sprite = this.spritemap.sprite(spritename);
		if (sprite) {
			for (let layer in sprite.layers) {
				sprite.layers[layer].drawOn(this.ctxs[layer], x, y);
			}
		} else {
			this.ctxs.base.fillStyle = this._getColor(name);
			this.ctxs.base.fillRect(x, y, this.tileSize, this.tileSize);
		}
	}

	_drawTile(tileX, tileY, sprites) {
		let x = (tileX - this.offsetX) * this.tileSize;
		let y = (tileY - this.offsetY) * this.tileSize;
		let hoY = y;// - this.tileSize;
		// this.ctxs.ground.clearRect(x, y, this.tileSize, this.tileSize);
		this.ctxs.ground.clearRect(x, y, this.tileSize, this.tileSize);
		this.ctxs.fuzz.globalCompositeOperation = "destination-out";
		this.fuzzSprite.drawOn(this.ctxs.fuzz, x, y);
		this.ctxs.fuzz.globalCompositeOperation = "source-over";
		this.ctxs.base.clearRect(x, y, this.tileSize, this.tileSize);
		this.ctxs.main.clearRect(x, y, this.tileSize, this.tileSize);
		this.ctxs.ho.clearRect(x, hoY, this.tileSize, this.tileSize);
		for (let i=sprites.length; i --> 0;) {
			let name = sprites[i];
			this._drawSprite(name, x, y);
		}
	}

	_drawBorder(x, y) {
		let lx = x - this.offsetX;
		let ly = y - this.offsetY;
		let px = lx * this.tileSize;
		let py = ly * this.tileSize;
		this.ctxs.borders.clearRect(px, py, this.tileSize, this.tileSize);
		let border = this._borderAt(x, y);
		if (!border) {
			return;
		}
		this.ctxs.borders.strokeStyle = border;
		if (this._borderAt(x - 1, y) !== border) {
			this.ctxs.borders.beginPath();
			this.ctxs.borders.moveTo(px+0.5, py);
			this.ctxs.borders.lineTo(px+0.5, py + this.tileSize);
			this.ctxs.borders.stroke();
			// console.log("drawing border", lx, ly, border);
		}
		if (this._borderAt(x, y - 1) !== border) {
			this.ctxs.borders.beginPath();
			this.ctxs.borders.moveTo(px, py+0.5);
			this.ctxs.borders.lineTo(px + this.tileSize, py+0.5);
			this.ctxs.borders.stroke();
		}
		if (this._borderAt(x + 1, y) !== border) {
			this.ctxs.borders.beginPath();
			this.ctxs.borders.moveTo(px + this.tileSize-0.5, py);
			this.ctxs.borders.lineTo(px + this.tileSize-0.5, py + this.tileSize);
			this.ctxs.borders.stroke();
		}
		if (this._borderAt(x, y + 1) !== border) {
			this.ctxs.borders.beginPath();
			this.ctxs.borders.moveTo(px, py + this.tileSize-0.5);
			this.ctxs.borders.lineTo(px + this.tileSize, py + this.tileSize-0.5);
			this.ctxs.borders.stroke();
		}
		this.ctxs.borders.stroke();
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
		// let srcX = Math.max(0, Math.min(this.buffer.width, this.
		this.outerCtx.imageSmoothingEnabled = false;
		for (let layer of this.layers) {
			let buffer = this.buffers[layer];
			this.outerCtx.drawImage(
				buffer,
				this.canvas.width / 2 - centerX,
				this.canvas.height / 2 - centerY - (layer === "ho" ? tileSize : 0),
				buffer.width * this.scale,
				buffer.height * this.scale
			);
		}
	}

	resize(width, height) {
		this.canvas.width = width;;
		this.canvas.height = height;
		this.redraw();
	}
}
