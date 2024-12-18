import sys
sys.path.append("..")
sys.path.append("../..")
sys.path.append("../../..")

import math
from faceplate_maker import *
from svgwrite.path import Path

module = Module.from_cli(10, global_y_offset=12, title="Quantizer")

def draw_line(p1, p2):
    (x1, y1), (x2, y2) = p1, p2
    path = Path(stroke="black", fill="none", stroke_width=.5)
    path.push(f"M {x1} {y1}")
    path.push(f"L {x2}, {y2}")
    module.draw(lambda _: path)

circle_center = inches(.9)

black_keys = [True, False, False, True, False, True, False, True, False, False, True, False]
path_data = [
    # "M 0.37578,0.174503 V 2.825497 H 2.62422 V 0.857147 L 2.09764,0.174503 H 1.92969 V 0.783247 H 0.78868 V 0.174503 Z m 1.14825,0.05426 V 0.648892 H 1.74211 V 0.228764 Z M 0.76129,1.662787 h 1.48053 v 0.80253 H 0.76129 Z",
    # "m -0.06136709,0.4011005 v 2.197282 h 0.30179 v 5.17e-4 H 2.5250379 l 0.602031,-1.369425 H 0.84245391 l -0.438733,0.997872 V 0.8677395 H 2.5069519 v -0.466639 z",
    "M 0.10335286 0.16381429 L 0.10335286 2.8360026 L 0.60358073 2.8360026 L 0.60358073 1.749764 L 1.3730428 1.749764 L 1.3730428 2.3822835 L 2.0138306 1.9409668 L 2.4530802 1.6386597 L 2.4530802 2.8360026 L 2.9533081 2.8360026 L 2.9533081 0.16381429 L 2.4530802 0.16381429 L 2.4530802 1.3611572 L 2.0138306 1.0588501 L 1.3730428 0.61753337 L 1.3730428 1.2500529 L 0.60358073 1.2500529 L 0.60358073 0.16381429 L 0.10335286 0.16381429 z",
    "M -0.42501411,0.05430934 V 0.5138234 h 0.72230017 v 1.755604 l -0.37899061,0.5747543 0.002411,0.00145 H 0.58803839 L 2.427059,0.05430934 H 1.7573161 L 1.5586595,0.35566955 V 0.05430934 Z M 2.4429708,0.46946317 c -0.036388,0 -0.071672,0.001381 -0.1060787,0.003857 l -0.68903,1.04632263 c 0.034591,0.056031 0.076159,0.1061641 0.1248836,0.1494747 0.062212,0.052996 0.1323695,0.097661 0.2107111,0.1345272 0.078341,0.034562 0.1564784,0.065892 0.2348199,0.093542 0.1797246,0.059908 0.3067039,0.1162486 0.3804371,0.1692439 0.073733,0.050691 0.1104183,0.1150112 0.1104183,0.1933527 0,0.036867 -0.00584,0.070339 -0.017358,0.1002927 C 2.6802531,2.39003 2.6595271,2.416593 2.6295731,2.439635 2.5996191,2.460373 2.5597211,2.477885 2.509029,2.49171 2.4606415,2.5032316 2.3979819,2.5085869 2.3219446,2.5085869 2.1698699,2.5085868 2.0412306,2.4936985 1.9352391,2.4637446 1.8315519,2.4314864 1.743882,2.3958722 1.672453,2.3567014 L 1.5205675,2.7853562 c 0.032258,0.018433 0.071086,0.038088 0.1171688,0.058826 0.048387,0.020738 0.1047273,0.040392 0.1692439,0.058825 0.066821,0.018433 0.1432976,0.033321 0.2285516,0.044842 0.085254,0.013825 0.1804213,0.020734 0.2864128,0.020734 0.3156702,0 0.5521161,-0.061106 0.7087992,-0.183227 0.156683,-0.1244247 0.2348199,-0.2986933 0.2348199,-0.522197 0,-0.1152081 -0.014888,-0.2129997 -0.044842,-0.2936454 C 3.1907672,1.8888678 3.1445491,1.8187106 3.0823367,1.7588027 3.0224284,1.6988944 2.9464338,1.6468387 2.8542673,1.6030597 2.7621007,1.5569762 2.6541873,1.5112403 2.5297626,1.4651573 2.4698545,1.4444196 2.4145857,1.4247649 2.3638939,1.4063318 2.3155066,1.3855946 2.2714306,1.3637976 2.2322598,1.3407558 c -0.036866,-0.025346 -0.065572,-0.05298 -0.08631,-0.082934 -0.020738,-0.029954 -0.031341,-0.067122 -0.031341,-0.1109006 0,-0.073733 0.027635,-0.1279307 0.082934,-0.1624934 0.057604,-0.0368668 0.1511113,-0.0549681 0.2801445,-0.0549681 0.115208,0 0.2129997,0.0138172 0.2936453,0.0414672 0.08295,0.0253452 0.1578735,0.0551221 0.2246942,0.0896848 L 3.1517701,0.63533184 C 3.0757327,0.59385688 2.9790122,0.55610061 2.8614999,0.52153822 2.7439877,0.48697582 2.6042622,0.46946317 2.4429708,0.46946317 Z M 0.83635921,0.5138234 H 1.4545094 L 0.83635921,1.4516563 Z",
    "m 2.577453,0.0887165 -0.63252,1.095023 h 0.400492 v 0.632003 H 1.944933 l 0.63252,1.095541 0.632002,-1.095541 H 2.809996 v -0.632003 h 0.399459 z m -1.406633,0.03101 a 0.31675062,0.31675062 0 0 0 -0.304891,0.232544 0.31675062,0.31675062 0 0 0 -0.385506,-0.05426 0.31675062,0.31675062 0 0 0 -0.137976,0.383956 0.31675062,0.31675062 0 0 0 -0.366903,0.143661 0.31675062,0.31675062 0 0 0 0.04031,0.371553 0.31675062,0.31675062 0 0 0 -0.225309,0.302824 0.31675062,0.31675062 0 0 0 0.23151,0.304891 0.31675062,0.31675062 0 0 0 -0.04651,0.378788 0.31675062,0.31675062 0 0 0 0.366903,0.143661 0.31675062,0.31675062 0 0 0 0.137976,0.383956 0.31675062,0.31675062 0 0 0 0.387573,-0.05633 0.31675062,0.31675062 0 0 0 0.302824,0.225309 0.31675062,0.31675062 0 0 0 0.307475,-0.242879 0.31675062,0.31675062 0 0 0 0.382405,0.05168 0.31675062,0.31675062 0 0 0 0.116272,-0.433049 0.31675062,0.31675062 0 0 0 -0.433048,-0.115755 0.31675062,0.31675062 0 0 0 -0.148828,0.201022 0.31675062,0.31675062 0 0 0 -0.224276,-0.09457 0.31675062,0.31675062 0 0 0 -0.228927,0.0987 A 0.31675062,0.31675062 0 0 0 0.7972,2.1624935 0.31675062,0.31675062 0 0 0 0.54657,2.1345835 0.31675062,0.31675062 0 0 0 0.52383,1.8668995 0.31675062,0.31675062 0 0 0 0.333144,1.7206555 a 0.31675062,0.31675062 0 0 0 0.09095,-0.220659 0.31675062,0.31675062 0 0 0 -0.08268,-0.213423 0.31675062,0.31675062 0 0 0 0.182418,-0.144178 0.31675062,0.31675062 0 0 0 0.02326,-0.267167 0.31675062,0.31675062 0 0 0 0.250113,-0.02842 0.31675062,0.31675062 0 0 0 0.147278,-0.190169 0.31675062,0.31675062 0 0 0 0.226343,0.09663 0.31675062,0.31675062 0 0 0 0.222209,-0.09147 0.31675062,0.31675062 0 0 0 0.150895,0.207222 0.31675062,0.31675062 0 0 0 0.433048,-0.115755 0.31675062,0.31675062 0 0 0 -0.116272,-0.432532 0.31675062,0.31675062 0 0 0 -0.380855,0.04961 0.31675062,0.31675062 0 0 0 -0.309025,-0.25063 z",
    "M 1.5002595,0.0887165 0.86774053,1.1837405 H 1.2682325 v 0.632003 H 0.86774053 l 0.63251897,1.09554 0.632003,-1.09554 h -0.399459 v -0.632003 h 0.399459 z m 0.987537,0.551905 a 0.31675062,0.31675062 0 0 0 -0.173633,0.04186 0.31675062,0.31675062 0 0 0 -0.115755,0.433049 0.31675062,0.31675062 0 0 0 0.200504,0.148828 0.31675062,0.31675062 0 0 0 -0.100252,0.230477 0.31675062,0.31675062 0 0 0 0.107487,0.238228 0.31675062,0.31675062 0 0 0 -0.207739,0.151412 0.31675062,0.31675062 0 0 0 0.115755,0.432532 0.31675062,0.31675062 0 0 0 0.433048,-0.115755 0.31675062,0.31675062 0 0 0 -0.06615,-0.397392 0.31675062,0.31675062 0 0 0 0.25063,-0.309025 0.31675062,0.31675062 0 0 0 -0.241845,-0.307475 0.31675062,0.31675062 0 0 0 0.05736,-0.388607 0.31675062,0.31675062 0 0 0 -0.259415,-0.158129 z m -1.97507297,0.02222 a 0.31675062,0.31675062 0 0 0 -0.259416,0.15813 0.31675062,0.31675062 0 0 0 0.04031,0.371037 0.31675062,0.31675062 0 0 0 -0.225309,0.302824 0.31675062,0.31675062 0 0 0 0.232027,0.305408 0.31675062,0.31675062 0 0 0 -0.04702,0.378788 0.31675062,0.31675062 0 0 0 0.433048,0.115755 0.31675062,0.31675062 0 0 0 0.115756,-0.432532 0.31675062,0.31675062 0 0 0 -0.190686,-0.146244 0.31675062,0.31675062 0 0 0 0.09043,-0.221175 0.31675062,0.31675062 0 0 0 -0.08216,-0.212907 0.31675062,0.31675062 0 0 0 0.182418,-0.144177 0.31675062,0.31675062 0 0 0 -0.115756,-0.433049 0.31675062,0.31675062 0 0 0 -0.173632,-0.04186 z",
    "m 0.422547,0.0887165 0.63252,1.095023 H 0.654575 v 0.632003 h 0.400492 l -0.63252,1.095541 -0.632002,-1.095541 h 0.399459 v -0.632003 h -0.399459 z m 1.406633,0.03101 a 0.31675062,0.31675062 0 0 1 0.304891,0.232544 0.31675062,0.31675062 0 0 1 0.385506,-0.05426 0.31675062,0.31675062 0 0 1 0.137976,0.383956 0.31675062,0.31675062 0 0 1 0.366903,0.143661 0.31675062,0.31675062 0 0 1 -0.04031,0.371553 0.31675062,0.31675062 0 0 1 0.225309,0.302824 0.31675062,0.31675062 0 0 1 -0.23151,0.304891 0.31675062,0.31675062 0 0 1 0.04651,0.378788 0.31675062,0.31675062 0 0 1 -0.366903,0.143661 0.31675062,0.31675062 0 0 1 -0.137976,0.383956 0.31675062,0.31675062 0 0 1 -0.387573,-0.05633 0.31675062,0.31675062 0 0 1 -0.302824,0.225309 0.31675062,0.31675062 0 0 1 -0.307475,-0.242879 0.31675062,0.31675062 0 0 1 -0.382405,0.05168 0.31675062,0.31675062 0 0 1 -0.116272,-0.433049 0.31675062,0.31675062 0 0 1 0.433048,-0.115755 0.31675062,0.31675062 0 0 1 0.148828,0.201022 0.31675062,0.31675062 0 0 1 0.224276,-0.09457 0.31675062,0.31675062 0 0 1 0.228927,0.0987 0.31675062,0.31675062 0 0 1 0.144694,-0.182935 0.31675062,0.31675062 0 0 1 0.25063,-0.02791 0.31675062,0.31675062 0 0 1 0.02274,-0.267684 0.31675062,0.31675062 0 0 1 0.190686,-0.146244 0.31675062,0.31675062 0 0 1 -0.09095,-0.220659 0.31675062,0.31675062 0 0 1 0.08268,-0.213423 0.31675062,0.31675062 0 0 1 -0.182418,-0.144178 0.31675062,0.31675062 0 0 1 -0.02325,-0.267167 0.31675062,0.31675062 0 0 1 -0.250113,-0.02842 0.31675062,0.31675062 0 0 1 -0.147278,-0.190169 0.31675062,0.31675062 0 0 1 -0.226343,0.09663 0.31675062,0.31675062 0 0 1 -0.222209,-0.09147 0.31675062,0.31675062 0 0 1 -0.150895,0.207222 0.31675062,0.31675062 0 0 1 -0.433048,-0.115755 0.31675062,0.31675062 0 0 1 0.116272,-0.432532 0.31675062,0.31675062 0 0 1 0.380855,0.04961 0.31675062,0.31675062 0 0 1 0.309025,-0.25063 z",
    "m 1.7049835,0.2844615 v 0.711756 h 0.89917 v 0.406522 h -0.89917 v 0.712431 h -0.409967 v -0.712431 h -0.89917 v -0.406522 h 0.89917 v -0.711756 z m -1.309137,2.02111 h 2.208307 v 0.409967 h -2.208307 z",
    "m 1.704121,0.2326405 c -0.356693,0 -0.669057,0.200305 -0.829923,0.494027 h 0.421679 c 0.06718,0 0.132934,0.0062 0.196888,0.01809 0.06348,-0.0303 0.134849,-0.04754 0.211356,-0.04754 H 2.54593 c 0.270218,0 0.480591,0.210373 0.480591,0.480591 0,0.270218 -0.210373,0.480591 -0.480591,0.480591 H 1.704121 c -0.147311,0 -0.276484,-0.06319 -0.363802,-0.163814 -0.01448,-0.0018 -0.02925,-0.0026 -0.04444,-0.0026 H 0.813221 c 0.13037,0.366496 0.481218,0.630969 0.890902,0.630969 h 0.841809 c 0.519785,0 0.945162,-0.425377 0.945162,-0.945162 0,-0.519784 -0.425377,-0.945162 -0.945162,-0.945162 z m -1.250053,0.644405 c -0.519784,0 -0.945161,0.425379 -0.945162,0.945162 0,0.519785 0.425378,0.945162 0.945162,0.945162 h 0.841809 c 0.356697,0 0.669059,-0.200301 0.829924,-0.494026 H 1.732026 c -0.0741,0 -0.146367,-0.0079 -0.216524,-0.02222 -0.06552,0.03287 -0.139742,0.05168 -0.219625,0.05168 H 0.454068 c -0.270218,0 -0.480591,-0.210372 -0.480591,-0.480591 0,-0.270217 0.210373,-0.48059 0.480591,-0.48059 h 0.841809 c 0.144299,0 0.270906,0.06059 0.358118,0.157613 0.0249,0.0056 0.05078,0.0088 0.07803,0.0088 h 0.454753 c -0.13037,-0.366497 -0.481217,-0.63097 -0.890902,-0.63097 z",
    "M 2.0193485,2.3173494 H 0.9823741 L 0.81873196,2.7858827 H 0.15210555 L 1.1046751,0.2141173 H 1.895325 L 2.8478945,2.7858827 H 2.1812681 Z M 1.1477388,1.8402034 h 0.7045225 l -0.3514,-1.023194 z",
    "m 1.4715779,1.2097505 q 0.156752,0 0.2377118,-0.068902 0.08096,-0.068902 0.08096,-0.20326081 0,-0.13263626 -0.08096,-0.20153822 -0.08096,-0.0706245 -0.2377118,-0.0706245 H 1.104675 V 1.2097505 Z m 0.022393,1.1248244 q 0.1998156,0 0.2997235,-0.084405 0.1016304,-0.084405 0.1016304,-0.2549372 0,-0.1670873 -0.099908,-0.2497696 -0.099908,-0.084405 -0.301446,-0.084405 H 1.104675 V 2.3345749 Z M 2.1106436,1.4095662 q 0.213596,0.062012 0.3307294,0.229099 0.1171333,0.1670872 0.1171333,0.4099666 0,0.3720706 -0.2514922,0.5546607 Q 2.055522,2.7858827 1.5422025,2.7858827 H 0.44149373 V 0.21411727 H 1.437127 q 0.5357127,0 0.775147,0.16191959 0.2411568,0.1619196 0.2411568,0.51848721 0,0.18775783 -0.08785,0.32039413 -0.08785,0.1309137 -0.2549372,0.194648 z",
    "m 1.5945448,0.22704506 c 0.410144,0.0063 0.803216,0.211334 1.040246,0.565857 0.316042,0.47269404 0.279165,1.09973304 -0.0894,1.53272304 -0.368571,0.432987 -0.983066,0.568775 -1.500167,0.332279 l 0.19327,-0.421679 c 0.32991,0.150883 0.717765,0.06489 0.952913,-0.211357 0.235147,-0.276248 0.258479,-0.673038 0.05684,-0.974618 C 2.0466138,0.74867106 1.6714818,0.61864106 1.3263398,0.73037306 1.0962158,0.80487306 0.91956178,0.97426111 0.83076278,1.1861591 H 1.4095388 l -0.82165502,1.423169 -0.821656,-1.423169 h 0.575159 c 0.106204,-0.41557704 0.419155,-0.76062804 0.84232602,-0.89761904 0.135244,-0.04378 0.274113,-0.0636 0.410828,-0.0615 z",
    "m 1.4058221,0.22704508 c -0.41014396,0.0063 -0.803216,0.211334 -1.040246,0.565857 -0.316042,0.47269402 -0.279165,1.09973302 0.0894,1.53272302 0.36857104,0.432987 0.983066,0.568775 1.500167,0.332279 l -0.19327,-0.421679 c -0.32991,0.150883 -0.717765,0.06489 -0.95291296,-0.211357 C 0.5738131,1.7486201 0.5504811,1.3518301 0.75212014,1.0502501 0.95375314,0.74867108 1.3288851,0.61864108 1.6740271,0.73037308 c 0.230124,0.0745 0.406778,0.243888 0.495577,0.45578602 h -0.578776 l 0.821655,1.423169 0.821656,-1.423169 H 2.6589801 C 2.5527761,0.77058208 2.2398251,0.42553108 1.8166541,0.28854008 c -0.135244,-0.04378 -0.274113,-0.0636 -0.410828,-0.0615 z",
    "m 2.7929688,0.37890625 c -0.5029022,0 -0.8496417,0.1282258 -1.0742188,0.34179687 C 1.4941729,0.9342742 1.4064891,1.197779 1.3242187,1.4179687 1.2419484,1.6381585 1.1680308,1.8153568 1.0292969,1.9375 0.89056297,2.0596432 0.66621394,2.15625 0.20703125,2.15625 v 0.4648438 c 0.52889681,0 0.89101515,-0.1247755 1.13085935,-0.3359375 C 1.5777348,2.0739942 1.6759905,1.8062485 1.7597656,1.5820312 1.8435408,1.357814 1.9151111,1.1763748 2.0410156,1.0566406 2.1669201,0.93690645 2.3639844,0.84375 2.7929688,0.84375 Z",
]

for i in range(12):
    # r1 = inches(.675)
    # r2 = inches(.455)
    # r3 = inches(.3)
    r1 = inches(.75)
    r2 = inches(.5)
    r3 = inches(.35)
    theta = math.pi * 2 * i / 12
    x = math.cos(theta)
    y = math.sin(theta)
    module.add(TL1105SP(x * r1, circle_center + y * r1))
    module.add(SmallLED(x * r2, circle_center + y * r2 - inches(.05)))

    def add_icon(d):
        group = d.g(transform=f"translate({(x * r3) - 1.5},{circle_center + (y * r3) - 1.5})")
        path = Path(d=path_data[i], stroke="none", fill="black")
        group.add(path)
        return group

    module.draw(add_icon)

    circle_r = inches(.15)
    module.draw(lambda g: g.circle(center=(x*r1, y*r1+circle_center), r=circle_r, stroke_width=.4, stroke="black", fill="black" if black_keys[i] else "none"))
    draw_line((x*(r1-circle_r), y*(r1-circle_r) + circle_center), (x*r2, y*r2 + circle_center))
    
path = Path(stroke="none", fill="black")
path.push(f"M {0} {inches(2.6)}")
path.push(f"v {inches(1.35)}")
module.draw(lambda _: path)

def draw_text(text, x, y, **text_props):
    module.draw(lambda ctx: ctx.text(text, insert=(x, y), **text_props))

menu_button_y = inches(2.05)
button_label_y = menu_button_y + inches(.25)

module.add(TL1105SP(inches(-.7), menu_button_y))
draw_text("Shift", inches(-.7), button_label_y, text_anchor="middle")
# draw_text("Shift", inches(-.525), inches(2.24), text_anchor="start")

module.add(TL1105SP(inches(.275), menu_button_y))
draw_text("Load", inches(.275), button_label_y, text_anchor="middle")
module.add(TL1105SP(inches(.7), menu_button_y))
draw_text("Save", inches(.7), button_label_y, text_anchor="middle")

draw_text("Channel A", inches(-.5), inches(2.575), text_anchor="middle", font_size=4)
draw_text("Channel B", inches(.5), inches(2.575), text_anchor="middle", font_size=4)

jack_start_y = inches(2.95)
jack_spacing_y = inches(0.7)
jack_pos_x = inches(.275)
jack_spacing_x  = inches(.425)

module.add(JackSocketCentered(-(jack_pos_x + jack_spacing_x), jack_start_y, "V/O In", False, rotation=2))
module.add(JackSocketCentered(-jack_pos_x, jack_start_y, "Sample", False, rotation=2))
module.add(JackSocketCentered(-(jack_pos_x + jack_spacing_x), jack_start_y + jack_spacing_y, "Trig", True, rotation=2))
module.add(JackSocketCentered(-jack_pos_x, jack_start_y + jack_spacing_y, "Out", True, rotation=2))

led_pos_1 = -(jack_pos_x + jack_spacing_x / 2), jack_start_y - jack_spacing_x / 2 - inches(.05)
module.add(SmallLED(*led_pos_1))
draw_line((led_pos_1[0], led_pos_1[1] + inches(.05)), (-jack_pos_x, jack_start_y))
led_pos_2 = -(jack_pos_x + jack_spacing_x / 2), jack_start_y + jack_spacing_y - jack_spacing_x / 2 - inches(.05)
module.add(SmallLED(*led_pos_2))
draw_line((led_pos_2[0], led_pos_2[1] + inches(.05)), (-(jack_pos_x+jack_spacing_x), jack_start_y+jack_spacing_y))

module.add(JackSocketCentered(jack_pos_x, jack_start_y, "V/O In", False, rotation=2))
module.add(JackSocketCentered(jack_pos_x + jack_spacing_x, jack_start_y, "Sample", False, rotation=2))
module.add(JackSocketCentered(jack_pos_x, jack_start_y + jack_spacing_y, "Trig", True, rotation=2))
module.add(JackSocketCentered(jack_pos_x + jack_spacing_x, jack_start_y + jack_spacing_y, "Out", True, rotation=2))

led_pos_1 = (jack_pos_x + jack_spacing_x / 2), jack_start_y - jack_spacing_x / 2 - inches(.05)
module.add(SmallLED(*led_pos_1))
draw_line((led_pos_1[0], led_pos_1[1] + inches(.05)), (jack_pos_x+jack_spacing_x, jack_start_y))
led_pos_2 = (jack_pos_x + jack_spacing_x / 2), jack_start_y + jack_spacing_y - jack_spacing_x / 2 - inches(.05)
module.add(SmallLED(*led_pos_2))
draw_line((led_pos_2[0], led_pos_2[1] + inches(.05)), (jack_pos_x, jack_start_y+jack_spacing_y))

path = Path(stroke="black", fill="none", stroke_width=.5)
path.push(f"M {0} {inches(2.6)}")
path.push(f"v {inches(1.35)}")
module.draw(lambda _: path)

module.add(M3Bolt(-inches(0.8), inches(.2)))
module.add(M3Bolt(inches(0.8), inches(.2)))

module.save()
