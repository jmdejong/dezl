"use strict";

class ActionBar {

	constructor() {
		this.actions = [
			{name: "<inspect>", message: direction => ({inspect: direction})},
			{name: "<take>", message: direction => ({interact: [null, direction]})}
		];
		this.selector = 0;
		this.items = [];
		let actionTable = document.getElementById("interactions");
		for (let i in this.actions) {
			let action = this.actions[i];
			let row = this._buildRow(i, action.name, null);
			actionTable.appendChild(row);
		}
		this.select(0);
	}

	setInventory(items) {
		this.items = items;
		let table = document.getElementById("inventory");

		let rows = table.querySelectorAll("li");
		rows.forEach(function(row) {
			row.remove();
		});

		for (let i in items) {
			let item = items[i];
			let row = this._buildRow(i, item[0], item[1]);

			table.appendChild(row);
		}
		this.select(Math.min(this.selector, this.actions.length + this.items.length - 1));
	}

	_buildRow(index, name, quantity) {
		let row = document.createElement("li");
		row.onclick = () => this.select(index | 0);
		row.className = "inv-row selectable-row";

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
		return row;
	}

	select(idx) {
		this.selector = idx;
		let items = document.getElementsByClassName("selectable-row");
		for (let i=0; i<items.length; ++i) {
			let row = items.item(i);
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
			return this.actions[this.selector].message(direction);
		} else {
			return {interact: [this.selector - this.actions.length, direction]};
		}
	}
}
