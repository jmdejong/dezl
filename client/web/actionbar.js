"use strict";

class ActionBar {

	constructor() {
		this.actions = [
			["<inspect>", null, direction => {return {inspect: direction}}],
			["<take>", null, direction => {return {take: direction}}]
		];
		this.selector = 0;
		this.items = [];
	}

	setInventory(items) {
		this.items = items;
		let table = document.getElementById("inventory");

		let rows = table.querySelectorAll("li");
		rows.forEach(function(row) {
			row.remove();
		});

		let entries = this.actions.concat(items);

		for (let i=0; i<entries.length; ++i) {
			let item = entries[i];
			let name = item[0];
			let quantity = item[1];
			let row = document.createElement("li");
			row.onclick = () => this.select(i | 0);
			row.className = "inv-row";

			let nm = document.createElement("span");
			nm.className = "inventory-name";
			nm.innerText = name;
			row.appendChild(nm);

			let am = document.createElement("span");
			am.className = "inventory-amount";
			if (quantity !== null && quantity !== undefined) {
				am.innerText = quantity;
			}
			row.appendChild(am);

			table.appendChild(row);
		}
		this.select(Math.min(this.selector, entries.length - 1));
	}

	select(idx) {
		this.selector = idx;
		let table = document.getElementById("inventory");
		for (let i=0; i<table.children.length; ++i) {
			let row = table.children[i];
			row.classList.remove("inv-selected");
			if (i == this.selector) {
				row.classList.add("inv-selected");
			}
			if (Math.abs(i - this.selector) <= 1) {
				row.scrollIntoView({behavior: "instant", block: "nearest"});
			}
		}
	}


	selectRel(dif) {
		let n = this.items.length + this.actions.length;
		this.select((this.selector + n + dif) % n);
	}

	selectedAction(direction) {
		if (this.selector < this.actions.length) {
			return this.actions[this.selector][2](direction);
		} else {
			return {use: [this.selector - this.actions.length, direction]};
		}
	}
}
