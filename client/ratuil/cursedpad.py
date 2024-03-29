

import curses
from .constants import INT_INFINITY
from .drawtarget import DrawTarget

from .strwidth import charwidth


class CursedPad(DrawTarget):
	
	def __init__(self, scr, colours, width, height):
		self.scr = scr
		self.colours = colours
		self.width = width
		self.height = height
		self.pad = curses.newpad(max(height, 1), max(width, 1))
		self.clear()
	
	def resize(self, width, height):
		self.width = width
		self.height = height
		self.pad.resize(max(height, 1), max(width, 1))
		self.clear()
	
	
	def clear(self):
		self.pad.clear()
	
	def write(self, x, y, text, style=None):
		if y >= self.height:
			return
		for char in text:
			w = charwidth(char)
			if x + w > self.width:
				break
			self.set_char(x, y, char, self.get_raw_style(style))
			#if w == 2:
				#self.delete(x + 1, y)
			x += w
			
	
	def set_char(self, x, y, char, raw_style):
		try:
			self.pad.addstr(y, x, char, raw_style)
		except curses.error:
			# ncurses has a weird problem:
			# it always raises an error when drawing to the last character in the window
			# it draws first and then raises the error
			# therefore to draw in the last place of the window the last character needs to be ingored
			# other solutions might be possible, but are more hacky
			pass
	
	def get_raw_style(self, style):
		return self.colours.attrs(style)
	
	def draw_pad(self, src, dest_x=0, dest_y=0, width=INT_INFINITY, height=INT_INFINITY, src_x=0, src_y=0):
		width = min(width, self.width - dest_x, src.width - src_x)
		height = min(height, self.height - dest_y, src.height - src_y)
		src.pad.overlay(self.pad, src_y, src_x, dest_y, dest_x, dest_y + height-1, dest_x + width-1)
