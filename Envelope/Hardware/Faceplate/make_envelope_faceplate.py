import sys
sys.path.append("..")
sys.path.append("../..")
sys.path.append("../../..")

from faceplate_maker import *

HP = 8

module = Module(HP, (inches(.2), 12), title="Envelope", filename="envelope_faceplate.svg")

y = 0

Button = BasicCircle(0, 0, 3.5)

module.add(Button(inches(.2), y))

for i in range(4):
    x = inches(.5 + .2 * i)
    module.add(SmallLED(x, y))

y += inches(.1)

for i in range(4):
    y += inches(.3)
    module.add(JackSocket(inches(0.1), y, "", False))
    y += inches(.5)
    module.add(Potentiometer(inches(.9), y))

y += inches(.3)


module.add(JackSocket(inches(0), y, "Gate", False))
module.add(JackSocket(inches(.4), y, "Trig", False))
module.add(JackSocket(inches(.8), y, "Out", True))
module.add(JackSocket(inches(1.2), y, "Inv", True))

module.save()
