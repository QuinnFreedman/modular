import sys
sys.path.append("..")
sys.path.append("../..")
sys.path.append("../../..")

from faceplate_maker import *
from svgwrite.path import Path

module = Module.from_cli(8, global_y_offset=12, title="Envelope", title_size=4.6)

y = inches(0.1)

module.add(TL1105SP(inches(.25-.75), y + inches(.05)))

colors = ["gold", "blue", "red", "green"]

for i in range(4):
    x = inches(.225 * i - .175)
    module.add(SmallLED(x, y, color=colors[i]))

y += inches(.05)

colors = [PotColor.YELLOW, PotColor.BLUE, PotColor.RED, PotColor.GREEN]
labels = ["A/A/A/A", "D/C/C/H", "S/R/R/R", "R/C/C/D"]


def get_center(circle):
    return (circle.offset[0] + circle.position[0],
            circle.offset[1] + circle.position[1])

            
for i in range(4):
    y += inches(.5)
    jack = JackSocketCentered(-inches(0.5), y, labels[i], False)
    module.add(jack)
    
    stroke_width = .6
    center = get_center(jack)
    path = Path(stroke="black", fill="none", stroke_width=stroke_width)
    path.push(f"M {center[0]} {center[1]}")
    path.push(f"h {inches(.9)}")
    module.draw(lambda _: path)
    
    y += inches(.3)
    module.add(Potentiometer(inches(0.3), y, color=colors[i]))

y += inches(.375)

module.add(JackSocketCentered(inches(-.6), y, "Gate", False))
module.add(JackSocketCentered(inches(-.2), y, "Trig", False))
module.add(JackSocketCentered(inches(.2), y, "Out", True))
module.add(JackSocketCentered(inches(.6), y, "Aux", True))

module.save()
