try:
    import svgwrite
    from svgwrite import mm
    from svgwrite.mixins import Transform
    from svgwrite.container import Group
except ImportError:
    print("This library requires svgwrite but it is not installed.")
    print("Install it with:")
    print("    python3 -m pip install --user svgwrite")
    import sys
    sys.exit(1)

try:
    import colour
    from colour import Color
except ImportError:
    print("This library requires the colour package but it is not installed.")
    print("Install it with:")
    print("    python3 -m pip install --user colour")
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

HOLE_ALLOWANCE = .15  # mm 

from os import path

FONT_CACHE_PATH = path.join(
    path.dirname(path.abspath(__file__)),
    "_faceplate_font_cache.txt")

if path.exists(FONT_CACHE_PATH):
    with open(FONT_CACHE_PATH) as f:
        font_string = f.read()
else:
    try:
        import urllib.request
        import base64

        with urllib.request.urlopen("https://fonts.gstatic.com/s/ubuntu/v14/4iCv6KVjbNBYlgoCjC3jsGyN.woff2") as response:
            font_b64 = base64.b64encode(response.read())
            font_string = "url(\"data:application/font-woff;charset=utf-8;base64,{}\")".format(font_b64.decode('utf-8'))

        with open(FONT_CACHE_PATH, "w") as f:
            font_string = f.write(font_string)

    except Exception as e:
        print("Warning: Unable to download font")
        font_string = "local('Ubuntu Medium'), local('Ubuntu-Medium')"

import math
from math import sqrt
import subprocess
import random
import string
import io
from typing import List, Callable, Tuple
from enum import Enum

def inches(n):
    """Returns n inches in mm"""
    return n * 25.4

class Module:
    def __init__(self, hp, global_y_offset=0, title=None, filename="output.svg", debug=False, cosmetics=False, drill_markers=True, outline=None, title_size=5, hide_logo=False, title_offset=0):
        assert isinstance(global_y_offset, (int, float))
        assert isinstance(hp, int)
        HP = inches(0.2)
        self.height = 128.5
        self.width = math.floor((hp * HP) * 10 - 3) / 10

        if outline == 3:
            svg_size = ((self.width + 1) * mm, (self.height + 1) * mm)
        else:
            svg_size = (self.width * mm, self.height * mm)

        self.d = svgwrite.Drawing(filename=filename.replace(" ", "_"), size=svg_size)

        self.d.add(
            self.d.rect(size=(self.width, self.height), fill="white", id="background"))
        
        self.d.defs.add(self.d.style(id="font-style", content="@font-face {{ font-family: 'Ubuntu'; font-style: normal; font-weight: 500; src: {}; }}".format(font_string)))
        if outline == 3:
            self.d.viewbox(-.5, -.5, self.width+1, self.height+1)
        else:
            self.d.viewbox(0, 0, self.width, self.height)
        self.outline = self.d.add(self.d.g(id="outline", fill="none", stroke="black"))
        self.stencil = self.d.add(self.d.g(id="stencil", font_family="Ubuntu", font_size=3))
        self.holes = self.d.add(self.d.g(id="throughholes", fill="black", stroke="none"))
        self.inkscape_actions = []
        self.post_process = []

        self.drill_markers = None
        if drill_markers:
            self.drill_markers = self.d.add(self.d.g(id="drill_markers", stroke="white"))
        
        self.debug = None
        if debug:
            self.debug = self.d.add(self.d.g(id="debug", fill="red", stroke="red"))
            
        self.cosmetics = None
        if cosmetics:
            self.cosmetics = self.d.add(self.d.g(id="cosmetics"))

        screw_hole_y = 3
        hole_spacing_x = inches(.2) * (hp - 3)
        center = self.width / 2
        screw_hole_x1 = center - hole_spacing_x / 2
        screw_hole_d = 3.4

        def screw_hole(x, y):
            self.holes.add(
                self.d.circle(center=(x, y), r=screw_hole_d/2,
                              stroke="none"))
            if self.drill_markers:
                self.drill_markers.add(
                    draw_drill_marker(self.d, x, y))
            
        # draw left mounting holes
        screw_hole(screw_hole_x1, screw_hole_y)
        screw_hole(screw_hole_x1, self.height - screw_hole_y)

        # draw right mounting holes
        if hp > 6:
            screw_hole_x2 = screw_hole_x1 + hole_spacing_x
            screw_hole(screw_hole_x2, screw_hole_y)
            screw_hole(screw_hole_x2, self.height - screw_hole_y)

        # Draw title
        if title:
            title_offset_y = 6.3
            if hp < 8:
                title_offset_y = 10
            if title_offset:
                title_offset_y = title_offset
            self.stencil.add(
                self.d.text(title,
                    insert=(self.width / 2, title_offset_y),
                    font_size=title_size,
                    text_anchor="middle"
                ))

        # Draw logo
        if not hide_logo:
            logo_width = min(self.width, 15)
            # logo_height = .465 * logo_width
            logo_y = self.height - 9
            if hp < 8:
                logo_y = self.height - 12
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
        elif outline not in [0, 1, 2, 3]:
                raise ValueError("outline must be 0, 1, 2, 3, or None (default)")
            
        if outline == 0:
            pass
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
        elif outline == 2:
            self.outline.add(
                self.d.rect(size=(self.width, self.height), stroke_width=1))
        elif outline == 3:
            self.outline.add(
                self.d.rect((-.25, -.25), size=(self.width+.5, self.height+.5), stroke_width=.5))

        if self.debug:
            center = self.width / 2
            self.debug.add(self.d.line((center, 0), (center, self.height), stroke="green", stroke_dasharray="4,3", stroke_width=.5))

            for x in range(-hp, hp):
                _x = center + x * inches(0.1)
                self.debug.add(self.d.line((_x, global_y_offset), (_x, global_y_offset + 100), stroke_width=0.2 if x % 5 == 0 else 0.1))
            for y in range(int(100 / inches(0.1)) + 1):
                _y = y * inches(0.1) + global_y_offset
                self.debug.add(self.d.line((0, _y), (inches(hp * .2), _y), stroke_width=0.2 if y % 5 == 0 else 0.1))

            # self.debug.add(self.d.line((0, global_y_offset + 100), (inches(hp * .2), global_y_offset + 100), stroke_width=0.2))

            self.debug.add(self.d.rect((0, 0), (inches(hp * .2), inches(.4)), fill_opacity=0.5, fill="cyan", stroke_width=0))
            self.debug.add(self.d.rect((0, self.height - inches(.4)), (inches(hp * .2), inches(.4)), fill_opacity=0.5, fill="cyan", stroke_width=0))

                        

        global_offset = (self.width / 2, global_y_offset)
        self.holes = self.holes.add(self.d.g(id="throughholes_offset"))
        self.holes.translate(global_offset)
        if self.drill_markers:
            self.drill_markers = self.drill_markers.add(self.d.g(id="drill_markers_offset"))
            self.drill_markers.translate(global_offset)
        self.stencil = self.stencil.add(self.d.g(id="stencil_offset"))
        self.stencil.translate(global_offset)
        if self.debug:
            self.debug = self.debug.add(self.d.g(id="debug_offset"))
            self.debug.translate(global_offset)
        if self.cosmetics:
            self.cosmetics = self.cosmetics.add(self.d.g(id="cosmetics_offset"))
            self.cosmetics.translate(global_offset)


    def add(self, component):
        if not self.cosmetics or component.cosmetic_holes:
            group = self.holes.add(self.d.g())
            group.translate(*component.position)
            for x in component.draw_holes(self.d):
                group.add(x)

        if self.drill_markers:
            group = self.drill_markers.add(self.d.g())
            group.translate(*component.position)
            for x in component.draw_drill_markers(self.d):
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

            tree = ET.parse(self.d.filename)
            for process in self.post_process:
                process(tree)
            font_style = tree.findall(f".//*[@id='font-style']")[0]
            parent = font_style.find("..")
            parent.remove(font_style)

            try:
                from scour import scour
                string = ET.tostring(tree)
                with open(self.d.filename, "wb") as output_file:
                    input_file = io.BytesIO(string)
                    class ScourOptions:
                        def __init__(self):
                            self.digits = 5
                            self.cdigits = -1
                            self.simple_colors = True
                            self.style_to_xml = True
                            self.group_collapse = False
                            self.group_create = False
                            self.keep_editor_data = False
                            self.keep_defs = False
                            self.renderer_workaround = True
                            self.indent_type = "space"
                            self.indent_depth = 2
                            self.newlines = True
                            self.shorten_ids_prefix = ""
                            self.protect_ids_noninkscape = False
                            self.quiet = True
                    scour.start(ScourOptions(), input_file, output_file)
            except ImportError:
                print("Warning: This tool uses scour but it is not installed.")
                print("Install it with:")
                print("    python3 -m pip install --user scour")
                print("Skipping scour step")
                tree.write(self.d.filename)


    @classmethod
    def from_cli(self, hp, global_y_offset=0, title=None, title_size=5, hide_logo=False, **kwargs):
        import argparse
        parser = argparse.ArgumentParser(description="Script to make faceplate SVGs for FM modules")
        parser.add_argument("--mode", choices=["stencil", "display", "debug"], default="stencil", help="which features should be included in the image")
        parser.add_argument("-o", "--output", metavar="FILE", help="path to svg file to output")
        parser.add_argument("--outline", action="store_true", help="Add outline")
        args = parser.parse_args()
        return Module(
            hp,
            global_y_offset=global_y_offset,
            title=title,
            title_size=title_size,
            hide_logo=hide_logo,
            filename=args.output or f"{title}.svg",
            debug=args.mode == "debug",
            cosmetics=args.mode == "display",
            drill_markers=args.mode == "stencil",
            outline=3 if args.outline else None,
            **kwargs
        )


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
    def __init__(self, position_x: float, position_y: float):
        self.position = (position_x, position_y)
        self.inkscape_actions: List[str] = []
        self.post_process: List[Callable] = []
        self.cosmetic_holes = True

    def draw_holes(self, context: Group):
        return []

    def draw_drill_markers(self, context: Group):
        return []

    def draw_stencil(self, context: Group):
        return []
        
    def draw_cosmetics(self, context: Group):
        return []


def BasicCircle(offset_x: float, offset_y: float, r: float):
    class BasicCircle(Component):
        def __init__(self, position_x: float, position_y: float, rotation=0):
            super().__init__(position_x, position_y)
            self.rotation = rotation
            if rotation not in [0, 1, 2, 3]:
                raise ValueError("rotation must be 0...3")

            self.offset = self.rotated((offset_x, offset_y))
                
            self.radius = r
            
        def draw_holes(self, context):
            return [context.circle(center=self.offset, r=self.radius)]
            
        def draw_drill_markers(self, context):
            return [draw_drill_marker(context, *self.offset)]
            
        def draw_debug(self, context):
            return [draw_x(context, 0, 0)]

        def rotated(self, point: Tuple[float, float]) -> Tuple[float, float]:
            x, y = point
            if self.rotation == 0:
                return x, y
            elif self.rotation == 1:
                return -y, x
            elif self.rotation == 2:
                return -x, -y
            elif self.rotation == 3:
                return y, -x
            raise ValueError("rotation must be 0...3")
            
    return BasicCircle

def draw_drill_marker(context, x, y, size=1, stroke_width=.2):
    return context.path([
        f"M {x} {y-size}",
        f"L {x} {y+size}",
        f"M {x-size} {y}",
        f"L {x+size} {y}",
    ], stroke_width=stroke_width)

def draw_x(context, x, y, size=1, stroke_width=.1):
    return context.path([
        f"M {x-size} {y-size}",
        f"L {x+size} {y+size}",
        f"M {x-size} {y+size}",
        f"L {x+size} {y-size}",
    ], stroke_width=stroke_width)

# The datasheet says this should be an offset of 4.92mm for the "Thonkicon" but it
# the distance between the throughholes is 8.3mm (.33 inches) so I kept the ratio and scaled
# it down to .3in
class JackSocket(BasicCircle(0, 4.51691566, 3 + HOLE_ALLOWANCE)):
    def __init__(self, x, y, label, is_output, rotation=0, font_size=None, label_above=False, text_offset=None):
        super().__init__(x, y, rotation)
        self.label = label
        self.is_output = is_output
        self.font_size = font_size
        self.label_above = label_above
        self.text_offset = text_offset
        self.hex = False
       
    def draw_holes(self, context):
        return [context.circle(center=self.offset, r=self.radius)]

    def draw_stencil(self, context):
        hole_center = self.offset
        hole_radius = self.radius
        text_offset = self.text_offset
        if not text_offset:
            text_offset = (0, self.radius + 4.85)
            if self.label_above:
                text_offset = (0, -(self.radius + 2.35))
            
        text_props = {
            "insert": (hole_center[0] + text_offset[0], hole_center[1] + text_offset[1]),
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
            padding = 1.5
            width = 2 * (hole_radius + padding)
            height = 15 if self.label else width
            outer_path = round_rect_as_path(
                hole_center[0] - width/2,
                hole_center[1] - hole_radius - padding,
                width,
                height,
                1.5
            )
            inner_r = hole_radius+.35
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
        radius = self.radius
        elements = []
        gradient = context.linearGradient(
            (1, 0),
            (0, 1),
        )
        gradient.add_stop_color(-1, "white")
        gradient.add_stop_color(2, "black")
        context.defs.add(gradient)
        if self.hex:
            elements.append(draw_regular_polygon(
                context,
                self.offset,
                6,
                radius + 1.3,
                math.pi / 6,
                fill=gradient.get_paint_server()
            ))
        else:
            elements.append(draw_bumpy_circle(
                context,
                self.offset,
                radius + .4,
                radius + .6,
                18,
                fill=gradient.get_paint_server()
            ))

        ring_thickness = .8
        gradient = context.radialGradient((.5, .5), .5)
        gradient.add_stop_color(1 - ring_thickness / radius, "black")
        gradient.add_stop_color(1 - ring_thickness / radius / 2, "white")
        gradient.add_stop_color(1, "#444")
        context.defs.add(gradient)
        elements.append(context.circle(
            self.offset,
            radius,
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
            radius - ring_thickness,
            fill=gradient.get_paint_server()
        ))
        return elements
        
class JackSocketCentered(JackSocket):
    def __init__(self, x, y, label, is_output, rotation=0, font_size=None, label_above=False, text_offset=None):
        super().__init__(x, y, label, is_output, rotation, font_size, label_above, text_offset)
        self.offset = (0, 0)

    def draw_debug(self, context):
        return [
            *super().draw_debug(context),
            context.circle(center=self.rotated((0, -4.92)), r=0.25),
            context.circle(center=self.rotated((0, 3.38)), r=0.25),
            context.circle(center=self.rotated((0, 6.48)), r=0.25),
        ]

class JackSocketQuarterInch(JackSocket):
    def __init__(self, x, y, label, is_output, rotation=0, font_size=None, label_above=False, text_offset=None):
        super().__init__(x, y, label, is_output, rotation, font_size, label_above, text_offset)
        self.offset = (0, 0)
        self.radius = 9.5 / 2
        self.hex = True

    def draw_debug(self, context):
        return [
            *super().draw_debug(context),
            context.circle(center=self.rotated((4.67, 4.67)), r=0.25),
            context.circle(center=self.rotated((-2.4, 6.38)), r=0.25),
            context.circle(center=self.rotated((-6.38, 0.56)), r=0.25),
            context.circle(center=self.rotated((0.56, -6.38)), r=0.25),
            context.circle(center=self.rotated((6.38, -2.4)), r=0.25),
            context.rect((-8, -8), (16, 16), stroke="cyan", fill="none", stroke_width=.2),
        ]


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


def draw_bumpy_circle_2(context, center, inner_r, outer_r, n, large_frac, small_frac, start_theta=0, **kwargs):
    assert outer_r > inner_r
    rads_per_lobe = 2 * math.pi / n
    lobe_width_rad = large_frac * rads_per_lobe
    cut_width_rad = small_frac * rads_per_lobe
    slope_width_rad = (rads_per_lobe - lobe_width_rad - cut_width_rad) / 2
    assert slope_width_rad > 0
    assert lobe_width_rad + cut_width_rad + 2 * slope_width_rad == rads_per_lobe

    cx, cy = center
    def from_polar(theta, r):
        return cx + math.cos(theta) * r, cy + math.sin(theta) * r

    start_x, start_y = from_polar(start_theta, outer_r)
    path = context.path(**kwargs)
    path.push(f"M {start_x} {start_y}")

    for i in range(n):
        theta = start_theta + (i * rads_per_lobe)

        # Lobe
        theta += lobe_width_rad
        x, y = from_polar(theta, outer_r)
        path.push(f"A {outer_r} {outer_r} 0 0 1 {x} {y}")

        # Bevel 1
        theta += slope_width_rad
        x, y = from_polar(theta, inner_r)
        path.push(f"L {x} {y}")

        # Cut
        theta += cut_width_rad
        x, y = from_polar(theta, inner_r)
        path.push(f"L {x} {y}")

        # Bevel 1
        if i != n - 1:
            theta += slope_width_rad
            x, y = from_polar(theta, outer_r)
            path.push(f"L {x} {y}")

    path.push("Z")
    return path


def lighten(color, amount):
    color = Color(color)
    color.luminance += amount
    return color.hex

def darken(color, amount):
    color = Color(color)
    color.luminance -= amount
    return color.hex


class Switch(BasicCircle(0, 0, inches(1/8) + HOLE_ALLOWANCE)):
    def __init__(self, x, y, label=None, left_text=None, right_text=None, font_size=None, rotation=0):
        super().__init__(x, y, 0)
        self.label = label
        self.font_size = font_size
        self.left_text = left_text
        self.right_text = right_text
        self.hole_radius = self.radius
        self.hole_center = self.offset
        self.rotation = rotation
        if rotation not in [0, 1, 2, 3]:
            raise ValueError("rotation must be 0...3")

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

        angle = self.rotation * math.pi / 2
        length = 4
        width = 2.2
        spread = .1
        rounding = 2.5
        wide_width = width + 2 * length * math.sin(spread)

        if not self.rotation:
            grad_offset = (.8, .35)
        elif self.rotation == 1:
            grad_offset = (.65, .6)
        elif self.rotation == 2:
            grad_offset = (.2, .35)
        elif self.rotation == 3:
            grad_offset = (.65, .2)

        gradient = context.radialGradient(grad_offset, .6)
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


class SmallSwitch(Switch):
    def __init__(self, x, y, label=None, left_text=None, right_text=None, font_size=None, rotation=0):
        super().__init__(x, y, label=label, left_text=left_text, right_text=right_text, font_size=font_size, rotation=rotation)
        self.radius = 2.25 + HOLE_ALLOWANCE
        self.hole_radius = self.radius
        self.cosmetic_holes = False


class SmallLED(BasicCircle(0, inches(.05), 1.55)):
    def __init__(self, x, y, rotation=0, font_size=None, color="red"):
        super().__init__(x, y, rotation)
        self.color = color
        self.cosmetic_holes = False

    def draw_debug(self, context):
        return [
            *super().draw_debug(context),
            context.circle(center=self.rotated((0, 0)), r=0.25),
            context.circle(center=self.rotated((0, inches(.1))), r=0.25),
        ]


class LED(BasicCircle(0, inches(.05), 2.55)):
    def __init__(self, x, y, rotation=0, font_size=None, color="red"):
        super().__init__(x, y, rotation)
        self.color = color
        self.cosmetic_holes = False

    def draw_debug(self, context):
        return [
            *super().draw_debug(context),
            context.circle(center=self.rotated((0, 0)), r=0.25),
            context.circle(center=self.rotated((0, inches(.1))), r=0.25),
        ]


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


def draw_button_cosmetic(self, context, with_washer=True, colors=[("#aaa", "#000"), ("#888", "#111")]):
    elements = []
    if with_washer:
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
    gradient.add_stop_color(0, colors[0][0])
    gradient.add_stop_color(1, colors[0][1])
    context.defs.add(gradient)
    elements.append(context.circle(center=self.offset,
            r=self.radius,
            fill=gradient.get_paint_server()))
    gradient2 = context.linearGradient(
        (1, 0),
        (0, 1),
    )
    gradient2.add_stop_color(0, colors[1][1])
    gradient2.add_stop_color(1, colors[1][0])
    context.defs.add(gradient2)
    elements.append(context.circle(center=self.offset,
            r=self.radius * .8,
            fill=gradient2.get_paint_server()))
            
    return elements
    

Button = BasicCircle(0, 0, 4)
setattr(Button, 'draw_cosmetics', draw_button_cosmetic)


TL1265 = BasicCircle(3.0, 4.5/2, 2.55)
setattr(TL1265, 'draw_cosmetics', draw_button_cosmetic)


class TL1105SP(BasicCircle(0, 0, 5.1/2)):
    def __init__(self, x, y, rotation=0):
        super().__init__(x, y, rotation)
        self.cosmetic_holes = False

    def draw_cosmetics(self, context):
        return draw_button_cosmetic(self, context, with_washer=False)

    def draw_debug(self, context):
        w = 6.5
        h = 4.5
        return [
            context.circle(center=self.rotated((-w/2, -h/2)), r=0.25),
            context.circle(center=self.rotated(( w/2, -h/2)), r=0.25),
            context.circle(center=self.rotated((-w/2,  h/2)), r=0.25),
            context.circle(center=self.rotated(( w/2,  h/2)), r=0.25),
        ]

class D6R30(BasicCircle(0, 0, 9/2 + HOLE_ALLOWANCE)):
    def __init__(self, x, y, rotation):
        super().__init__(x, y, rotation)
        self.cosmetic_holes = False

    def draw_cosmetics(self, context):
        return draw_button_cosmetic(self, context, with_washer=False, colors=[("#ff0", "#550"), ("#ff9", "#dd0")])

    def draw_debug(self, context):
        spread = 2.5
        return [
            context.circle(center=self.rotated((-spread, -spread)), r=0.25),
            context.circle(center=self.rotated(( spread, -spread)), r=0.25),
            context.circle(center=self.rotated((-spread,  spread)), r=0.25),
            context.circle(center=self.rotated(( spread,  spread)), r=0.25),
        ]


class PotStyle(Enum):
    OLD = 1
    ROGAN_PT_1S = 2
    CHROMATIC = 3
    CHROMATIC_SMALL = 10
    SIFAM_MEDIUM = 20
    SIFAM_MEDIUM_RE = 21
    SIFAM_LARGE = 30

class PotColor(Enum):
    WHITE = 1,
    RED = 2
    ORANGE = 3
    YELLOW = 4
    GREEN = 5
    BLUE = 6
    MAGENTA = 7


class Potentiometer(BasicCircle(inches(.1), inches(-.3), 3.5 + HOLE_ALLOWANCE)):
    def __init__(self, x, y, label=None, rotation=0, font_size=None, color=PotColor.WHITE, text_offset=None, style=PotStyle.SIFAM_MEDIUM):
        super().__init__(x, y, rotation)
        if text_offset is None:
            if "SMALL" in style.name:
                text_offset = 10
            elif style == PotStyle.SIFAM_LARGE:
                text_offset = 12.75
            else:
                text_offset = 11.5
        self.label = label
        self.font_size = font_size
        self.color = color
        self.text_offset = text_offset
        self.style = style

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

    def draw_debug(self, context):
        return [
            *super().draw_debug(context),
            context.circle(center=self.rotated((inches( 0), 0)), r=0.25),
            context.circle(center=self.rotated((inches(.1), 0)), r=0.25),
            context.circle(center=self.rotated((inches(.2), 0)), r=0.25),
        ]
    
    def draw_cosmetics(self, context):
        if self.style == PotStyle.OLD:
            return self.draw_old_cap(context)

        if self.style == PotStyle.SIFAM_MEDIUM or self.style == PotStyle.SIFAM_MEDIUM_RE:
            skirt_radius = 14.3 / 2
            outer_r = 11 / 2
            inner_r = 10.5 / 2
            cap_r = 4
            radii=[skirt_radius, outer_r, inner_r, cap_r]
            return self.draw_sifam_cap(context, radii, self.style == PotStyle.SIFAM_MEDIUM)
        elif self.style == PotStyle.SIFAM_LARGE:
            skirt_radius = 18.5 / 2
            outer_r = 15.3 / 2
            inner_r = outer_r - .35
            cap_r = 11.5/2
            radii=[skirt_radius, outer_r, inner_r, cap_r]
            return self.draw_sifam_cap(context, radii)
        
        if self.style == PotStyle.ROGAN_PT_1S:
            skirt_radius = 14.38 / 2
            outer_r = 11 / 2
            inner_r = 10 / 2
            cap_r = 4
            radii=[skirt_radius, outer_r, inner_r, cap_r]
            return self.draw_chromatic_cap(context, radii, cap_color=["#fff", "#bbb"])

        if "CHROMATIC" in self.style.name and "SMALL" not in self.style.name:
            if self.style == PotStyle.CHROMATIC_WHITE:
                cap_color = ["#fff", "#bbb"]
                pointer_color = "#eee"
            elif self.style == PotStyle.CHROMATIC_RED:
                cap_color = ["#e25f62", "#d23e3e"]
                pointer_color = "#f44b4c"
            skirt_radius = 16 / 2
            outer_r = 11 / 2
            inner_r = 10 / 2
            cap_r = 4
            radii=[skirt_radius, outer_r, inner_r, cap_r]
            return self.draw_chromatic_cap(context, radii, cap_color, pointer_color)

        if "CHROMATIC" in self.style.name and "SMALL" in self.style.name:
            skirt_radius = 11.5/2
            outer_r = 11 / 2
            inner_r = 10 / 2
            cap_r = 4
            radii=[skirt_radius, outer_r, inner_r, cap_r]
            return self.draw_chromatic_cap(context, radii, cap_color=["#fff", "#bbb"], pointer_color="#eee")

    def draw_sifam_cap(self, context, radii, pointer=True):
        cap_colors = {
            PotColor.WHITE: ["#fff", "#ccc"],
            PotColor.RED: ["#e25f62", "#d23e3e"],
            PotColor.GREEN: [lighten("#54ad77", .18), "#379a64"],
            PotColor.ORANGE: ["#fe8f78", "#f15d38"],
            PotColor.YELLOW: ["#fae98a", "#f7e150"],
            PotColor.BLUE: [lighten("#0bbff2", .18), darken("#0bbff2", .1)],
            PotColor.MAGENTA: ["#ff85be", "#e8538c"],
        }
        cap_color = cap_colors[self.color]
        if self.color in [PotColor.WHITE, PotColor.BLUE, PotColor.YELLOW]:
            pointer_color = "#000"
        else:
            pointer_color = "#fff"

        elements=[]
        skirt_radius = radii[0]
        outer_r, inner_r = radii[1:3]
        if skirt_radius:
            skirt_gradient = context.linearGradient(
                (sqrt(1/2), 0),
                (0, sqrt(1/2)),
            )
            skirt_gradient.add_stop_color(0, "#666")
            skirt_gradient.add_stop_color(1, "#191919")
            context.defs.add(skirt_gradient)
            elements.append(context.circle(
                center=self.offset,
                r=skirt_radius,
                fill=skirt_gradient.get_paint_server()
            ))
            skirt_gradient2 = context.radialGradient(
                center=(.5, .5),
                r=.5,
            )
            scale = outer_r / skirt_radius
            skirt_gradient2.add_stop_color(scale * .85, "#000", 1)
            skirt_gradient2.add_stop_color(scale * 1.15, "#000", 0)
            context.defs.add(skirt_gradient2)
            elements.append(context.circle(
                center=self.offset,
                r=skirt_radius,
                fill=skirt_gradient2.get_paint_server()
            ))

        cx, cy = self.offset
        def from_polar(theta, r):
            return cx + math.cos(theta) * r, cy + math.sin(theta) * r

        num_lobes = 6
        lobe_width_ratio = .7
        cut_width_ratio = .12

        knob_theta = -math.pi * 3/4
        start_theta = knob_theta + ((1 - lobe_width_ratio) * (2 * math.pi / num_lobes)) / 2

        grip_gradient = context.linearGradient(
            (sqrt(1/2), 0),
            (0, sqrt(1/2)),
        )
        grip_gradient.add_stop_color(0, "#666")
        grip_gradient.add_stop_color(.04, "#686868")
        grip_gradient.add_stop_color(1, "#212121")
        context.defs.add(grip_gradient)
        elements.append(draw_bumpy_circle_2(
            context, self.offset, inner_r, outer_r, num_lobes,
            lobe_width_ratio, cut_width_ratio, start_theta,
            fill=grip_gradient.get_paint_server()
        ))

        cap_r = radii[3]

        grip_gradient2 = context.linearGradient(
            (sqrt(1/2), 0),
            (0, sqrt(1/2)),
        )
        grip_gradient2.add_stop_color(0, "#777")
        grip_gradient2.add_stop_color(1, "#555555")
        context.defs.add(grip_gradient2)
        elements.append(draw_bumpy_circle_2(
            context, self.offset,
            cap_r + .5, cap_r + .5 + outer_r - inner_r,
            num_lobes,
            lobe_width_ratio, cut_width_ratio, start_theta,
            fill=grip_gradient2.get_paint_server()
        ))

        cap_gradient = context.linearGradient(
            (sqrt(1/2), 0),
            (0, sqrt(1/2)),
        )
        cap_gradient.add_stop_color(0, cap_color[0])
        cap_gradient.add_stop_color(1, cap_color[1])
        context.defs.add(cap_gradient)
        elements.append(context.circle(
            center=self.offset,
            r=cap_r,
            fill=cap_gradient.get_paint_server()
        ))

        if pointer:
            elements.append(context.line(
                self.offset,
                from_polar(knob_theta, cap_r),
                stroke_width=.8,
                stroke=pointer_color))

        return elements
        
    def draw_chromatic_cap(self, context, radii, cap_color=None, pointer_color=None):
        elements = []
        skirt_radius = radii[0]
        if skirt_radius:
            skirt_gradient = context.linearGradient(
                (1, 0),
                (0, 1),
            )
            skirt_gradient.add_stop_color(0, "#555")
            skirt_gradient.add_stop_color(1, "#111")
            context.defs.add(skirt_gradient)
            elements.append(context.circle(
                center=self.offset,
                r=skirt_radius,
                fill=skirt_gradient.get_paint_server()
            ))

        num_lobes = 6
        outer_r, inner_r = radii[1:3]

        num_steps = num_lobes * 2 if pointer_color is None else num_lobes * 2 - 1
        lobe_width_ratio = .7
        lobe_width_rad = 2 * math.pi / num_lobes * lobe_width_ratio
        cut_width_rad = 2 * math.pi / num_lobes * (1 - lobe_width_ratio)

        cx, cy = self.offset

        def from_polar(theta, r):
            return cx + math.cos(theta) * r, cy + math.sin(theta) * r

        start_theta = -math.pi * 3/4
        theta = start_theta + cut_width_rad / 2
        start_x, start_y = from_polar(theta, outer_r)
        path_d = f"M {start_x} {start_y}"
        for step in range(num_steps):
            if step % 2 == 0:
                dtheta = lobe_width_rad
                r = outer_r
                next_r = inner_r
            else:
                dtheta = cut_width_rad
                r = inner_r
                next_r = outer_r

            theta += dtheta
            x1, y1 = from_polar(theta, r)
            x2, y2 = from_polar(theta, next_r)
            if step % 2 == 0:
                path_d += f" A {r} {r} 0 0 1 {x1} {y1}"
            else:
                path_d += f" L {x1} {y1}"
            if step != num_steps - 1:
                path_d += f" L {x2} {y2}"
        path_d += " Z"

        knob_gradient = context.linearGradient(
            (1, 0),
            (0, 1),
        )
        knob_gradient.add_stop_color(0, "#777")
        knob_gradient.add_stop_color(1, "#222")
        context.defs.add(knob_gradient)
        elements.append(context.path(path_d, fill=knob_gradient.get_paint_server()))

        if cap_color is not None:
            cap_gradient = context.linearGradient(
                (1, 0),
                (0, 1),
            )
            cap_gradient.add_stop_color(0, cap_color[0])
            cap_gradient.add_stop_color(1, cap_color[1])
            context.defs.add(cap_gradient)
            elements.append(context.circle(
                center=self.offset,
                r=radii[3],
                fill=cap_gradient.get_paint_server()
            ))

        if pointer_color is not None:
            offset = 1.6
            start = from_polar(start_theta, offset)
            end = from_polar(start_theta, offset + 4.5)
            pointer = context.line(start, end, stroke_width=1, stroke=pointer_color)
            shiftx, shifty = -.3, .6
            start = (start[0] + shiftx, start[1] + shifty)
            end = (end[0] + shiftx, end[1] + shifty)
            blur_filter = context.defs.add(context.filter())
            blur_filter.feGaussianBlur(in_='SourceGraphic', stdDeviation=.3)
            shadow = context.line(start, end, stroke_width=1, stroke="#000", opacity=.2, filter=blur_filter.get_funciri())
            elements.append(shadow)
            elements.append(pointer)
            
        return elements

    def draw_old_cap(self, context):
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


def draw_regular_polygon(context: Group, center:Tuple[float, float], n: int, r: float, rotation: float=0, **kwargs):
    cx, cy = center
    points = []
    for i in range(n):
        theta = 2 * math.pi * i / n + rotation
        x = math.cos(theta) * r + cx
        y = math.sin(theta) * r + cy
        points.append((x, y))

    return context.path([
        f"M {points[0][0]} {points[0][1]}",
        *[
            f"L {p[0]} {p[1]}"
            for p in points[1:]       
        ],
        "z"
    ], **kwargs)

def draw_m2_bolt_head(context: Group, point: Tuple[float, float]):
    r = 3.5/2
    edge_gradient = context.linearGradient(
        (1, 0),
        (0, 1),
    )
    edge_gradient.add_stop_color(-1, "white")
    edge_gradient.add_stop_color(2, "black")
    context.defs.add(edge_gradient)
    elements = []
    elements.append(context.circle(
        point,
        r,
        fill=edge_gradient.get_paint_server()
    ))
    top_gradient = context.radialGradient(
        center=(.5, .5),
        r=.5,
    )
    top_gradient.add_stop_color(0, "#666", opacity=1)
    top_gradient.add_stop_color(.75, "#666", opacity=1)
    top_gradient.add_stop_color(.85, "#666", opacity=0)
    top_gradient.add_stop_color(1, "#666", opacity=0)
    context.defs.add(top_gradient)
    elements.append(context.circle(
        point,
        r,
        fill=top_gradient.get_paint_server()
    ))
    hex_gradient = context.linearGradient(
        (1, 0),
        (0, 1),
    )
    hex_gradient.add_stop_color(0, "#111")
    hex_gradient.add_stop_color(1, "#444")
    context.defs.add(hex_gradient)
    elements.append(draw_regular_polygon(context, point, 6, 1, fill=hex_gradient.get_paint_server()))

    return elements
    

class OLED(Component):
    def __init__(self, x, y, rotation=0):
        super().__init__(x, y)
        # screen is 0.96in diagonal, 128x64, ~2mm bezel
        # leave 1mm overlap between faceplate and screen
        true_height = inches(0.96)/math.sqrt(5)
        true_width = true_height * 2
        self.rotation = rotation
        self.screen_height = true_height + 2
        self.screen_width = true_width + 2
        self.center_x = inches(.15)
        self.screen_bottom_offset = -3.4
        self.hole_spacing_x = 23.5
        self.hole_spacing_y = 24
        self.hole_offset_y = -inches(0.01)

    def rotated(self, point):
        x, y = point
        if self.rotation == 0:
            return x, y
        elif self.rotation == 1:
            return -y, x
        elif self.rotation == 2:
            return -x, -y
        elif self.rotation == 3:
            return y, -x

    @property
    def screen_offset(self):
        x, y = (
            -self.screen_width / 2 + self.center_x,
            self.screen_bottom_offset - self.screen_height
        )
        if self.rotation == 0:
            return x, y
        elif self.rotation == 1:
            return -y - self.screen_height, x
        elif self.rotation == 2:
            return -x - self.screen_width, -y - self.screen_height
        elif self.rotation == 3:
            return y, -x - self.screen_width

    @property
    def screen_size(self):
        if self.rotation == 0 or self.rotation == 2:
            return self.screen_width, self.screen_height
        else:
            return self.screen_height, self.screen_width

    def _get_hole_locations(self):
        hole_center_y = self.hole_offset_y - self.hole_spacing_y / 2
        holes = []
        for x in (-1, 1):
            for y in (-1, 1):
                holes.append(self.rotated((
                    self.center_x + x * (self.hole_spacing_x / 2),
                    hole_center_y + y * (self.hole_spacing_y / 2),
                )))
        return holes

    def draw_holes(self, context):
        screw_hole_d = inches(3/32)

        elements = []
        for p in self._get_hole_locations():
            elements.append(context.circle(center=p, r=screw_hole_d/2))

        elements.append(context.rect(insert=self.screen_offset, size=self.screen_size))

        return elements
        
    def draw_debug(self, context):
        return [
            draw_x(context, 0, 0),
            *[context.circle(center=self.rotated((inches(i * .1), 0)), r=.25) for i in range(4)],
        ]
        
    def draw_drill_markers(self, context):
        screw_hole_d = inches(3/32)

        elements = []
        for p in self._get_hole_locations():
            elements.append(draw_drill_marker(context, *p))

        return elements

    def draw_cosmetics(self, context):
        clip_path = context.defs.add(context.clipPath())
        clip_path.add(context.rect(insert=self.screen_offset, size=self.screen_size))
        
        elements = []
        
        elements.append(context.rect(insert=self.screen_offset, size=self.screen_size, fill="black"))
        x, y = self.screen_offset
        w, h = self.screen_size
        elements.append(context.ellipse(
            (x + w, y),
            r=(w / 2, h / 2),
            fill="white", opacity=.5,
            clip_path=f"url(#{clip_path.get_id()})"))

        for p in self._get_hole_locations():
            elements.extend(draw_m2_bolt_head(context, p))

        return elements


class OLEDSPI(OLED):
    def __init__(self, x, y, rotation=1):
        super().__init__(x, y, rotation)
        x, y = self.screen_offset
        self.center_x = inches(.3)

    def draw_debug(self, context):
        return [
            draw_x(context, 0, 0),
            *[context.circle(center=self.rotated((inches(i * .1), 0)), r=.25) for i in range(7)],
        ]
