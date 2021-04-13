import sys
sys.path.append("..")
sys.path.append("../..")
sys.path.append("../../..")

from faceplate_maker import *
from svgwrite.path import Path

HP = 8

module = Module(HP, (0, 12), title="Quantizer", filename="quantizer_faceplate.svg", cosmetics=True)


def get_center(circle):
    return (circle.offset[0] + circle.position[0],
            circle.offset[1] + circle.position[1])

for y in [0, inches(2.3)]:
    module.add(JackSocket(inches(1.2), y + inches(0.0), "In", False))
    module.add(JackSocket(inches(1.2), y + inches(0.6), "Trig", False))
    module.add(JackSocket(inches(1.2), y + inches(1.2), "Out", True))

module.add(Button(inches(1.2), inches(1.95)))

LEDButton = BasicCircle(0, 0, inches(3/16))

column = [0, 1, 0, 1, 0, 0, 1, 0, 1, 0, 1, 0]
y = inches(.2)
for i in range(12):
    x = inches(.3) + inches(.3) * column[i]
    if i != 0:
        y += inches(.5) if column[i-1] == column[i] else inches(.32)
    module.add(LEDButton(x, y))


module.save()
