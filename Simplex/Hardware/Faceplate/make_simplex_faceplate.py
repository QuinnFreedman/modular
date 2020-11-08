import sys
sys.path.append("..")
sys.path.append("../..")
sys.path.append("../../..")

from faceplate_maker import *

HP = 4

module = Module(HP, (inches(-.015), 27.5), title="Simplex", filename="simplex_faceplate.svg")

y = 0
speed_pot = Potentiometer(inches(.3), y, label="Speed")
y += inches(.9)
texture_pot = Potentiometer(inches(.3), y, label="Texture")
module.add(speed_pot)
module.add(texture_pot)
y += inches(.9)
module.add(Potentiometer(inches(.3), y, label="Atten"))

y += inches(.4)

speed_jack = JackSocket(inches(0.2), y, "", False)
texture_jack = JackSocket(inches(0.6), y, "", False)
module.add(speed_jack)
module.add(texture_jack)

y += inches(.5)

module.add(LED(inches(.2), y, color="blue"))
module.add(LED(inches(.6), y, color="blue"))

y += inches(.5)

module.add(JackSocket(inches(0.2), y, "A", True, rotation=2))
module.add(JackSocket(inches(0.6), y, "B", True, rotation=2))

def unpack(tup):
    return ",".join(map(str, tup))

def get_center(circle):
    return (circle.offset[0] + circle.position[0],
            circle.offset[1] + circle.position[1])

texture_pot_center = get_center(texture_pot)
texture_cv_center = get_center(texture_jack)
speed_pot_center = get_center(speed_pot)
speed_cv_center = get_center(speed_jack)

base_control = 40
tip_control = 15
stroke_width = .6

from svgwrite.path import Path
path = Path(stroke="black", fill="none", stroke_width=stroke_width)
path.push(f"M {unpack(texture_pot_center)}")
path.push(f"C {texture_pot_center[0] + tip_control},{texture_pot_center[1] + tip_control},"
         +f"{texture_cv_center[0]},{texture_cv_center[1] - base_control},"
         +f"{unpack(texture_cv_center)}")
module.draw(lambda _: path)
path2 = Path(stroke="black", fill="none", stroke_width=stroke_width)
path2.push(f"M {unpack(speed_pot_center)}")
path2.push(f"C {speed_pot_center[0] - tip_control},{speed_pot_center[1] + tip_control},"
         +f"{speed_cv_center[0]},{speed_cv_center[1] - base_control},"
         +f"{unpack(speed_cv_center)}")
module.draw(lambda _: path2)

module.save()
