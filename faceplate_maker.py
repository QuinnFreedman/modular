try:
    import svgwrite
    from svgwrite import mm
    from svgwrite.mixins import Transform
except ImportError:
    print("This library requires svgwrite but it is not installed.")
    print("Install it with:")
    print("    python3 -m pip install --user svgwrite")
    import sys
    sys.exit(1)

import urllib.request
import base64

try:
    with urllib.request.urlopen("https://fonts.gstatic.com/s/ubuntu/v14/4iCv6KVjbNBYlgoCjC3jsGyN.woff2") as response:
        font_b64 = base64.b64encode(response.read())
        font_string = "url(\"data:application/font-woff;charset=utf-8;base64,{}\")".format(font_b64.decode('utf-8'))
except Exception as e:
    print("Warning: Unable to download font")
    font_string = "local('Ubuntu Medium'), local('Ubuntu-Medium')"

def inches(n):
    """Returns n inches in mm"""
    return n * 25.4

class Module:
    def __init__(self, hp, global_offset, title=None, filename="output.svg"):
        HP = inches(0.2)
        self.height = 128.5
        self.width = hp * HP - 1

        self.d = svgwrite.Drawing(filename=filename, size=(self.width * mm, self.height * mm))
        self.d.defs.add(self.d.style(content="@font-face {{ font-family: 'Ubuntu'; font-style: normal; font-weight: 500; src: {}; }}".format(font_string)))
        self.d.viewbox(width=self.width, height=self.height)
        self.outline = self.d.add(self.d.g(id="outline", fill="none", stroke="black"))
        self.stencil = self.d.add(self.d.g(id="stencil", font_family="Ubuntu", font_size=3))
        self.holes = self.d.add(self.d.g(id="throughholes", fill="black", stroke="none"))

        # Draw outline
        self.outline.add(
            self.d.rect(size=(self.width, self.height), stroke_width=1))
                        
        screw_hole_y = 3
        screw_hole_x = 1.5 * HP
        screw_hole_d = 3.2

        def screw_hole(x, y):
            self.holes.add(
                self.d.circle(center=(x, y), r=screw_hole_d/2,
                              stroke="none"))
            
        # draw left mounting holes
        screw_hole(screw_hole_x, screw_hole_y)
        screw_hole(screw_hole_x, self.height - screw_hole_y)

        # draw right mounting holes
        if hp > 6:
            screw_hole(self.width - screw_hole_x, screw_hole_y)
            screw_hole(self.width - screw_hole_x, self.height - screw_hole_y)

        # Draw title
        if title:
            self.stencil.add(
                self.d.text(title,
                    insert=(self.width / 2, 5),
                    font_size=5,
                    text_anchor="middle"
                ))

        if global_offset:
            self.holes = self.holes.add(self.d.g(id="throughholes_offset"))
            self.holes.translate(global_offset)
            self.stencil = self.stencil.add(self.d.g(id="stencil_offset"))
            self.stencil.translate(global_offset)

    def add(self, component):
        group = self.holes.add(self.d.g())
        group.translate(*component.position)
        for x in component.draw_holes(self.d):
            group.add(x)
            
        group = self.stencil.add(self.d.g())
        group.translate(*component.position)
        for x in component.draw_stencil(self.d):
            group.add(x)

    def save(self):
        self.d.save()


class Component:
    def __init__(self, position_x, position_y):
        self.position = (position_x, position_y)

    def draw_holes(self, context):
        return []

    def draw_stencil(self, context):
        return []


def BasicCircle(offset_x, offset_y, r):
    class BasicCircle(Component):
        def __init__(self, position_x, position_y, rotation=0):
            super(BasicCircle, self).__init__(position_x, position_y)
            if rotation not in [0, 1, 2, 3]:
                raise ValueError("rotation must be 0...3")

            if rotation == 0:
                self.offset = (offset_x, offset_y)
            elif rotation == 1:
                self.offset = (-offset_y, offset_x)
            elif rotation == 2:
                self.offset = (-offset_x, -offset_y)
            elif rotation == 3:
                self.offset = (offset_y, -offset_x)
                
            self.radius = r
            
        def draw_holes(self, context):
            return [context.circle(center=self.offset, r=self.radius)]
            
    return BasicCircle


class JackSocket(BasicCircle(0, 4.92, 4)):
    def __init__(self, x, y, label, is_output, rotation=0, font_size=None):
       super(JackSocket, self).__init__(x, y, rotation)
       self.label = label
       self.is_output = is_output
       self.font_size = font_size
       self.hole_radius = self.radius
       self.hole_center = self.offset
       
    def draw_holes(self, context):
        return [context.circle(center=self.hole_center, r=self.hole_radius)]

    def draw_stencil(self, context):
        text_props = {
            "insert": (self.hole_center[0], self.hole_center[1] + 8),
            "text_anchor": "middle",
        }

        if self.font_size:
            text_props["font_size"] = self.font_size
        
        if self.is_output:
            text_props["fill"] = "#ffffff"
            
        elements = []
        if self.is_output:
            padding = 1
            width = 2 * (self.hole_radius + padding)
            height = 15
            elements.append(context.rect(
                insert=(self.hole_center[0] - width/2, self.hole_center[1] - self.hole_radius - padding),
                size=(width, height),
                fill="#000000",
                rx=1.5))
            elements.append(context.circle(center=self.hole_center, r=self.hole_radius+.3, fill="#ffffff"))

        elements.append(context.text(self.label, **text_props))

        # elements.append(context.circle(center=(0, 0), r=.5, fill="red"))
        
        return elements


LED = BasicCircle(0, inches(.05), 2.5)


class Potentiometer(BasicCircle(inches(.15), inches(.3), 3)):
    def __init__(self, x, y, label=None, rotation=0, font_size=None):
        super(Potentiometer, self).__init__(x, y, rotation)
        self.label = label
        self.font_size = font_size

    def draw_stencil(self, context):
        elements = []
        text_props = {
            "insert": (self.offset[0], self.offset[1] + 11),
            "text_anchor": "middle",
        }

        if self.font_size:
            text_props["font_size"] = self.font_size

        if self.label:
            elements.append(context.text(self.label, **text_props))
        
        # elements.append(context.circle(center=(0, 0), r=.5, fill="red"))
        
        return elements


class OLED(Component):
    def __init__(self, x, y):
       super(OLED, self).__init__(x, y)

    def draw_holes(self, context):
        height = inches(1/2)
        width = inches(15/16)

        screen_offset = (inches(-(1/4 + 1/16)), inches(-(1/8 + 1/32)) - height)
        left_hole_offset_x = -inches(1/4)
        right_hole_offset_x = inches(0.3) - left_hole_offset_x
        bottom_hole_offset = -inches(1/32)
        top_hole_offset = bottom_hole_offset - inches(7/8)

        screw_hole_d = inches(3/32)

        elements = []

        for x in (left_hole_offset_x, right_hole_offset_x):
            for y in (top_hole_offset, bottom_hole_offset):
                elements.append(context.circle(center=(x, y), r=screw_hole_d/2))

        elements.append(context.rect(insert=screen_offset, size=(width, height)))

        # elements.append(context.circle(center=(0,0), r=.5))
        # elements.append(context.circle(center=(inches(.1),0), r=.5))
        # elements.append(context.circle(center=(inches(.2),0), r=.5))
        # elements.append(context.circle(center=(inches(.3),0), r=.5))

        return elements


if __name__ == "__main__":
    module = Module(8, (10, 30), title="Title")
    module.add(JackSocket(0, 0, "input", False))
    module.add(LED(inches(.3), inches(.3)))

    module.add(JackSocket(inches(.6), 0, "output", True))
    module.add(LED(inches(.9), inches(.3)))

    module.add(Potentiometer(inches(.5), inches(.9), "Level", rotation=0))
    
    module.add(Potentiometer(inches(.5), inches(2), "Level", rotation=1))
    
    module.add(JackSocket(0, inches(.9), "rotated", True, rotation=1))

    #module.add(OLED(inches(.2), inches(3)))

    module.save()
