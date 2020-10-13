import sys
sys.path.append("..")
sys.path.append("../..")
sys.path.append("../../..")

from faceplate_maker import *

from svgwrite.path import Path

if __name__ == "__main__":
    module = Module(10, (0, 20), title="RNG", debug=True)

    y = 0
    
    NUM_LEDS = 7
    CENTER_X = inches(10 * .2 / 2)
    for i in range(NUM_LEDS):
        x = inches(.3 * (i - NUM_LEDS // 2)) + CENTER_X
        module.add(LED(x, y))

    y += inches(1)
    module.add(Potentiometer(CENTER_X - inches(.5), y - inches(.1), label="Chaos"))
    module.add(Potentiometer(CENTER_X + inches(.4), y, label="Time"))

    y += inches(.9)
    module.add(Potentiometer(CENTER_X + inches(.4), y, label="Spread"))
    module.add(Potentiometer(CENTER_X - inches(.6), y, label="Bias"))

    y += inches(.3)
    module.add(Switch(CENTER_X - inches(.5), y, label="Bipolar/Unipolar"))
    module.add(Switch(CENTER_X + inches(.5), y, label="Trigger/Gate"))

    y += inches(.4)
    module.add(JackSocket(CENTER_X - inches(.7), y, label="Clock", is_output=False))
    module.add(JackSocket(CENTER_X - inches(.3), y, label="CV", is_output=False))
    module.add(JackSocket(CENTER_X + inches(.5), y, label="Bias", is_output=False))
    
    y += inches(.8)
    module.add(JackSocket(CENTER_X - inches(.5), y, label="Output", is_output=True, rotation=2))
    module.add(JackSocket(CENTER_X + inches(.3), y, label="A", is_output=True, rotation=2))
    module.add(JackSocket(CENTER_X + inches(.7), y, label="B", is_output=True, rotation=2))
    
    module.add(SmallLED(CENTER_X + inches(.2), y-inches(.5)))
    module.add(SmallLED(CENTER_X + inches(.8), y-inches(.5)))

    path = Path(stroke="black", fill="none", stroke_width=1)
    path.push(f"M {CENTER_X},40")
    path.push(f"v 50")
    module.draw(lambda _: path)

    module.save()
