"use strict";


window.addEventListener("load", main);

function main(){
	let loginForm = document.getElementById("login");
	loginForm.hidden = false;
	let hostInput = document.getElementById("hostinput");
	if (hostInput.value === hostInput.defaultValue) {
		hostInput.value = `ws://${window.location.hostname || "localhost"}:9231`;
	}

	let params = new URLSearchParams(window.location.search);
	if (params.get("login")) {
		start(params.get("login"), loginForm.host.value);
	} else {
		loginForm.addEventListener("submit", e => start(e.target.username.value, e.target.host.value));
	}
}



function start(username, host) {
	
	let canvas = document.getElementById("canvas");

	let fuzzTemplate = new FuzzTemplate(document.getElementById("fuzz-template"), 1, 1);

	let spritemap = loadSprites();
	let targets = {
		lo: document.getElementById("canvas-lo"),
		creatures: document.getElementById("canvas-creatures"),
		mid: document.getElementById("canvas-mid"),
		effect: document.getElementById("canvas-effect"),
		hi: document.getElementById("canvas-hi"),
	};
	let display = new Display(targets, spritemap, fuzzTemplate.asSprite());
	let client = new Client(username, host, display, parseParameters());
	client.start()
	document.getElementById("login").hidden = true;
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
