import sys
sys.path.append("..")
sys.path.append("../..")
sys.path.append("../../..")

from faceplate_maker import *


module = Module.from_cli(8, global_y_offset=19, title="Clock")

module.add(OLEDSPI(inches(.3), -inches(.3), rotation=2))

module.add(Potentiometer(inches(.1), inches(.85), rotation=1))
module.add(TL1105SP(-inches(.4), inches(.95)))

jack_offset_y = inches(1.7)

i = 0
for y in range(4):
    for x in range(2):
        _x = inches(.6 * x) - inches(.3)
        _y = inches(.6 * y) + jack_offset_y
        i += 1
        module.add(JackSocketCentered(_x + inches(0), _y + inches(0), "", False, rotation=2))
        module.add(SmallLED(_x + inches(.15), _y - inches(.3)))


module.save()
