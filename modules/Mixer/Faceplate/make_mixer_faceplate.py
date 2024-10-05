import sys
sys.path.append("..")
sys.path.append("../..")
sys.path.append("../../..")

from faceplate_maker import *
from svgwrite.path import Path

HP = 8

module = Module.from_cli(HP, global_y_offset=16, title="Mixer")


def get_center(circle):
    return (circle.offset[0] + circle.position[0],
            circle.offset[1] + circle.position[1])

y = 0
            
for i in range(5):
    jack = JackSocketCentered(-inches(0.55), y + inches(.2), "", False)
    module.add(jack)
    
    stroke_width = .6
    center = get_center(jack)
    path = Path(stroke="black", fill="none", stroke_width=stroke_width)
    path.push(f"M {center[0]} {center[1]}")
    if i < 4:
        path.push(f"h {inches(.45) - center[0]}")
        if i == 3:
            v_length = inches(0.8) - 3.7
        else:
            v_length = inches(0.8)
        path.push(f"v {v_length}")
    else:
        path.push(f"h {inches(.45) - center[0] - 3.7}")
    module.draw(lambda _: path)
    
    module.add(SmallSwitch(-inches(.15), y + inches(.2), rotation=1))

    if i == 4:
        module.add(JackSocketCentered(inches(0.45), y + inches(.2), "OUT", True))
        module.add(SmallLED(inches(0.6), y - inches(0.15), color="green"))
    else: 
        module.add(Potentiometer(inches(0.35), y + inches(0.5), style=PotStyle.SIFAM_MEDIUM))
        y += inches(.8)


module.save()
