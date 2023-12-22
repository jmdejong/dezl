"use strict";


window.addEventListener("load", main);

function main(){
	let loginForm = document.getElementById("login");
	loginForm.hidden = false;
	loginForm.addEventListener("submit", start);
	let hostInput = document.getElementById("hostinput");
	if (hostInput.value === hostInput.defaultValue) {
		hostInput.value = `ws://${window.location.hostname || "localhost"}:9231`;
	}
}



function start(e) {
	let form = e.target;
	let username = form.username.value;
	let host = form.host.value;
	
	let canvas = document.getElementById("canvas");

	let fuzzTemplate = new FuzzTemplate(document.getElementById("fuzz-template"), 1, 1);

	let spritemap = new SpriteMap();
	spritemap.addSprites(
		document.getElementById("spritemap"),
		{
			player: {x: 0, y: 0, layer: "creatures"},
			sage: {x: 1, y: 0},
			worktable: {x: 6, y: 0},
			altar: {x: 7, y: 0},
			grass1: {x: 0, y: 1, layer: "ground"},
			grass2: {x: 1, y: 1, layer: "ground"},
			grass3: {x: 2, y: 1, layer: "ground"},
			dirt: {x: 3, y: 1, layer: "ground"},
			rockmid: {x: 4, y: 1, border: "#222", layer: "base"},
			" ": {x: 4, y: 1},
			rock: {x: 5, y: 1, border: "#222", layer: "base"},
			water: {x: 6, y: 1, border: "#004", layer: "base"},
			moss: {x: 7, y: 1, layer: "ground"},
			deadleaves: {x: 0, y: 2, layer: "ground"},
			densegrass: {x: 1, y: 2, layer: "ground"},
			wall: {x: 2, y: 2, border: "#222", layer: "base"},
			woodwall: {x: 3, y: 2, border: "#220", layer: "base"},
			stonefloor: {x: 4, y: 2, layer: "base"},
			rockfloor: {x: 5, y: 2, layer: "ground"},
			rush: {x: 0, y: 3},
			pitcherplant: {x: 1, y: 3},
			tree: {x: 2, y: 5, ho: true},
			oldtree: {x: 3, y: 5},
			oldtreetinder: {x: 4, y: 5, ho: true},
			youngtree: {x: 1, y: 5},
			sapling: {x: 0, y: 5},
			shrub: {x: 6, y: 3},
			bush: {x: 7, y: 3},
			reed: {x: 2, y: 3},
			gravel: {x: 5, y: 3},
			pebble: {x: 0, y: 6},
			stone: {x: 1, y: 6},
			stick: {x: 2, y: 6},
		},
		8,
		fuzzTemplate
	);
	let display = new Display(canvas, spritemap, fuzzTemplate.asSprite());
	let client = new Client(username, host, display, parseParameters());
	client.start()
	form.hidden = true;
	window.game_client_debug = client;
}


function parseParameters(){
	let ps = new URLSearchParams(window.location.search)
	let parameters = {};
	for (let p of ps){
		parameters[p[0]] = p[1];
	}
	return parameters;
}
