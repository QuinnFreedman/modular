import sys
sys.path.append("..")
sys.path.append("../..")
sys.path.append("../../..")

import math
from faceplate_maker import *
from svgwrite.path import Path

icon_path = "M 6.6758458,2.7453636 H 8 m -1.6551926,1.4758802 1.4345,0.882769 M 6.2758413,1.1867251 7.5999949,0.12464356 M 1.3241536,2.7453636 H 0 M 1.6551919,4.2212438 0.22069231,5.1040128 M 1.7241583,1.186725 0.40000476,0.12464347 M 3.438421,6.6641275 3.5382878,7.0542841 H 4.3820817 L 4.5186428,6.8002901 M 3.3260769,6.2176145 3.1724577,6.4379869 3.3710946,6.6589848 4.5949859,6.7766287 4.7581247,6.5942204 4.6407527,6.3833303 v 0 M 3.2810592,5.7762441 3.12744,5.9966165 3.3260769,6.2176145 4.6531478,6.3523421 4.8617069,6.1734239 4.7255989,5.9167469 v 0 M 3.3000925,5.3920215 3.1317755,5.5698819 3.2810592,5.7762441 4.7255989,5.9167469 4.866101,5.6752581 4.6562807,5.3920215 m 0,0 c 0,-1.2877031 1.1345438,-1.3406439 1.1358962,-2.6865304 8.046e-4,-0.7984172 -0.7077897,-1.75099022 -1.8139902,-1.75099028 -1.1062007,0 -1.8147928,0.95257308 -1.8139906,1.75099028 0.00135,1.3458865 1.1358964,1.3988273 1.1358964,2.6865304 0.4520684,0.00474 0.9041187,0.00216 1.3561882,0 z"

module = Module.from_cli(2, global_y_offset=17.5, title="")

icon = module.d.path(
    icon_path,
    stroke_width=.344, 
    stroke="black",
    fill="none",
    transform=f"translate({-4}, {-10})",
)
module.stencil.add(icon)

module.add(LED(inches(0), inches(.1), color="red"))

module.add(TL1105SP(inches(0), inches(.6)))

jack_start = inches(1.6)
module.add(JackSocketCentered(0, jack_start + inches(0.0), "A", False, rotation=2))
module.add(JackSocketCentered(0, jack_start + 1 * inches(0.6), "R", False, rotation=2))
module.add(JackSocketCentered(0, jack_start + 2 * inches(0.6), "G", False, rotation=2))
module.add(JackSocketCentered(0, jack_start + 3 * inches(0.6), "B", False, rotation=2))

module.save()
