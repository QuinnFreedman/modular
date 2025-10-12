import sys
sys.path.append("..")
sys.path.append("../..")
sys.path.append("../../..")

import math
from faceplate_maker import *
from svgwrite.path import Path

module = Module.from_cli(6, global_y_offset=17, title="Biodata")

probe_x = inches(-.3)
probe_y = inches(.1)
module.add(JackSocketCentered(probe_x, probe_y, "Probe", False, rotation=3))
# module.draw(lambda c: c.text("Probe", insert=(-probe_y, probe_x - inches(.25)), transform="rotate(-90)", text_anchor="middle"))

path = Path(stroke="black", fill="none", stroke_width=.8)
r = 5
path.push(f"M {probe_x+r},{probe_y}")
path.push(f"A {r} {r} 0 1 0 {probe_x-r} {probe_y}")
path.push(f"A {r} {r} 0 1 0 {probe_x+r} {probe_y}")
module.draw(lambda _: path)

module.add(Potentiometer(-inches(.1), inches(1.3), "Sensitivity", False, color=PotColor.GREEN, style=PotStyle.SIFAM_LARGE))
y = inches(2.3)
module.add(SmallPotentiometer(-inches(.3)-inches(.1), y, "Density", False))
module.add(SmallPotentiometer(inches(.3)-inches(.1), y, "Spread"))


 
y = inches(2.75)
module.add(JackSocketCentered(inches(-.4), y, "Trig", False, rotation=2))
module.add(JackSocketCentered(inches(.4), y, "Gate", True, rotation=2))
module.add(LED(0, y-inches(.05), color="green"))

y += inches(.65)
module.add(JackSocketCentered(inches(-.4), y, "Raw", True, rotation=2))
module.add(JackSocketCentered(inches(.4), y, "V/Oct", True, rotation=2))
module.add(JackSocketCentered(inches(0), y, "CV", True, rotation=2))

module.save()
