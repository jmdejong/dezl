
from . import utils


class Inventory:

	def __init__(self, display):
		self.display = display
		self.items = []
		self.selector = 0

	def getSelected(self):
		return self.selector

	def select(self, value):
		self.doSelect(utils.clamp(value, 0, len(self.items)-1))

	def selectRelative(self, d):
		itemLen = len(self.items)
		if itemLen < 1:
			return
		self.doSelect((self.selector + d + itemLen) % itemLen)

	def doSelect(self, value):
		self.selector = value
		self.display.setInventory(self.items, self.selector)

	def setItems(self, items):
		self.items = items
		self.selector = utils.clamp(self.selector, 0, len(items)-1)
		self.display.setInventory(self.items, self.selector)

	def getItem(self, num):
		return self.items[num]

	def getSelectedItem(self):
		return self.getItem(self.getSelected())

	def getNumItems(self):
		return len(self.items)

	def itemName(self, item):
		return item

