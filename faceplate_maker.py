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

LOGO_DATAURL = "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='523.843' height='243.844' viewBox='0 0 138.6 64.517' xmlns:v='https://vecta.io/nano' style='background-color:white'%3E%3Cdefs%3E%3CclipPath id='A'%3E%3Cpath d='M115.46-14.165h44.634v93.277H115.46z' opacity='.997' fill='%23fff' fill-opacity='1' stroke='none' stroke-width='6' stroke-miterlimit='4' stroke-dasharray='none' stroke-opacity='1'/%3E%3C/clipPath%3E%3C/defs%3E%3Cpath d='M106.34 5.8c14.613 0 26.458 11.846 26.458 26.458s-11.846 26.458-26.458 26.458S85.175 48.133 69.3 32.258C85.175 16.383 91.73 5.8 106.34 5.8zm-74.083 0C17.645 5.8 5.8 17.645 5.8 32.258s11.846 26.458 26.458 26.458S53.425 48.133 69.3 32.258C53.425 16.383 46.87 5.8 32.258 5.8z' opacity='.996' fill='none' stroke='%23000' stroke-width='11.6'/%3E%3Cpath d='M61.07 16.1l-.012 7.31c-11.285-.015-22.57-.01-33.854.006-.252.113-1.024-.295-1.474-.392-1.062-.302-2.1-.79-3.214-.86-1.52.167-2.033 1.836-2.898 2.847l-5.24 7.26 7.64 10.43c2.34-.75 4.652-1.587 7.014-2.266 1.784 1.165 4 .55 5.98.68l26.06.005-.008 6.875c.335.967 1.505 1.018 2.31.908l42.5.014 11.63-4.5 14.182.158-.04-4.734 5.328-2.46.182-9.926-5.623-2.756-.04-4.62c-4.87-.323-9.83.16-14.6-.518-4.037-1.365-7.967-3.124-12.156-3.982h-43.68v.53z' opacity='.997' fill='%23fff'/%3E%3Cpath d='M65.26 19.777v7.825H31.763v9.312H65.26v7.825h39.835l11.64-4.527 10.735.13-.03-3.922 7.45-.4.134-7.253-7.65-.668-.03-3.922-10.347-.13-11.9-4.268zm-41.71 6.96l-4 5.535 4.03 5.505 6.482-2.133-.026-6.824z' opacity='.997'/%3E%3Cpath clip-path='url(%23A)' d='M105.833 5.292c14.613 0 26.458 11.846 26.458 26.458s-11.846 26.458-26.458 26.458S84.667 47.625 68.792 31.75C84.667 15.875 91.22 5.292 105.833 5.292zm-74.083 0c-14.613 0-26.458 11.846-26.458 26.458S17.137 58.208 31.75 58.208 52.917 47.625 68.792 31.75C52.917 15.875 46.363 5.292 31.75 5.292z' opacity='.996' fill='none' stroke='%23000' stroke-width='11.6' transform='translate(.508 .508)'/%3E%3Cpath d='M31.92 22.98h.275v18.014h-.275z' opacity='.997' stroke='%23fff' stroke-width='3.217'/%3E%3C/svg%3E"

HOLE_ALLOWANCE = .2  # mm 

try:
    import urllib.request
    import base64

    with urllib.request.urlopen("https://fonts.gstatic.com/s/ubuntu/v14/4iCv6KVjbNBYlgoCjC3jsGyN.woff2") as response:
        font_b64 = base64.b64encode(response.read())
        font_string = "url(\"data:application/font-woff;charset=utf-8;base64,{}\")".format(font_b64.decode('utf-8'))
except Exception as e:
    print("Warning: Unable to download font")
    font_string = "local('Ubuntu Medium'), local('Ubuntu-Medium')"

import math

def inches(n):
    """Returns n inches in mm"""
    return n * 25.4

class Module:
    def __init__(self, hp, global_offset, title=None, filename="output.svg", debug=False, cosmetics=False):
        HP = inches(0.2)
        self.height = 128.5
        self.width = hp * HP - .5

        self.d = svgwrite.Drawing(filename=filename, size=(self.width * mm, self.height * mm))

        self.d.add(
            self.d.rect(size=(self.width, self.height), fill="white", id="background"))
        
        self.d.defs.add(self.d.style(content="@font-face {{ font-family: 'Ubuntu'; font-style: normal; font-weight: 500; src: {}; }}".format(font_string)))
        self.d.viewbox(width=self.width, height=self.height)
        self.outline = self.d.add(self.d.g(id="outline", fill="none", stroke="black"))
        self.stencil = self.d.add(self.d.g(id="stencil", font_family="Ubuntu", font_size=3))
        self.holes = self.d.add(self.d.g(id="throughholes", fill="black", stroke="none"))
        
        self.debug = None
        if debug:
            self.debug = self.d.add(self.d.g(id="debug", fill="red", stroke="red"))
            
        self.cosmetics = None
        if cosmetics:
            self.cosmetics = self.d.add(self.d.g(id="cosmetics"))

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
            title_offset_y = 5
            if hp < 8:
                title_offset_y = 9
            self.stencil.add(
                self.d.text(title,
                    insert=(self.width / 2, title_offset_y),
                    font_size=5,
                    text_anchor="middle"
                ))

        # Draw logo
        logo_y = self.height - 9
        if hp < 8:
            logo_y = self.height - 12
        logo_width = min(self.width, 15)
            
        image = self.d.image(LOGO_DATAURL, insert=(self.width / 2 - logo_width / 2, logo_y), size=(logo_width, 8))
        image.fit("center", "middle", "meet")
        self.stencil.add(image)

        # Draw outline
        if cosmetics:
            pass
        elif debug:
            self.outline.add(
                self.d.rect(size=(self.width, self.height), stroke_width=1))
        else:
            length = 3
            lines = [
                ((0, 0), (0, length)),
                ((0, 0), (length, 0)),
                ((self.width, 0), (self.width - length, 0)),
                ((self.width, 0), (self.width, length)),
                ((0, self.height), (0, self.height - length)),
                ((0, self.height), (length, self.height)),
                ((self.width, self.height), (self.width, self.height - length)),
                ((self.width, self.height), (self.width - length, self.height)),
                ]
            for start, end in lines:
                self.stencil.add(self.d.line(start, end, stroke_width=1, stroke="black"))

        if self.debug:
            self.debug.add(self.d.line((self.width / 2, 0), (self.width / 2, self.height), stroke="green", stroke_dasharray="4,3", stroke_width=.5))
                        
        if global_offset:
            self.holes = self.holes.add(self.d.g(id="throughholes_offset"))
            self.holes.translate(global_offset)
            self.stencil = self.stencil.add(self.d.g(id="stencil_offset"))
            self.stencil.translate(global_offset)
            if self.debug:
                self.debug = self.debug.add(self.d.g(id="debug_offset"))
                self.debug.translate(global_offset)
            if self.cosmetics:
                self.cosmetics = self.cosmetics.add(self.d.g(id="cosmetics_offset"))
                self.cosmetics.translate(global_offset)

        if self.debug:
            for x in range(hp * 2):
                _x = x * inches(0.1)
                self.debug.add(self.d.line((_x, 0), (_x, inches(5)), stroke_width=0.1))
            for y in range(50):
                _y = y * inches(0.1)
                self.debug.add(self.d.line((0, _y), (inches(hp * .2), _y), stroke_width=0.1))

    def add(self, component):
        group = self.holes.add(self.d.g())
        group.translate(*component.position)
        for x in component.draw_holes(self.d):
            group.add(x)
            
        group = self.stencil.add(self.d.g())
        group.translate(*component.position)
        for x in component.draw_stencil(self.d):
            group.add(x)

        if self.debug and hasattr(component, "draw_debug"):
            group = self.debug.add(self.d.g())
            group.translate(*component.position)
            for x in component.draw_debug(self.d):
                group.add(x)
                
        if self.cosmetics and hasattr(component, "draw_cosmetics"):
            group = self.cosmetics.add(self.d.g())
            group.translate(*component.position)
            for x in component.draw_cosmetics(self.d):
                group.add(x)

    def draw(self, function):
        self.stencil.add(function(self.d))

    def save(self):
        self.d.save()


class Component:
    def __init__(self, position_x, position_y):
        self.position = (position_x, position_y)

    def draw_holes(self, context):
        return []

    def draw_stencil(self, context):
        return []
        
    def draw_cosmetics(self, context):
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
            
        def draw_debug(self, context):
            return [context.circle(center=(0,0), r=.6)]
            
    return BasicCircle


# The datasheet says this should be an offset of 4.92mm for the "Thonkicon" but it actually seems closer to 3.9
class JackSocket(BasicCircle(0, 3.92, 3 + HOLE_ALLOWANCE)):
    def __init__(self, x, y, label, is_output, rotation=0, font_size=None, label_above=False):
       super(JackSocket, self).__init__(x, y, rotation)
       self.label = label
       self.is_output = is_output
       self.font_size = font_size
       self.hole_radius = self.radius
       self.hole_center = self.offset
       self.label_above = label_above
       
    def draw_holes(self, context):
        return [context.circle(center=self.hole_center, r=self.hole_radius)]

    def draw_stencil(self, context):
        text_offset = 8
        if self.label_above:
            text_offset = -5.5
            
        text_props = {
            "insert": (self.hole_center[0], self.hole_center[1] + text_offset),
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

        return elements
        
    def draw_cosmetics(self, context):
        elements = []
        gradient = context.linearGradient(
            (1, 0),
            (0, 1),
        )
        gradient.add_stop_color(-1, "white")
        gradient.add_stop_color(2, "black")
        context.defs.add(gradient)
        elements.append(draw_bumpy_circle(
            context,
            self.offset,
            self.radius + .4,
            self.radius + .6,
            18,
            fill=gradient.get_paint_server()
        ))

        ring_thickness = .8
        gradient = context.radialGradient((.5, .5), .5)
        gradient.add_stop_color(1 - ring_thickness / self.radius, "black")
        gradient.add_stop_color(1 - ring_thickness / self.radius / 2, "white")
        gradient.add_stop_color(1, "#444")
        context.defs.add(gradient)
        elements.append(context.circle(
            self.offset,
            self.radius,
            fill=gradient.get_paint_server()
        ))
        
        gradient = context.linearGradient(
            (1, 0),
            (0, 1),
        )
        gradient.add_stop_color(0, "black")
        gradient.add_stop_color(1, "#333")
        context.defs.add(gradient)
        elements.append(context.circle(
            self.offset,
            self.radius - ring_thickness,
            fill=gradient.get_paint_server()
        ))
        return elements


def draw_bumpy_circle(context, center, r1, r2, n, **kwargs):
    n *= 4
    path = context.path(**kwargs)
    for i in range(n):
        if i % 4 < 2:
            r = r1
        else:
            r = r2
        theta = 2 * math.pi / n * i
        x = center[0] + math.cos(theta) * r
        y = center[1] + math.sin(theta) * r
        path.push(f"{'M' if i == 0 else 'L'} {x} {y}")
    path.push("z")
    return path

class Switch(BasicCircle(0, 0, inches(1/8) + HOLE_ALLOWANCE)):
    def __init__(self, x, y, label=None, left_text=None, right_text=None, font_size=None):
       super(Switch, self).__init__(x, y, 0)
       self.label = label
       self.font_size = font_size
       self.left_text = left_text
       self.right_text = right_text
       self.hole_radius = self.radius
       self.hole_center = self.offset

    def draw_stencil(self, context):
        text_props = {
            "text_anchor": "middle",
        }

        if self.font_size:
            text_props["font_size"] = self.font_size

        if self.font_size:
            approx_text_size = self.font_size
        else:
            approx_text_size = 3

        elements = []

        if self.label:
            elements.append(context.text(self.label,
                insert=(self.hole_center[0], self.hole_center[1] + 8),
                **text_props))
        if self.left_text:
            elements.append(context.text(self.left_text,
                insert=(self.hole_center[0] - 8, self.hole_center[1] + approx_text_size / 2),
                **text_props))
        if self.right_text:
            elements.append(context.text(self.right_text,
                insert=(self.hole_center[0] + 8, self.hole_center[1] + approx_text_size / 2),
                **text_props))
            
        return elements
        
    def draw_cosmetics(self, context):
        elements = []
        gradient = context.linearGradient(
            (1, 0),
            (0, 1),
        )
        gradient.add_stop_color(-1, "white")
        gradient.add_stop_color(2, "black")
        context.defs.add(gradient)
        elements.append(context.circle(
            self.offset,
            self.radius + .35,
            fill=gradient.get_paint_server()
        ))

        ring_thickness = .8
        gradient = context.radialGradient((.5, .5), .5)
        gradient.add_stop_color(1 - ring_thickness / self.radius, "black")
        gradient.add_stop_color(1 - ring_thickness / self.radius / 2, "white")
        gradient.add_stop_color(1, "#444")
        context.defs.add(gradient)
        elements.append(context.circle(
            self.offset,
            self.radius,
            fill=gradient.get_paint_server()
        ))
        
        elements.append(context.circle(
            self.offset,
            self.radius - ring_thickness,
            fill="#111"
        ))

        angle = 0#-.2
        length = 4
        width = 2.2
        spread = .1
        rounding = 2.5
        wide_width = width + 2 * length * math.sin(spread)

        gradient = context.radialGradient((.8, .5), .6)
        gradient.add_stop_color(0, "#eee")
        gradient.add_stop_color(1, "#111")
        context.defs.add(gradient)

        path = context.path(fill=gradient.get_paint_server())
        y = math.sin(angle - math.pi / 2) * width / 2
        x = math.cos(angle - math.pi / 2) * width / 2
        path.push(f"M {self.offset[0] + x} {self.offset[1] + y}")
        
        y = math.sin(angle - spread) * length
        x = math.cos(angle - spread) * length
        path.push(f"l {x} {y}")
        
        y = math.sin(angle + math.pi / 2) * wide_width
        x = math.cos(angle + math.pi / 2) * wide_width
        
        cy1 = math.sin(angle - spread) * rounding
        cx1 = math.cos(angle - spread) * rounding
        
        cy2 = y + math.sin(angle + spread) * rounding
        cx2 = x + math.cos(angle + spread) * rounding
        path.push(f"c {cx1} {cy1} {cx2} {cy2} {x} {y}")
        
        y = math.sin(angle + spread + math.pi) * length
        x = math.cos(angle + spread + math.pi) * length
        path.push(f"l {x} {y}")
        path.push(f"z")

        
        elements.append(path)
        
        return elements

class SmallLED(BasicCircle(0, inches(.05), 1.5 + HOLE_ALLOWANCE)):
    def __init__(self, x, y, rotation=0, font_size=None, color="red"):
       super(SmallLED, self).__init__(x, y, rotation)
       self.color = color


class LED(BasicCircle(0, inches(.05), 2.5 + HOLE_ALLOWANCE)):
    def __init__(self, x, y, rotation=0, font_size=None, color="red"):
       super(LED, self).__init__(x, y, rotation)
       self.color = color


def draw_led_cosmetic(self, context):
    gradient = context.radialGradient((.5, .5), .5)
    gradient.add_stop_color(0, "white")
    gradient.add_stop_color(1, self.color)
    context.defs.add(gradient)
    elements = []
    elements.append(context.circle(center=self.offset,
            r=self.radius,
            fill=gradient.get_paint_server()))

    highlight_center = (self.offset[0] + self.radius / 3, self.offset[1] - self.radius / 2)
    highlight = context.ellipse(
            center=highlight_center,
            r=(self.radius / 2, self.radius / 3),
            fill="white",
            opacity=0.8)
    highlight.rotate(20, center=highlight_center)
    elements.append(highlight)
    return elements

    
setattr(SmallLED, 'draw_cosmetics', draw_led_cosmetic)
setattr(LED, 'draw_cosmetics', draw_led_cosmetic)


def draw_button_cosmetic(self, context):
    gradient = context.radialGradient((.5, .5), .5)
    gradient.add_stop_color(0, "#999")
    gradient.add_stop_color(1, "#111")
    context.defs.add(gradient)
    elements = []
    elements.append(context.circle(center=self.offset,
            r=self.radius,
            fill=gradient.get_paint_server()))
    highlight_center = (self.offset[0] + self.radius / 3, self.offset[1] - self.radius / 2)
    highlight = context.ellipse(
            center=highlight_center,
            r=(self.radius / 3, self.radius / 5),
            fill="white",
            opacity=0.3)
    highlight.rotate(30, center=highlight_center)
    elements.append(highlight)
    return elements
    

Button = BasicCircle(0, 0, 3.5)
setattr(Button, 'draw_cosmetics', draw_button_cosmetic)


class Potentiometer(BasicCircle(inches(.1), inches(-.3), 3.5 + HOLE_ALLOWANCE)):
    def __init__(self, x, y, label=None, rotation=0, font_size=None, color="white", text_offset=12, cosmetic_radius=None):
        super(Potentiometer, self).__init__(x, y, rotation)
        self.label = label
        self.font_size = font_size
        self.color = color
        self.text_offset = text_offset
        self.cosmetic_radius = cosmetic_radius

    def draw_stencil(self, context):
        elements = []
        text_props = {
            "insert": (self.offset[0], self.offset[1] + self.text_offset),
            "text_anchor": "middle",
        }

        if self.font_size:
            text_props["font_size"] = self.font_size

        if self.label:
            elements.append(context.text(self.label, **text_props))
        
        return elements
        
    def draw_cosmetics(self, context):
        colors = {
            "yellow": "#f5d400",
            "blue": "#3b75ff",
            "red": "#ed2222",
            "green": "#5ece1c",
            "black": "#444",
            "white": "#eee"
        }
        border_width = 2
        marker_width = 2
        if self.cosmetic_radius:
            base_radius = self.cosmetic_radius
        else:
            base_radius = inches(1/4)
        
        top_radius = base_radius - .5
        gradient = context.linearGradient(
            (1, 0),
            (0, 1),
        )
        gradient.add_stop_color(0, "white")
        gradient.add_stop_color(1, "black")
        context.defs.add(gradient)
        elements = []
        elements.append(context.circle(center=self.offset,
                r=base_radius + inches(1/16),
                fill=gradient.get_paint_server()))
        elements.append(context.circle(center=self.offset,
                r=top_radius,
                fill=colors[self.color],
                stroke="black",
                stroke_width=border_width))
        tip_offset = (top_radius ** 2 / 2) ** (1/2)
        tip_size = (marker_width ** 2 / 2) ** (1/2)
        marker_tip =  (self.offset[0] - tip_offset, self.offset[1] - tip_offset)
        elements.append(context.line(self.offset,
            marker_tip,
            stroke="black",
            stroke_width=marker_width,
            stroke_linecap="round"
            ))
        square_offset = ((border_width / 2) ** 2 / 2) ** (1/2)
        elements.append(context.rect(
            (marker_tip[0] - square_offset - tip_size / 2, marker_tip[1] - square_offset - tip_size / 2),
            (tip_size, tip_size),
            fill="black"
        ))
        return elements


class OLED(Component):
    def __init__(self, x, y):
       super(OLED, self).__init__(x, y)
       self.screen_height = inches(1/2)
       self.screen_width = inches(15/16)
       self.screen_offset = (inches(-(1/4 + 1/16)), inches(-(1/8 + 1/32)) - self.screen_height)

    def draw_holes(self, context):
        height = self.screen_height 
        width = self.screen_width 

        left_hole_offset_x = -inches(1/4)
        right_hole_offset_x = inches(0.3) - left_hole_offset_x
        bottom_hole_offset = -inches(1/32)
        top_hole_offset = bottom_hole_offset - inches(7/8)

        screw_hole_d = inches(3/32)

        elements = []

        for x in (left_hole_offset_x, right_hole_offset_x):
            for y in (top_hole_offset, bottom_hole_offset):
                elements.append(context.circle(center=(x, y), r=screw_hole_d/2))

        elements.append(context.rect(insert=self.screen_offset, size=(width, height)))


        return elements
        
    def draw_debug(self, context):
        return [
            context.circle(center=(0,0), r=.5),
            context.circle(center=(inches(.1),0), r=.5),
            context.circle(center=(inches(.2),0), r=.5),
            context.circle(center=(inches(.3),0), r=.5)
        ]

    def draw_cosmetics(self, context):
        clip_path = context.defs.add(context.clipPath())
        clip_path.add(context.rect(insert=self.screen_offset, size=(self.screen_width, self.screen_height)))
        
        elements = []
        
        elements.append(context.rect(insert=self.screen_offset, size=(self.screen_width, self.screen_height), fill="black"))
        elements.append(context.ellipse(
            (self.screen_offset[0] + self.screen_width, self.screen_offset[1]),
            r=(self.screen_width / 2, self.screen_height / 2),
            fill="white", opacity=.5,
            clip_path=f"url(#{clip_path.get_id()})"))

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
