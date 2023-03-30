import sys
sys.path.append("..")
sys.path.append("../..")
sys.path.append("../../..")

from faceplate_maker import *

HP = 4
WIDTH = inches(HP * .2)

module = Module(HP, (WIDTH/2, 32), title="Distortion", filename="dd_faceplate.svg", title_size=4)

module.add(Potentiometer(inches(-.1), inches(0)))

module.add(JackSocket(inches(0), inches(1.2), "CV", False))
module.add(JackSocket(inches(0), inches(1.2 + 0.7), "In", False))
module.add(JackSocket(inches(0), inches(1.2 + 0.7 + 0.7), "Out", True))


module.save()
