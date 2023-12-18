

import os
import sys
import time

import threading
from queue import Queue

import ratuil.inputs

from .inputhandler import InputHandler
from .common import messages

class Client:
	
	def __init__(self, display, name, connection, keybindings, logFile=None):
		
		self.display = display
		self.name = name
		self.keepalive = True
		self.connection = connection
		self.logFile = logFile
		self.closeMessage = None
		self.helpVisible = False
		self.playerPos = None
		
		self.inputHandler = InputHandler(self, keybindings.actions)
		
		self.shortHelp = keybindings.shorthelp or ""
		self.longHelp = keybindings.longhelp or ""
		
		self.display.showInfo(self.shortHelp)
		self.display.setLongHelp(self.longHelp)
		self.queue = Queue()
		
	
	def sendMessage(self, message):
		self.connection.send(message)
	
	def sendInput(self, inp):
		message = messages.InputMessage(inp)
		self.sendMessage(message)
	
	def sendChat(self, text):
		try:
			self.sendMessage(messages.ChatMessage(text))
		except messages.InvalidMessageError as e:
			self.log(e.description)
	
	def start(self):
		threading.Thread(target=self.listen, daemon=True).start()
		threading.Thread(target=self.timeInput, daemon=True).start()
		
		self.command_loop()
	
	def listen(self):
		try:
			self.connection.listen(self.pushMessage, self.onConnectionError)
		except BaseException as error:
			self.queue.put(("error", error))
	
	def pushMessage(self, message):
		self.queue.put(("message", message))
	
	def onConnectionError(self, error):
		self.queue.put(("error", error))
	
	def timeInput(self):
		while True:
			self.queue.put(("checkinput", None))
			time.sleep(0.04)
	
	def getInput(self):
		try:
			while True:
				key = self.display.screen.get_key()
				self.queue.put(("input", key))
		except BaseException as error:
			self.queue.put(("error", error))
	
	def close(self, msg=None):
		self.keepalive = False
		self.closeMessage = msg
	
	def toggleHelp(self):
		self.helpVisible = not self.helpVisible
		if self.helpVisible:
			for line in self.longHelp.splitlines():
				self.display.addMessage(line, "help")
			self.display.showHelp()
		else:
			self.display.hideHelp()
	
	
	def update(self, message):
		if message is None:
			self.close("Connection closed by server")
			return
		if isinstance(message, messages.ErrorMessage):
			error = message.errType
			if error == "nametaken":
				self.close("error: name is already taken")
				return
			if error == "invalidname":
				self.close("Invalid name error: "+ str(message.description))
				return
			self.log(message.errType + ": " + message.description)
		elif isinstance(message, messages.MessageMessage):
			self.log(message.text, message.type)
		elif isinstance(message, messages.WorldMessage):
			self.handleWorldUpdate(message.updates)
	
	def handleWorldUpdate(self, m):
		viewArea = m.get('viewarea')
		if viewArea:
			area = viewArea["area"]
			self.display.setViewArea(**area)

		section = m.get('section')
		if section:
			rawarea = section["area"]
			area = ((rawarea["x"], rawarea["y"]), (rawarea["w"], rawarea["h"]))
			self.display.drawSection(area, section["field"], section["mapping"])
		
		changeCells = m.get('changecells')
		if changeCells and len(changeCells):
			self.display.drawFieldCells(changeCells)

		dynamics = m.get('dynamics')
		if dynamics:
			self.display.drawDynamics(dynamics)

		playerPos = m.get('playerpos')
		if playerPos:
			self.display.setFieldCenter(playerPos["pos"])
			self.playerPos = playerPos["pos"]
		
		inventory = m.get("inventory")
		if inventory:
			items, selected = inventory
			self.display.setInventory(items, selected)

		sounds = m.get("messages")
		if sounds:
			for message in sounds:
				type = message[0]
				text = message[1]
				arg = None
				if len(message) > 2:
					arg = message[2]
				if type == "options":
					self.log(arg["description"])
					for (command, description) in arg["options"]:
						self.log("/q {:<24}   - {}".format(command, description))
				else:
					self.log(text, type)
	
	def log(self, text, type=None):
		if not isinstance(text, str):
			text = str(text)
		self.display.addMessage(text, type)
		if self.logFile:
			with(open(self.logFile, 'a')) as f:
				f.write("[{}] {}\n".format(type or "", text))
	
	
	def command_loop(self):
		while self.keepalive:
			self.display.update()
			action = self.queue.get()
			if action[0] == "message":
				self.update(action[1])
			elif action[0] == "input":
				if action[1] == "^C":
					raise KeyboardInterrupt
				self.inputHandler.onInput(action[1])
			elif action[0] == "checkinput":
				key = self.display.screen.get_key_now()
				if key == "^C":
					raise KeyboardInterrupt
				self.inputHandler.onInput(key)
			elif action[0] == "error":
				raise action[1]
			elif action[0] == "sigwinch":
				self.display.update_size()
			else:
				raise Exception("invalid action in queue")
	
	def onSigwinch(self, signum, frame):
		self.queue.put(("sigwinch", (signum, frame)))
	



