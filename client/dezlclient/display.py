


import os
from ratuil.textstyle import TextStyle
from ratuil.layout import Layout
from .listselector import ListSelector
from ratuil.boxstyle import Value, Relativity

ALPHABET = "!\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~"

class Display:
	
	def __init__(self, screen, charmap):
		
		self.screen = screen
		self.screen.clear()
		
		self.charmap = charmap
		
		fname = os.path.join(os.path.dirname(__file__), "layout.xml")
		self.layout = Layout.from_xml_file(screen, fname)
		self.layout.get("field").set_char_size(self.charmap.character_width)
		
		self.layout.update()
		
		# temporary, until these have a better place
		self.inventory = ListSelector(self.getWidget("inventory"))
		self.inventory._debug_name = "inventory"
		self.fieldBuffer = {}
		self.previousDynamics = set()
	
	def getWidget(self, name):
		return self.layout.get(name)

	def setViewArea(self, x, y, w, h):
		field = self.getWidget("field")
		field.set_dimensions((x, y), w, h, keep=True)
		self.fieldBuffer = {pos: sprites for pos, sprites in self.fieldBuffer.items() if not (pos[0] < x-1 or pos[1] < y-1 or pos[0] > x+w or pos[1] > y+h)}

	def drawSection(self, area, fieldCells, mapping):
		field = self.getWidget("field")
		brushes = [self.brush(spriteNames) for spriteNames in mapping]
		field.draw_all(fieldCells, brushes, area)
		((xmin, ymin), (w, h)) = area
		for (i, c) in enumerate(fieldCells):
			x = i % w + xmin
			y = i // w + ymin
			self.fieldBuffer[(x, y)] = mapping[c]
	
	def drawFieldCells(self, cells):
		field = self.getWidget("field")
		for cell in cells:
			(x, y), spriteNames = cell
			brush = self.brush(spriteNames)
			field.change_cell(x, y, *brush)
			self.fieldBuffer[(x, y)] = spriteNames

	def drawDynamics(self, dynamics):
		field = self.getWidget("field")
		knownDynamics = set()
		for d in dynamics.values():
			x, y = d["p"]
			pos = (x, y)
			sprite = d["s"]
			field.change_cell(x, y, *self.brush([sprite, *self.fieldBuffer.get((x, y), [])]))
			self.previousDynamics.discard(pos)
			knownDynamics.add(pos)
		for (x, y) in self.previousDynamics:
			field.change_cell(x, y, *self.brush(self.fieldBuffer.get((x, y), [])))
		self.previousDynamics = knownDynamics
	
	def brush(self, spriteNames):
		if not len(spriteNames):
			char, fg, bg = self.charmap.get(' ')
		else:
			char, fg, bg = self.charmap.get(spriteNames[0])
			for spriteName in spriteNames[1:]:
				if bg is not None:
					break
				_char, _fg, bg = self.charmap.get(spriteName)
		return (char, TextStyle(fg, bg))
	
	def setFieldCenter(self, pos):
		self.getWidget("field").set_center(*pos)
	
	# def setHealth(self, health, maxHealth):
	# 	if health is None:
	# 		health = 0
	# 	if maxHealth is None:
	# 		maxHealth = 0
	# 	self.getWidget("health").set_total(maxHealth)
	# 	self.getWidget("health").set_filled(health)
	# 	self.getWidget("healthtitle").format({"filled": health, "total":maxHealth})
		
	
	def showInfo(self, infostring):
		self.getWidget("info").set_text(infostring)
	
	def setLongHelp(self, longHelp):
		pass
		#self.getWidget("help").set_text(longHelp)
	
	def getSelectedItem(self, menu=None):
		return self.inventory.getSelected()
	
	def selectItem(self, menu=None, *args, **kwargs):
		self.inventory.select(*args, **kwargs)
	
	def setInventory(self, items):
		self.inventory.setItems(items)
	
	def setInv(self, items):
		self.inventory.setItems([(":" if is_equipped else " ") + item for (item, is_equipped) in items])
	
	def setInventory(self, items, selected):
		itemStrs = ["{} {}".format(item, siCount(count)) for item, count in items]
		self.inventory.setItems(itemStrs)
		self.inventory.select(selected)
	
	def addMessage(self, message, msgtype=None):
		if msgtype is not None:
			style = self.charmap.get_message_style(msgtype)
		else:
			style = None
		self.getWidget("msg").add_message(message, style)
	
	def log(self, message):
		self.addMessage(str(message))
	
	def scrollBack(self, amount, relative=True):
		self.getWidget("msg").scroll(amount, relative)
	
	def setInputString(self, string, cursor):
		self.getWidget("textinput").set_text(string, cursor)
	
	def showHelp(self):
		self.layout.id_elements.get("msg").style.height = Value(.8, Relativity.VERY_RELATIVE)
		self.layout.resize()
		self.update(force=True)
		
	def hideHelp(self):
		self.layout.id_elements.get("msg").style.height = Value(3, Relativity.ABSOLUTE)
		self.layout.resize()
		self.update(force=True)
	
	def update(self, force=False):
		self.layout.update(force)
		self.screen.update()
	
	def update_size(self):
		self.screen.reset()

def siCount(count):
	if count is None:
		return ""
	elif count < 1000:
		return str(count)
	else:
		thousands = 0
		while count >= 1000:
			thousands += 1
			count /= 1000
		suffix = "_KMGTPEZY"[thousands]
		if count < 10:
			return "{:.1f}{}".format(count, suffix)
		else:
			return "{:.0f}{}".format(count, suffix)
