import sys
sys.path.append("..")
sys.path.append("../..")
sys.path.append("../../..")

from faceplate_maker import *

offset_x = inches(8 * .2) / 2 - inches(.35)

module = Module(8, (offset_x, 11), title="Clock", filename="clock_faceplate.svg")

module.add(OLED(inches(.2), inches(.9)))

module.add(Potentiometer(inches(.9), inches(1.4), rotation=3))

jack_start_y = inches(1.7)

i = 0
for y in range(4):
    for x in range(2):
        _x = inches(.6 * x)
        _y = inches(.6 * y) + jack_start_y
        i += 1
        module.add(JackSocket(_x + inches(0), _y + inches(0), str(i), False))
        module.add(SmallLED(_x + inches(.3), _y + inches(.3)))


module.save()
