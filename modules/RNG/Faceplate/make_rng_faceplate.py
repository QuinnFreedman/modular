import sys
sys.path.append("..")
sys.path.append("../..")
sys.path.append("../../..")

from faceplate_maker import *

from svgwrite.path import Path

module = Module.from_cli(10, global_y_offset=13, title="RNG")

NUM_LEDS = 7

for i in range(NUM_LEDS):
    x = inches(.25 * (i - NUM_LEDS // 2))
    module.add(SmallLED(x, inches(.1)))

module.add(Potentiometer(-inches(.6), inches(1), label="Chance", style=PotStyle.CHROMATIC_RED))
module.add(Potentiometer(inches(.4), inches(1), label="Time", style=PotStyle.ROGAN_PT_1S))
module.add(Potentiometer(-inches(.6), inches(2), label="Spread", style=PotStyle.CHROMATIC_WHITE_SMALL))
module.add(Potentiometer(inches(.4), inches(2), label="Bias", style=PotStyle.CHROMATIC_WHITE_SMALL))

module.add(Switch(-inches(.5), inches(2.5), left_text="Uni", right_text="Bi")) #label="Bipolar/Unipolar"))
module.add(Switch(inches(.5), inches(2.5), left_text="Trig", right_text="Gate")) #label="Trigger/Gate"))

module.add(JackSocket(-inches(.7), inches(2.9), label="Clock", is_output=False, label_above=True))
module.add(JackSocket(-inches(.3), inches(2.9), label="Enable", is_output=False, label_above=True))
module.add(JackSocket(-inches(.5), inches(3.8), label="Out", is_output=True, rotation=2))
module.add(SmallLED(-inches(.85), inches(3.3)))
module.add(SmallLED(-inches(.15), inches(3.3)))

module.add(JackSocket(inches(.5), inches(2.9), label="Bias", is_output=False, label_above=True))
module.add(JackSocket(inches(.3), inches(3.8), label="A", is_output=True, rotation=2))
module.add(JackSocket(inches(.7), inches(3.8), label="B", is_output=True, rotation=2))
module.add(SmallLED(inches(.85), inches(3.3)))
module.add(SmallLED(inches(.15), inches(3.3)))

path = Path(stroke="black", fill="none", stroke_width=.8)
path.push("M 0,41")
path.push("v 59")
module.draw(lambda _: path)

cursor_width = 3.5

cursor = Path(fill="black")
cursor.push("M 0,2")
cursor.push(f"l -{cursor_width / 2},-3")
cursor.push(f"h {cursor_width}")
cursor.push("z")
module.draw(lambda _: cursor)

module.save()
