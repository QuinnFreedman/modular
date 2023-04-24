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

try:
    import lxml.etree as ET
except ImportError:
    print("This tool uses lxml but it is not installed.")
    print("Install it with:")
    print("    python3 -m pip install --user lxml")
    print("Falling back to using etree. This may work but the output might be unreadable to some programs.")
    import xml.etree.ElementTree as ET

LOGO_PATH = "M 23.273995,0 C 10.46956,0 0,10.46956 0,23.273995 0,36.078431 10.46956,46.54799 23.273995,46.54799 c 6.2817,0 11.379649,-2.548635 15.730632,-6.245496 1.538657,-1.307338 3.076198,-2.787732 4.658303,-4.38762 l -0.0086,-5.869298 -5.989355,0.0026 c -1.483874,1.505733 -2.846746,2.828266 -4.080397,3.876451 -3.512556,2.984484 -6.049447,4.254142 -10.310608,4.254142 -8.281306,0 -14.9047822,-6.623476 -14.9047822,-14.904783 0,-8.281306 6.6234762,-14.9047823 14.9047822,-14.9047823 4.261161,0 6.798052,1.2696585 10.310608,4.2541413 1.233651,1.048185 2.596523,2.370718 4.080397,3.876452 l 5.989355,0.0026 0.0086,-5.869304 C 42.080825,9.0332294 40.543284,7.5528356 39.004627,6.2454977 34.653644,2.5486366 29.555695,0 23.273995,0 Z M 76.724807,0 C 70.443279,1.0197146e-4 65.345444,2.5487105 60.99455,6.2454977 59.387003,7.6113677 57.781031,9.1642634 56.123352,10.847502 H 68.695628 C 71.17209,9.1299217 73.43594,8.3692721 76.724807,8.369213 h 3.82e-4 c 6.200192,0 11.470469,3.712943 13.728089,9.054499 l -6.041554,-0.07531 -8.585089,-3.079684 -28.740996,3.81e-4 v 5.645584 H 24.399983 v 6.718636 h 22.685649 v 5.645592 l 28.740995,3.82e-4 8.58509,-3.079684 6.041553,-0.07531 c -2.25762,5.341544 -7.527894,9.054489 -13.728089,9.054489 h -3.82e-4 c -3.288867,-5.9e-5 -5.552715,-0.760709 -8.029179,-2.47829 H 56.123352 c 1.657679,1.68324 3.263651,3.236135 4.871198,4.602006 4.350894,3.696786 9.448729,6.245395 15.730257,6.245496 h 3.82e-4 C 89.52963,46.548 99.999184,36.07844 99.999184,23.274005 99.999176,10.46956 89.52962,0 76.725181,0 Z m -59.711596,19.295384 -2.890653,3.978611 2.890653,3.978612 4.677319,-1.51971 v -4.917803 z"

HOLE_ALLOWANCE = .4  # mm 

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
import subprocess
import random
import string

def inches(n):
    """Returns n inches in mm"""
    return n * 25.4

class Module:
    def __init__(self, hp, global_y_offset=0, title=None, filename="output.svg", debug=False, cosmetics=False, outline=None, title_size=5):
        HP = inches(0.2)
        self.tolerance = 0.5
        self.height = 128.5
        self.width = hp * HP - self.tolerance

        self.d = svgwrite.Drawing(filename=filename.replace(" ", "_"), size=(self.width * mm, self.height * mm))

        self.d.add(
            self.d.rect(size=(self.width, self.height), fill="white", id="background"))
        
        self.d.defs.add(self.d.style(id="font-style", content="@font-face {{ font-family: 'Ubuntu'; font-style: normal; font-weight: 500; src: {}; }}".format(font_string)))
        self.d.viewbox(width=self.width, height=self.height)
        self.outline = self.d.add(self.d.g(id="outline", fill="none", stroke="black"))
        self.stencil = self.d.add(self.d.g(id="stencil", font_family="Ubuntu", font_size=3))
        self.holes = self.d.add(self.d.g(id="throughholes", fill="black", stroke="none"))
        self.inkscape_actions = []
        self.post_process = []
        
        self.debug = None
        if debug:
            self.debug = self.d.add(self.d.g(id="debug", fill="red", stroke="red"))
            
        self.cosmetics = None
        if cosmetics:
            self.cosmetics = self.d.add(self.d.g(id="cosmetics"))

        screw_hole_y = 3
        screw_hole_x = 1.5 * HP
        screw_hole_d = 3.4

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
                    font_size=title_size,
                    text_anchor="middle"
                ))

        # Draw logo
        logo_y = self.height - 9
        if hp < 8:
            logo_y = self.height - 12
        logo_width = min(self.width, 15)
            
        logo = self.d.path(
            LOGO_PATH,
            transform=f"translate({self.width / 2 - logo_width / 2}, {logo_y}) scale({logo_width / 100})",
        )
        self.stencil.add(logo)

        # Draw outline
        if outline is None:
            if cosmetics:
                outline = 0
            elif debug:
                outline = 2
            else:
                outline = 1
        elif outline not in [0, 1, 2]:
                raise ValueError("rotation must be 0, 1, 2, or None (default)")
            
        if outline == 0:
            pass
        elif outline == 2:
            self.outline.add(
                self.d.rect(size=(self.width, self.height), stroke_width=1))
        elif outline == 1:
            length = 3
            lines = [
                ((0, -length), (0, length)),
                ((-length, 0), (length, 0)),
                ((self.width + length, 0), (self.width - length, 0)),
                ((self.width, -length), (self.width, length)),
                ((0, self.height + length), (0, self.height - length)),
                ((-length, self.height), (length, self.height)),
                ((self.width, self.height + length), (self.width, self.height - length)),
                ((self.width + length, self.height), (self.width - length, self.height)),
                ]
            for start, end in lines:
                self.outline.add(self.d.line(start, end, stroke_width=1, stroke="black"))

        if self.debug:
            center = self.width / 2
            self.debug.add(self.d.line((center, 0), (center, self.height), stroke="green", stroke_dasharray="4,3", stroke_width=.5))

            for x in range(hp * 2):
                _x = x * inches(0.1) - self.tolerance / 2
                self.debug.add(self.d.line((_x, global_y_offset), (_x, global_y_offset + 100), stroke_width=0.1))
            for y in range(int(100 / inches(0.1)) + 1):
                _y = y * inches(0.1) + global_y_offset
                self.debug.add(self.d.line((0, _y), (inches(hp * .2), _y), stroke_width=0.1))
                        

        global_offset = (self.width / 2, global_y_offset)
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

        self.inkscape_actions.extend(component.inkscape_actions)
        self.post_process.extend(component.post_process)

    def draw(self, function):
        self.stencil.add(function(self.d))

    def save(self):
        self.d.save()
        cmd = "".join(self.inkscape_actions)

        if not self.debug:
            run_inkscape(
                "--with-gui",
                f"--actions={cmd};select-by-element:text;ObjectToPath;select-by-id:throughholes_offset;SelectionUnGroup;select-by-id:stencil_offset;SelectionUnGroup;FileSave;FileQuit",
                self.d.filename,
            )

            if self.post_process:
                tree = ET.parse(self.d.filename)
                for process in self.post_process:
                    process(tree)
                font_style = tree.findall(f".//*[@id='font-style']")[0]
                parent = font_style.find("..")
                parent.remove(font_style)
                tree.write(self.d.filename)


def run_inkscape(*args):
    cmd = ["inkscape"] + list(args)
    try:
        result = subprocess.run(cmd, capture_output=True)
    except Exception as e:
        print("Error running inkscape. Make sure inkscape is installed and in your $PATH")
        return

    if result.returncode != 0:
        print("Error running inkscape:")
    print_inkscape_stdout_ignoring_font_face_warnings(result.stdout)


def print_inkscape_stdout_ignoring_font_face_warnings(output):
    lines = output.decode("utf-8").split("\n")
    i = 0
    while i < len(lines):
        if lines[i].startswith("end_font_face_cb"):
            del lines[i]
            if lines[i].startswith("  font-family"):
                del lines[i]
            if lines[i].startswith("  font-style"):
                del lines[i]
            if lines[i].startswith("  font-weight"):
                del lines[i]
            if lines[i].startswith("  src"):
                del lines[i]
        else:
            i += 1
    result = "\n".join(lines)
    if result:
        print(result)


class Component:
    def __init__(self, position_x, position_y):
        self.position = (position_x, position_y)
        self.inkscape_actions = []
        self.post_process = []

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


# The datasheet says this should be an offset of 4.92mm for the "Thonkicon" but it
# the distance between the throughholes is 8.3mm (.33 inches) so I kept the ratio and scaled
# it down to .3in
class JackSocket(BasicCircle(0, 4.51691566, 3 + HOLE_ALLOWANCE)):
    def __init__(self, x, y, label, is_output, rotation=0, font_size=None, label_above=False):
       super(JackSocket, self).__init__(x, y, rotation)
       self.label = label
       self.is_output = is_output
       self.font_size = font_size
       self.label_above = label_above
       
    def draw_holes(self, context):
        return [context.circle(center=self.offset, r=self.radius)]

    def draw_stencil(self, context):
        hole_center = self.offset
        hole_radius = self.radius
        text_offset = 8
        if self.label_above:
            text_offset = -5.5
            
        text_props = {
            "insert": (hole_center[0], hole_center[1] + text_offset),
            "text_anchor": "middle",
        }

        if self.font_size:
            text_props["font_size"] = self.font_size
        
        if self.is_output:
            text_props["fill"] = "#ffffff"
            
        id = random_str()
        path_id = f"path_{id}"
        text_id = f"text_{id}"
        elements = []
        if self.is_output:
            padding = 1
            width = 2 * (hole_radius + padding)
            height = 15 if self.label else width
            outer_path = round_rect_as_path(
                hole_center[0] - width/2,
                hole_center[1] - hole_radius - padding,
                width,
                height,
                1.5
            )
            inner_r = hole_radius+.3
            inner_path = " ".join([
                f"M {hole_center[0]} {hole_center[1] - inner_r}",
                f"a {inner_r} {inner_r} 0 1 0 0 {inner_r * 2}",
                f"a {inner_r} {inner_r} 0 1 0 0 {-inner_r * 2}",
                f"z",
            ])

            elements.append(context.path(
                outer_path + inner_path,
                fill="#000000",
                id=path_id,
            ))

            cmd = f"select-by-id:{path_id},{text_id};SelectionDiff;EditDeselect;select-by-id:{path_id};EditDeselect;"

            self.inkscape_actions.append(cmd)

            def remove_style(tree):
                element = tree.findall(f".//*[@id='{path_id}']")[0]
                element.set("style", "")

            self.post_process.append(remove_style)

        elements.append(context.text(self.label, id=text_id, **text_props))

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
        
class JackSocketCentered(JackSocket):
    def __init__(self, x, y, label, is_output, rotation=0, font_size=None, label_above=False):
        super(JackSocketCentered, self).__init__(x, y, label, is_output, rotation, font_size, label_above)
        self.offset = (0, 0)


def random_str(n = 10):
    return "".join(random.choices(string.ascii_lowercase, k=n))


def round_rect_as_path(x, y, w, h, r):
    h_edge = w - (2 * r)
    v_edge = h - (2 * r)
    return " ".join([
        f"M {x} {y + r}",
        f"a {r} {r} 0 0 1 {r} {-r}",
        f"h {h_edge}",
        f"a {r} {r} 0 0 1 {r} {r}",
        f"v {v_edge}",
        f"a {r} {r} 0 0 1 {-r} {r}",
        f"h {-h_edge}",
        f"a {r} {r} 0 0 1 {-r} {-r}",
        f"z",
    ])


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

class SmallLED(BasicCircle(0, inches(.05), 1.5)):
    def __init__(self, x, y, rotation=0, font_size=None, color="red"):
       super(SmallLED, self).__init__(x, y, rotation)
       self.color = color


class LED(BasicCircle(0, inches(.05), 2.5)):
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
    elements = []
    washer_gradient = context.linearGradient(
        (1, 0),
        (0, 1),
    )
    washer_gradient.add_stop_color(0, "#eee")
    washer_gradient.add_stop_color(1, "#aaa")
    context.defs.add(washer_gradient)
    elements.append(context.circle(center=self.offset,
            r=self.radius * 1.2,
            fill=washer_gradient.get_paint_server()))
    gradient = context.linearGradient(
        (1, 0),
        (0, 1),
    )
    gradient.add_stop_color(0, "#aaa")
    gradient.add_stop_color(1, "#000")
    context.defs.add(gradient)
    elements.append(context.circle(center=self.offset,
            r=self.radius,
            fill=gradient.get_paint_server()))
    gradient2 = context.linearGradient(
        (1, 0),
        (0, 1),
    )
    gradient2.add_stop_color(0, "#111")
    gradient2.add_stop_color(1, "#888")
    context.defs.add(gradient2)
    elements.append(context.circle(center=self.offset,
            r=self.radius * .8,
            fill=gradient2.get_paint_server()))
            
    return elements
    

Button = BasicCircle(0, 0, 4)
setattr(Button, 'draw_cosmetics', draw_button_cosmetic)

TL1265 = BasicCircle(3.0, 4.5/2, 2.5)
setattr(TL1265, 'draw_cosmetics', draw_button_cosmetic)
    


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
