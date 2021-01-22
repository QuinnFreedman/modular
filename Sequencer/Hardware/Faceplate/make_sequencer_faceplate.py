import sys
sys.path.append("..")
sys.path.append("../..")
sys.path.append("../../..")

from faceplate_maker import *
from svgwrite.path import Path
import math

HP = 26

module = Module(HP, (inches(.45), 12), title="Sequencer", filename="sequencer_faceplate.svg")


def get_center(circle):
    return (circle.offset[0] + circle.position[0],
            circle.offset[1] + circle.position[1])

"""
pad_x = .5
pad_y = 6
height = out_jack_max_y - out_jack_min_y + 2 * pad_y
width = out_jack_right_x - out_jack_left_x + 2 * pad_x
box_x = out_jack_left_x - pad_x
box_y = out_jack_min_y - pad_y
"""
height, width, box_x, box_y = 85.66, 18.78, 98.56, -3.46
module.draw(lambda context: context.rect((box_x, box_y), (width, height), rx=3) )

out_jack_min_y =  float("Infinity")
out_jack_max_y = -float("Infinity")

row_spacing = inches(.9)

y = 0

for i in range(4):
    input_jack = JackSocket(inches(0.0), y + inches(.025), "", False)
    module.add(input_jack)
    
    stroke_width = .6
    center = get_center(input_jack)
    path = Path(stroke="black", fill="none", stroke_width=stroke_width)
    path.push(f"M {center[0]} {center[1]}")
    path.push(f"h {inches(0.6)}")
    module.draw(lambda _: path)
    

    x = inches(-.3)
    for j in range(4):
        x += inches(.9)
        module.add(Potentiometer(x, y + inches(0.5)))
        module.add(SmallLED(x + inches(.4), y + inches(0.5), rotation=3))

    out_jack_left_x = x + inches(.6)
    out_jack_left_y = y + inches(.1)
    out_jack_right_x = x + inches(1.3)
    out_jack_right_y = y + inches(0.3)
    module.add(JackSocket(out_jack_left_x, out_jack_left_y, "", True, rotation=3))
    module.add(JackSocket(out_jack_right_x, out_jack_right_y, "", True, rotation=1))
    module.add(SmallLED(x + inches(.75), y + inches(0.35)))

    out_jack_min_y = min(out_jack_min_y, out_jack_left_y)
    out_jack_max_y = max(out_jack_max_y, out_jack_right_y)
        
    y += row_spacing


mode_select = Potentiometer(-inches(.1), y + inches(0.5), color="black")
module.add(mode_select)

path = Path(stroke="black", fill="none", stroke_width=stroke_width)
slice = 2 * math.pi / 12
center = get_center(mode_select)
stroke_width = .6

theta = slice * (6.5 + 0)
r = 10
path.push(f"M {center[0]} {center[1]}")
path.push(f"L {center[0] + r * math.cos(theta)} {center[1] + r * math.sin(theta)}")
path.push(f"v -14")

module.draw(lambda ctx: ctx.text("4x4", insert=(center[0] - 8, center[1] - 18), text_anchor="middle"))

theta = slice * (6.5 + 1)
r = 10
path.push(f"M {center[0]} {center[1]}")
path.push(f"L {center[0] + r * math.cos(theta)} {center[1] + r * math.sin(theta)}")
path.push(f"v -6")

module.draw(lambda ctx: ctx.text("2x8", insert=(center[0] - 6, center[1] - 14), text_anchor="middle"))

theta = slice * (6.5 + 2)
r = 10
path.push(f"M {center[0]} {center[1]}")
path.push(f"L {center[0] + r * math.cos(theta)} {center[1] + r * math.sin(theta)}")

module.draw(lambda ctx: ctx.text("1x16", insert=(center[0] - 3.5, center[1] - 10), text_anchor="middle"))

theta = slice * (6.5 + 3)
r = 10
path.push(f"M {center[0]} {center[1]}")
path.push(f"L {center[0] + r * math.cos(theta)} {center[1] + r * math.sin(theta)}")
module.draw(lambda ctx: ctx.rect((center[0] + 0.5, center[1] - 13), (6, 4), rx=1))
module.draw(lambda ctx: ctx.text("2x4", insert=(center[0] + 3.5, center[1] - 10), text_anchor="middle", fill="white"))

theta = slice * (6.5 + 4)
r = 10
path.push(f"M {center[0]} {center[1]}")
path.push(f"L {center[0] + r * math.cos(theta)} {center[1] + r * math.sin(theta)}")
path.push(f"v -7")

module.draw(lambda ctx: ctx.rect((center[0] + 4, center[1] - 17.5), (6, 4), rx=1))
module.draw(lambda ctx: ctx.text("2x8", insert=(center[0] + 7, center[1] - 14.5), text_anchor="middle", fill="white"))

theta = slice * (6.5 + 5)
r = 10
path.push(f"M {center[0]} {center[1]}")
path.push(f"L {center[0] + r * math.cos(theta)} {center[1] + r * math.sin(theta)}")
path.push(f"v -9.5")
path.push(f"h 1.5")

module.draw(lambda ctx: ctx.text("Matrix", insert=(center[0] + 16, center[1] - 11), text_anchor="middle"))

module.draw(lambda _: path)

y -= inches(.2)
x = inches(-.2)
for j in range(4):
    x += inches(.9)
    jack = JackSocket(x, y, "", False)  
    module.add(jack)
    module.add(Button(x, y + inches(.65)))
    
    center = get_center(jack)
    path = Path(stroke="black", fill="none", stroke_width=stroke_width)
    path.push(f"M {center[0]} {center[1]}")
    path.push(f"v {inches(0.6)}")
    module.draw(lambda _: path)


x += inches(.9)
random_jack = JackSocket(x, y, "", False)
module.add(random_jack)

center = get_center(random_jack)
module.draw(lambda ctx: ctx.text("Rand.",
                insert=(center[0] - 6, center[1] + 1),
                text_anchor="end"))


module.add(Switch(x, y + inches(.65), label="CV / Q. / P."))


center_y = out_jack_min_y + row_spacing * 1.5 + inches(.1)
center_x = out_jack_left_x + random_jack.radius + 1.5
module.draw(lambda ctx: ctx.text("- outs -",
                insert=(-center_y, center_x),
                text_anchor="middle",
                transform=f"rotate(-90)",
                fill="white"))
center_y += inches(.1)
center_x = out_jack_right_x - random_jack.radius - 0.5
module.draw(lambda ctx: ctx.text("- triggers -",
                insert=(-center_y, center_x),
                text_anchor="middle",
                transform=f"rotate(-90)",
                fill="white"))

module.save()
