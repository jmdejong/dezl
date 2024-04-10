"use strict";

function loadSprites(){

	let fuzzTemplate = new FuzzTemplate(document.getElementById("fuzz-template"), 1, 1);

	let spritemap = new SpriteMap();
	spritemap.addSprites(
		document.getElementById("spritemap"),
		{
			player: {x: 0, y: 0, layer: "creatures"},
			sage: {x: 1, y: 0},
			frog: {x: 3, y: 0, layer: "creatures"},
			worm: {x: 4, y: 0, layer: "creatures"},
			fireplace: {x: 2, y: 7},
			ashplace: {x: 3, y: 7},
			fire: {x: 4, y: 7},
			markstone: {x: 5, y: 7},
			worktable: {x: 6, y: 7},
			altar: {x: 7, y: 7},
			grass1: {x: 0, y: 1, layer: "ground"},
			grass2: {x: 1, y: 1, layer: "ground"},
			grass3: {x: 2, y: 1, layer: "ground"},
			dirt: {x: 3, y: 1, layer: "ground"},
			deadleaves: {x: 4, y: 1, layer: "ground"},
			rockfloor: {x: 5, y: 1, layer: "ground"},
			moss: {x: 6, y: 1, layer: "ground"},
			water: {x: 0, y: 2, border: 0x222266, layer: "base"},
			stonefloor: {x: 1, y: 2, border: 0xaaaaaa, layer: "base"},
			woodwall: {x: 3, y: 2, border: 0x222200, layer: "base"},
			wall: {x: 4, y: 2, border: 0x222222, layer: "base"},
			rock: {x: 5, y: 2, border: 0x222222, layer: "base"},
			rockmid: {x: 7, y: 2, border: 0x222222, layer: "base"},
			" ": {x: 7, y: 2},
			rush: {x: 0, y: 3},
			pitcherplant: {x: 1, y: 3},
			reed: {x: 2, y: 3},
			flower: {x: 3, y: 3},
			densegrass: {x: 4, y: 3, layer: "base"},
			gravel: {x: 5, y: 3, layer: "base"},
			shrub: {x: 6, y: 3},
			bush: {x: 7, y: 3},
			tree: {x: 3, y: 5, wide: true},
			oldtree: {x: 5, y: 5},
			oldtreetinder: {x: 6, y: 5},
			youngtree: {x: 1, y: 5},
			sapling: {x: 0, y: 5},
			pebble: {x: 0, y: 6},
			stone: {x: 1, y: 6},
			stick: {x: 2, y: 6},

			plantedseed: {x: 0, y: 8},
			seedling: {x: 1, y: 8},
			greenstem: {x: 2, y: 8},
			brownstem: {x: 3, y: 8},

			youngdiscplant: {x: 0, y: 9},
			discplant: {x: 1, y: 9},
			seedingdiscplant: {x: 2, y: 9},
			discshoot: {x: 3, y: 9},
			discleaf: {x: 4, y: 9},
			discknifeplant: {x: 5, y: 9},
			sawblade: {x: 6, y: 9},

			youngknifeplant: {x: 0, y: 10},
			knifeplant: {x: 1, y: 10},
			seedingknifeplant: {x: 2, y: 10},
			knifeshoot: {x: 3, y: 10},
			knifeleaf: {x: 4, y: 10},
			hardknifeplant: {x: 5, y: 10},
			hardwoodknife: {x: 6, y: 10},

			younghardplant: {x: 0, y: 11},
			hardplant: {x: 1, y: 11},
			seedinghardplant: {x: 2, y: 11},
			hardshoot: {x: 3, y: 11},
			hardwoodstick: {x: 4, y: 11},
			harddiscplant: {x: 5, y: 11},
			hardwoodtable: {x: 6, y: 11},

			sawtable: {x: 7, y: 11},
		},
		8,
		fuzzTemplate
	);
	return spritemap;
}
