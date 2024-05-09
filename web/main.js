"use strict";


window.addEventListener("load", main);

function main(){
	let loginForm = document.getElementById("login");
	loginForm.hidden = false;
	let hostInput = loginForm.hostinput;
	if (hostInput.value === hostInput.defaultValue || !hostInput.value) {
		hostInput.value = default_host;
	}
	let nameInput = loginForm.username;
	if (!nameInput.value || nameInput.value === nameInput.defaultValue) {
		nameInput.value = window.localStorage.getItem("dezl_username") || generateRandomName();
	}
	document.getElementById("randomname").addEventListener("click", _ => nameInput.value = generateRandomName());

	let params = new URLSearchParams(window.location.search);
	if (params.get("login")) {
		start(params.get("login"), default_host);
	} else {
		loginForm.addEventListener("submit", e => {
			let username = e.target.username.value;
			if (username) {
				window.localStorage.setItem("dezl_username", username);
			}
			start(username, e.target.host.value)
		});
	}
}

function generateRandomName() {
	let vowels = "aaaaeeeeeeiiioooouu";
	let consonants = "bbccdddffgghhjjkkllmmmnnnnppqrrrrsssstttttvvwxyz";

	let name = ""
	let nSyllables = 2 + Math.random() * 2;
	for (let i=0; i<nSyllables; ++i) {
		if (Math.random() > 0.5) {
			name += consonants.charAt((Math.random()*consonants.length)|0);
		}
		name += vowels.charAt((Math.random()*vowels.length)|0);
		if (Math.random() > 0.5) {
			name += consonants.charAt((Math.random()*consonants.length)|0);
		}
	}
	return name;
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
