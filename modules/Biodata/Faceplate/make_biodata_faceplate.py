import sys
sys.path.append("..")
sys.path.append("../..")
sys.path.append("../../..")

import math
from faceplate_maker import *
from svgwrite.path import Path

module = Module.from_cli(10, global_y_offset=15, title="Biodata")

module.add(JackSocketCentered(-inches(.6), inches(.3), "Probe", False, rotation=2))
module.add(Potentiometer(inches(.4), inches(.6), "Sensitivity", False, color=PotColor.BLUE))
module.add(Potentiometer(-inches(.6), inches(1.3), "Density", False, color=PotColor.GREEN))

module.add(SmallPotentiometer(-inches(.4), inches(2.0), "Slew"))
module.add(SmallPotentiometer(-inches(.65), inches(2.7), "Spread"))

module.add(LED(-inches(.75), inches(3.05), color="blue"))
module.add(JackSocketCentered(-inches(.25), inches(3.1), "Sample", False, rotation=2))
module.add(JackSocketCentered(-inches(.25), inches(3.7), "CV", True, rotation=2))
module.add(JackSocketCentered(-inches(.75), inches(3.7), "Gate", True, rotation=2))

module.add(SmallPotentiometer(inches(.2), inches(2.0), "Slew"))
module.add(SmallPotentiometer(inches(.45), inches(2.7), "Spread"))

module.add(LED(inches(.75), inches(3.05), color="green"))
module.add(JackSocketCentered(inches(.25), inches(3.1), "Sample", False, rotation=2))
module.add(JackSocketCentered(inches(.25), inches(3.7), "CV", True, rotation=2))
module.add(JackSocketCentered(inches(.75), inches(3.7), "Gate", True, rotation=2))

module.add(JackSocketCentered(inches(0), inches(2.4), "Raw", True, rotation=2))

module.save()
