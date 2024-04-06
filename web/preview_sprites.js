"use strict";
function main() {
	let resolution = 8;
	let spritemap = loadSprites();

	let fuzzSprite = new FuzzTemplate(document.getElementById("fuzz-template"), 1, 1).asSprite();

	let rows = [];
	for (let sprite of spritemap.all()) {
		while (sprite.y >= rows.length) {
			rows.push([]);
		}
		let row = rows[sprite.y];
		while (sprite.x >= row.length) {
			row.push([]);
		}
		row[sprite.x].push(sprite);
	}
	for (let row of rows) {
		let htmlRow = document.createElement("div");
		for (let sprites of row) {
			if (!sprites.length) {
				continue;
			}
			let sprite = sprites[0];
			let figure = document.createElement("figure");

			// let div = document.createElement("div");
			// div.innerText = "123";
			// figure.appendChild(div);

			let canvas = document.createElement("canvas");
			let display = new Display(canvas, spritemap, fuzzSprite);
			display.setViewArea({x: 0, y: 0, w: 2, h: 2});
			display.drawSprite(sprite.name, 0.5, 0.5);
			if (sprite.big) {
				display.resize(64, 72);
				display.setCenter(1, 0.5);
			} else {
				display.resize(40, 40);
				display.setCenter(1, 1);
			}
			display.init = true;
			display.redraw();
			figure.appendChild(canvas);

			// let span = document.createElement("div");
			// span.innerText = "abc";
			// figure.appendChild(span);

			let caption = document.createElement("figcaption");
			caption.innerText = sprites.map(s => s.name).join(",");
			figure.appendChild(caption);

			htmlRow.appendChild(figure);
		}
		document.getElementById("previews").appendChild(htmlRow);
	}
}

addEventListener("load", main);
