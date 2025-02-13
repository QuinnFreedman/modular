import sys
sys.path.append("..")
sys.path.append("../..")
sys.path.append("../../..")

import math
from faceplate_maker import *
from svgwrite.path import Path

module = Module.from_cli(4, global_y_offset=12, title="ALU")

y_spacing = inches(.65)
y_start = inches(.3)
x_offset = inches(.19)

module.add(JackSocketCentered(-x_offset, y_start, "A", False, rotation=2))
module.add(JackSocketCentered(x_offset, y_start, "B", False, rotation=2))
module.add(JackSocketCentered(-x_offset, y_start + y_spacing, "OR", True, rotation=2))
module.add(JackSocketCentered(x_offset, y_start + y_spacing, "AND", True, rotation=2))
module.add(JackSocketCentered(-x_offset, y_start + 2 * y_spacing, "NOR", True, rotation=2))
module.add(JackSocketCentered(x_offset, y_start + 2 * y_spacing, "XOR", True, rotation=2))

y_start = y_start + y_spacing * 3 + inches(.1)
module.add(JackSocketCentered(-x_offset, y_start, "A", False, rotation=2))
module.add(JackSocketCentered(x_offset, y_start, "B", False, rotation=2))
module.add(JackSocketCentered(-x_offset, y_start + y_spacing, "A+B", True, rotation=2))
module.add(JackSocketCentered(x_offset, y_start + y_spacing, "A-B", True, rotation=2))
module.add(JackSocketCentered(-x_offset, y_start + 2 * y_spacing, "MAX", True, rotation=2))
module.add(JackSocketCentered(x_offset, y_start + 2 * y_spacing, "|A-B|", True, rotation=2))

module.save()
