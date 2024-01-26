import sys
sys.path.append("..")
sys.path.append("../..")
sys.path.append("../../..")

from faceplate_maker import *
from svgwrite.path import Path

HEADPHONES_SVG = "M 50.0003 0 C 29.0053 4e-06 11.8974 17.1079 11.8974 38.1029 L 11.8974 41.2349 C 4.89548 45.0444 4.7e-05 52.4958 4.7e-05 61.1767 C 4.7e-05 73.5564 9.65832 83.884 21.7343 83.884 L 25.7684 83.884 L 25.7684 38.4693 L 21.7343 38.4693 C 21.1052 38.4693 20.5805 38.7926 19.9655 38.8475 L 19.9655 38.1029 C 19.9655 21.4682 33.3656 8.06812 50.0003 8.06812 C 66.635 8.06812 80.0312 21.4682 80.0312 38.1029 L 80.0312 38.8475 C 79.4165 38.7927 78.8912 38.4693 78.2624 38.4693 L 74.2283 38.4693 L 74.2283 42.5034 L 74.2283 83.884 L 78.2624 83.884 C 90.3384 83.884 100.001 73.5564 100.001 61.1767 C 100.001 52.494 95.1036 45.0396 88.0993 41.2309 L 88.0993 38.1029 C 88.0993 17.1079 70.9954 4e-06 50.0003 0 Z"

HP = 8

module = Module.from_cli(HP, global_y_offset=17, title="Output")

y = inches(.45)

module.add(Potentiometer(inches(.2), y, style=PotStyle.CHROMATIC_WHITE_SMALL, label="Phones"))
module.add(Potentiometer(-inches(.3), y + inches(.6), style=PotStyle.CHROMATIC_WHITE, label="Line out"))

y += inches(1.1)

module.add(JackSocketCentered(-inches(0.2), y, "In L", False))
module.add(JackSocketCentered(inches(0.2), y, "In R", False))

big_jack_start = y + inches(.75)
big_jack_pitch = inches(.65) # .675?

module.add(JackSocketQuarterInch(0, big_jack_start, "", False))
module.add(JackSocketQuarterInch(0, big_jack_start + big_jack_pitch, "L", False, text_offset=(-5, -5)))
module.add(JackSocketQuarterInch(0, big_jack_start + big_jack_pitch * 2, "R", False, text_offset=(-5, -5)))
module.draw(lambda _: Path(
    d=HEADPHONES_SVG,
    stroke_width=0,
    fill="black",
    transform=f"translate({-7}, {big_jack_start-7}) scale({2.5/100})",
    ))

led_midpoint = big_jack_start + big_jack_pitch
led_pitch = inches(.25)

for x in (-1, 1):
    x = x * inches(.45)
    colors = ["#f00", "#fcca05", "#2fdd04", "#2fdd04", "#2fdd04"]
    for i, color in enumerate(colors):
        module.add(SmallLED(x, led_midpoint + ((i - 2) * led_pitch) - inches(.05), color=color))


module.save()
