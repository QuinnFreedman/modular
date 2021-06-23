import sys
import os

path = ".."
while "faceplate_maker.py" not in os.listdir(path):
    path += "/.."
    if not os.path.isdir(path):
        print("Unable to find faceplate maker library")
        sys.exit(1)
sys.path.append(path)

from faceplate_maker import *
from svgwrite.path import Path

HP = 8

module = Module(HP, (0, 12), title="Quant", filename="quantizer_faceplate.svg")


module.add(JackSocketCentered(inches(1.25), inches(0.25), "In", False))
module.add(JackSocketCentered(inches(1.25), inches(0.85), "Trig", False))
module.add(JackSocketCentered(inches(1.25), inches(1.45), "Out", True))

module.add(Button(inches(1.25), inches(2.0)))

module.add(JackSocketCentered(inches(1.25), inches(2.55), "In", False))
module.add(JackSocketCentered(inches(1.25), inches(3.15), "Trig", False))
module.add(JackSocketCentered(inches(1.25), inches(3.75), "Out", True))

module.add(TL1265(inches(0.1), inches(0.2)))
module.add(TL1265(inches(0.4), inches(0.5)))
module.add(TL1265(inches(0.1), inches(0.8)))
module.add(TL1265(inches(0.4), inches(1.1)))
module.add(TL1265(inches(0.1), inches(1.4)))
module.add(TL1265(inches(0.1), inches(1.8)))
module.add(TL1265(inches(0.4), inches(2.1)))
module.add(TL1265(inches(0.1), inches(2.4)))
module.add(TL1265(inches(0.4), inches(2.7)))
module.add(TL1265(inches(0.1), inches(3.0)))
module.add(TL1265(inches(0.4), inches(3.3)))
module.add(TL1265(inches(0.1), inches(3.6)))

# LEDButton = Button

# column = [0, 1, 0, 1, 0, 0, 1, 0, 1, 0, 1, 0]
# y = inches(.2)
# for i in range(12):
#     x = inches(.3) + inches(.3) * column[i]
#     if i != 0:
#         y += inches(.5) if column[i-1] == column[i] else inches(.32)
#     module.add(LEDButton(x, y))


module.save()
