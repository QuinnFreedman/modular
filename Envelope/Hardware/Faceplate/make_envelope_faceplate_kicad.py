import sys
sys.path.append("..")
sys.path.append("../..")
sys.path.append("../../..")

from faceplate_maker import *
from svgwrite.path import Path

HP = 8

module = Module(HP, (0, 12), title="Envelope", filename="envelope_faceplate.svg", title_size=4.5)

y = inches(0.1)

module.add(Button(inches(.25), y))

colors = ["gold", "blue", "red", "green"]

for i in range(4):
    x = inches(0.6 + .2 * i)
    module.add(SmallLED(x, y, color=colors[i]))

y += inches(.2)

colors = ["yellow", "blue", "red", "green"]
labels = ["A/A/A/A", "D/A'/A'/S", "S/R/R/R", "R/R'/R'/D"]


def get_center(circle):
    return (circle.offset[0] + circle.position[0],
            circle.offset[1] + circle.position[1])

            
for i in range(4):
    y += inches(.4)
    jack = JackSocketCentered(inches(0.3), y, labels[i], False)
    module.add(jack)
    
    stroke_width = .6
    center = get_center(jack)
    path = Path(stroke="black", fill="none", stroke_width=stroke_width)
    path.push(f"M {center[0]} {center[1]}")
    path.push(f"H {inches(1.2)}")
    module.draw(lambda _: path)
    
    y += inches(.3)
    module.add(Potentiometer(inches(1.1), y, color=colors[i]))

y += inches(.5)


module.add(JackSocketCentered(inches(.2), y, "Gate", False))
module.add(JackSocketCentered(inches(.6), y, "Trig", False))
module.add(JackSocketCentered(inches(1.0), y, "Out", True))
module.add(JackSocketCentered(inches(1.4), y, "Inv", True))

module.save()
