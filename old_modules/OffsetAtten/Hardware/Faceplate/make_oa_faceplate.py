import sys
sys.path.append("..")
sys.path.append("../..")
sys.path.append("../../..")

from faceplate_maker import *

offset_x = inches(4 * .2) / 2 - inches(.2)

module = Module(4, (offset_x, 26), title="O/A", filename="oa_faceplate.svg")

for y in [0, inches(2.1)]:
    def draw_text(context):
        container = context.g()
        
        group = container.add(context.g())
        group.translate(-2, y - 3.5)
        offset_text = context.text("Offset")
        offset_text.rotate(-90)
        group.add(offset_text)
        
        group2 = container.add(context.g())
        group2.translate(12, y + 4)
        offset_text = context.text("Atten")
        offset_text.rotate(90)
        group2.add(offset_text)
        
        return container
        
        
    module.draw(draw_text)
    module.add(Potentiometer(inches(.2), y + inches(0), color="blue"))
    module.add(Potentiometer(inches(0), y + inches(.7), color="white"))

    module.add(LED(inches(.5), y + inches(.6)))
    module.add(JackSocket(inches(0), y + inches(.9), "In", False))
    module.add(JackSocket(inches(.4), y + inches(.9), "Out", True))


module.save()
