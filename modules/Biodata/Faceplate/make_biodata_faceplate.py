import sys
sys.path.append("..")
sys.path.append("../..")
sys.path.append("../../..")

import math
from faceplate_maker import *
from svgwrite.path import Path

module = Module.from_cli(12, global_y_offset=17, title="Biodata")

left_col = -inches(.8)

probe_x = left_col
probe_y = inches(.1)
module.add(JackSocketCentered(probe_x, probe_y, "Probe", False, rotation=1))
# module.draw(lambda c: c.text("Probe", insert=(-probe_y, probe_x - inches(.25)), transform="rotate(-90)", text_anchor="middle"))

path = Path(stroke="black", fill="none", stroke_width=.8)
r = 5
path.push(f"M {probe_x+r},{probe_y}")
path.push(f"A {r} {r} 0 1 0 {probe_x-r} {probe_y}")
path.push(f"A {r} {r} 0 1 0 {probe_x+r} {probe_y}")
module.draw(lambda _: path)

module.add(Potentiometer(left_col - inches(.1), inches(1.2), "Sensitivity", False, color=PotColor.BLUE))
module.add(Potentiometer(left_col - inches(.1), inches(2.15), "Density", False, color=PotColor.GREEN))
module.add(JackSocketCentered(left_col, inches(2.75), "CV", True, rotation=2))
module.add(JackSocketCentered(left_col, inches(3.4), "Raw", True, rotation=2))

led_colors = ["blue", "green", "blue"]

for i in range(3):
    odd = True #i % 2 == 0
    x = i * inches(.525) - inches(.15)
    y = inches(0.4)
    module.add(SmallPotentiometer(x - inches(.1), y, "Spread"))
    y += inches(.9)
    module.add(SmallPotentiometer(x - inches(.1), y, "Slew"))
    y += inches(.3)
    module.add(LED(x, y, color=led_colors[i]))
    y += inches(.5)
    module.add(JackSocketCentered(x, y, "Sample", False, rotation=2))
    y += inches(.65)
    module.add(JackSocketCentered(x, y, "CV", True, rotation=2))
    y += inches(.65)
    module.add(JackSocketCentered(x, y, "Gate", True, rotation=2))

module.save()
