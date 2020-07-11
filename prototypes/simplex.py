from opensimplex import OpenSimplex
import matplotlib.pyplot as plt
from matplotlib.widgets import Slider
import random

class MyNoise:
    def __init__(self, seed):
        self.seed = seed
        self.table = [i for i in range(256)]
        random.seed(seed) 
        random.shuffle(self.table)

    def noise2d(self, x, y):
        x_int = int(x)
        """
        a = x_int & 255
        b = (x_int >> 8) & 255
        c = (x_int >> 16) & 255
        d = (x_int >> 24) & 255
        index = a ^ b ^ c ^ d
        """
        a = self.table[x_int * 337 % 256]
        b = self.table[(x_int + 1) * 337 % 256]
        t = x - x_int
        return MyNoise.smooth_interp(a, b, t) / (255 / 2) - 1

    @staticmethod
    def lerp(a, b, t):
        return (1 - t) * a + t * b

    @staticmethod
    def smooth_interp(a, b, t):
        t = 6 * t**5 - 15 * t**4 + 10 * t**3
        return MyNoise.lerp(a, b, t)




        

MAX_OCTAVES = 1 #3
OCTAVE_STEP = 4

generators = [OpenSimplex(seed=i) for i in range(MAX_OCTAVES)]
#generators = [MyNoise(seed=i) for i in range(MAX_OCTAVES)]

FREQ = 1 #100

SMOOTHNESS = 0.9

OFFSET = 0

NUM_SAMPLES = 1000

output = [0 for i in range(NUM_SAMPLES)]
#output2 = [[0 for i in range(NUM_SAMPLES)] for _ in range(MAX_OCTAVES)]
def make_output():
    for i in range(NUM_SAMPLES):
        output[i] = 0
        max_value = 0
        for _oct, gen in enumerate(generators):
            oct = _oct * 3
            #decay_value = 1 / oct * SMOOTHNESS
            decay_value = SMOOTHNESS ** oct
            # if oct == 0:
            #     decay_value = 1
            # else:
            #     decay_value = SMOOTHNESS
            value = gen.noise2d(x=(i + OFFSET)/(FREQ / (oct + 1)), y=0) * decay_value
            output[i] += value
            max_value += decay_value
            #output2[_oct][i] = value
        output[i] /= max_value
    return output


fig, ax = plt.subplots()
plt.subplots_adjust(left=0.25, bottom=0.25)

graph, = plt.plot(make_output())
plt.ylim((-1, 1))

ax_freq = plt.axes([0.25, 0.1, 0.65, 0.03])
ax_smooth = plt.axes([0.25, 0.15, 0.65, 0.03])
ax_offset = plt.axes([0.25, 0.05, 0.65, 0.03])

s_freq = Slider(ax_freq, 'Frequency', 50, 200, valinit=FREQ)
s_smooth = Slider(ax_smooth, 'Smoothness', 0, 1, valinit=SMOOTHNESS)
s_offset = Slider(ax_offset, 'Offset', 0, 2000, valinit=OFFSET)

def update():
    y = make_output()
    graph.set_ydata(y)
    fig.canvas.draw_idle()

def update_freq(val):
    global FREQ
    FREQ = val
    update()

def update_smooth(val):
    global SMOOTHNESS
    SMOOTHNESS = val
    update()

def update_offset(val):
    global OFFSET
    OFFSET = val
    update()
    
s_freq.on_changed(update_freq)
s_smooth.on_changed(update_smooth)
s_offset.on_changed(update_offset)

plt.show()
