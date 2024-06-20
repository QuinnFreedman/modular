import sys
sys.path.append("..")
sys.path.append("../..")
sys.path.append("../../..")

from faceplate_maker import *
from svgwrite.path import Path

module = Module.from_cli(4, global_y_offset=13, title="Drift", title_size=4.6)

pot_spacing = inches(.9)

y = inches(.6)
speed_pot = Potentiometer(-inches(0.1), y, label="Speed", color=PotColor.RED)
y += pot_spacing
texture_pot = Potentiometer(-inches(0.1), y, label="Texture", color=PotColor.ORANGE)
module.add(speed_pot)
module.add(texture_pot)
y += pot_spacing
module.add(Potentiometer(-inches(0.1), y, label="Atten", color=PotColor.YELLOW))

y += inches(.6)

speed_jack = JackSocketCentered(-inches(0.2), y, "", False, rotation=2)
texture_jack = JackSocketCentered(inches(0.2), y, "", False, rotation=2)
module.add(speed_jack)
module.add(texture_jack)

y += inches(.6)
module.add(JackSocketCentered(inches(0.2), y, "Out", True, rotation=2))
module.add(LED(-inches(0.2), y-inches(.05)))

# line = Path(stroke="black", fill="none", stroke_width=stroke_width)
# line.push(f"M {-inches(0.2)},{y}")
# line.push(f"h {inches(0.4)}")
# module.draw(lambda _: line)

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
