import sys
sys.path.append("..")
sys.path.append("../..")
sys.path.append("../../..")

from faceplate_maker import *
from svgwrite.path import Path

HP = 4

module = Module.from_cli(HP, global_y_offset=15, title="O/A", hide_logo=True)

for offset in [0, inches(2.15)]:
    module.add(Potentiometer( inches(.00), offset + inches(.4), style=PotStyle.SIFAM_MEDIUM, color=PotColor.WHITE))
    module.add(Potentiometer(-inches(.20), offset + inches(1.2), style=PotStyle.SIFAM_MEDIUM, color=PotColor.BLUE))
    module.draw(lambda ctx: ctx.text("Offset",
                    insert=(-(offset + inches(.1)), -inches(.25)),
                    font_size=3,
                    text_anchor="middle",
                    transform="rotate(-90)"
                ))
    module.draw(lambda ctx: ctx.text("Atten",
                    insert=(offset + inches(.9), -inches(.25)),
                    font_size=3,
                    text_anchor="middle",
                    transform="rotate(90)"
                ))

    module.add(JackSocketCentered(-inches(0.2), offset + inches(1.55), "In", False, rotation=2))
    module.add(JackSocketCentered(inches(0.2), offset + inches(1.55), "Out", True, rotation=2))

    module.add(SmallLED(inches(.3), offset + inches(1.2), color="green"))

module.save()
