import sys
sys.path.append("..")
sys.path.append("../..")
sys.path.append("../../..")

from faceplate_maker import *
from svgwrite.path import Path

HP = 8

module = Module(HP, (inches(.2), 12), title="Envelope", filename="envelope_faceplate.svg")

y = 0

module.add(Button(inches(.2), y))

colors = ["gold", "blue", "red", "green"]

for i in range(4):
    x = inches(.5 + .2 * i)
    module.add(SmallLED(x, y, color=colors[i]))

y += inches(.1)

colors = ["yellow", "blue", "red", "green"]
labels = ["A/A/A/A", "D/A'/A'/S", "S/R/R/R", "R/R'/R'/D"]


def get_center(circle):
    return (circle.offset[0] + circle.position[0],
            circle.offset[1] + circle.position[1])

            
for i in range(4):
    y += inches(.3)
    jack = JackSocket(inches(0.1), y, labels[i], False)
    module.add(jack)
    
    stroke_width = .6
    center = get_center(jack)
    path = Path(stroke="black", fill="none", stroke_width=stroke_width)
    path.push(f"M {center[0]} {center[1]}")
    path.push(f"H {inches(.9)}")
    module.draw(lambda _: path)
    
    y += inches(.5)
    module.add(Potentiometer(inches(.9), y, color=colors[i]))

y += inches(.3)


module.add(JackSocket(inches(0), y, "Gate", False))
module.add(JackSocket(inches(.4), y, "Trig", False))
module.add(JackSocket(inches(.8), y, "Out", True))
module.add(JackSocket(inches(1.2), y, "Inv", True))

module.save()
