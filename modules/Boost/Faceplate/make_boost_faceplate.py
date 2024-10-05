import sys
sys.path.append("..")
sys.path.append("../..")
sys.path.append("../../..")

from faceplate_maker import *

module = Module.from_cli(4, global_y_offset=12, title="Boost")

module.add(Potentiometer(inches(-.1), inches(.8), label="Drive", color=PotColor.MAGENTA))
module.add(Potentiometer(inches(-.1), inches(1.7), label="Tone", color=PotColor.WHITE))

module.add(JackSocketCentered(inches(0), inches(2.8), "In", False, rotation=2))
module.add(JackSocketCentered(inches(0), inches(2.8 + 0.7), "Out", True, rotation=2))


module.save()
