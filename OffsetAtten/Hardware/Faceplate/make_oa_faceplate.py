import sys
sys.path.append("..")
sys.path.append("../..")
sys.path.append("../../..")

from faceplate_maker import *

offset_x = inches(4 * .2) / 2 - inches(.2)

module = Module(4, (offset_x, 26), title="O/A", filename="oa_faceplate.svg")

for y in [0, inches(2.1)]:
    module.add(Potentiometer(inches(.2), y + inches(0)))
    module.add(Potentiometer(inches(0), y + inches(.7)))

    module.add(LED(inches(.5), y + inches(.6)))
    module.add(JackSocket(inches(0), y + inches(.9), "In", False))
    module.add(JackSocket(inches(.4), y + inches(.9), "Out", True))


module.save()
