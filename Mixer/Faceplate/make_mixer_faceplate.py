import sys
sys.path.append("..")
sys.path.append("../..")
sys.path.append("../../..")

from faceplate_maker import *
from svgwrite.path import Path

HP = 8

module = Module(HP, (inches(.2), 16), title="Mixer", filename="mixer_faceplate.svg")


def get_center(circle):
    return (circle.offset[0] + circle.position[0],
            circle.offset[1] + circle.position[1])

y = 0
            
for i in range(5):
    jack = JackSocket(inches(0.0), y + inches(0.025), "", False)
    module.add(jack)
    
    stroke_width = .6
    center = get_center(jack)
    path = Path(stroke="black", fill="none", stroke_width=stroke_width)
    path.push(f"M {center[0]} {center[1]}")
    path.push(f"H {inches(1.05)}")
    path.push(f"V {inches(0.8)}")
    module.draw(lambda _: path)
    
    module.add(Switch(inches(.4), y + inches(.2)))

    if i == 4:
        module.add(JackSocket(inches(1.025), y + inches(0.05), "OUT", True))
    else: 
        module.add(Potentiometer(inches(0.75), y + inches(0.1), rotation=1))
        
    y += inches(.8)


module.save()
