
from . import utils


class Inventory:

	def __init__(self, display):
		self.display = display
		self.actions = [("<inspect>", None), ("<take>", None)]
		self.items = []
		self.selector = 1

	def size(self):
		return len(self.items) + len(self.actions)

	def select(self, value):
		self.selector = utils.clamp(value, 0, self.size() - 1)
		self.redraw()

	def selectRelative(self, d):
		itemLen = self.size()
		if itemLen < 1:
			return
		self.selector = (self.selector + d + itemLen) % itemLen
		self.redraw()

	def setItems(self, items):
		self.items = items
		self.selector = utils.clamp(self.selector, 0, self.size() - 1)
		self.redraw()

	def redraw(self):
		self.display.setInventory([*self.actions, *self.items], self.selector)

	def action(self, direction, pos):
		if self.selector == 0:
			return {"inspect": _movepos(pos, direction)}
		elif self.selector == 1:
			return {"interact": [None, direction]}
		else:
			selector = self.selector - len(self.actions)
			return {"interact": [selector, direction]}

def _movepos(pos, direction):
	if direction == "north":
		return (pos[0], pos[1] - 1)
	elif direction == "south":
		return (pos[0], pos[1] + 1)
	elif direction == "east":
		return (pos[0] + 1, pos[1])
	elif direction == "west":
		return (pos[0] - 1, pos[1])
	elif direction == None or direction == "":
		return pos
	else:
		raise ValueError("Unknown direction " + direction)


