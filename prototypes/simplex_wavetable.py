from mpl_toolkits.mplot3d import Axes3D
import matplotlib.pyplot as plt
import numpy as np
from opensimplex import OpenSimplex
from matplotlib.widgets import Slider

TWO_PI = 2 * np.pi
RESOLUTION = 40
HARMONICS = .3
TABLE_DEPTH = 2

generator = OpenSimplex(seed=1)
output = np.zeros((RESOLUTION, RESOLUTION))

def make_output():
    for y in range(RESOLUTION):
        for x in range(RESOLUTION):
            _x = x / RESOLUTION
            while _x > 1:
                _x -= 1

            angle = _x * TWO_PI

            output[y, x] = generator.noise3d(
                (np.cos(angle) + 1) * HARMONICS,
                (np.sin(angle) + 1) * HARMONICS,
                y / RESOLUTION * TABLE_DEPTH)

        max = np.max(output[y])
        min = np.min(output[y])
        output[y] = (output[y] - min) * (1 / (max - min))

    z = output
    x, y = np.meshgrid(range(z.shape[1]), range(z.shape[0]))
    return x, y, z


fig = plt.figure()
ax = fig.add_subplot(111, projection='3d')
x, y, z = make_output()
graph = ax.plot_surface(x, y, z, color='blue')

ax_freq = plt.axes([0.25, 0.1, 0.65, 0.03])
s_freq = Slider(ax_freq, 'Harmonics', 0, 2, valinit=HARMONICS)

def update():
    global graph
    x, y, z = make_output()
    graph.remove()
    graph = ax.plot_surface(x, y, z, color='blue')
    
    
def update_freq(val):
    global HARMONICS
    HARMONICS = val
    update()
    
s_freq.on_changed(update_freq)

plt.show()
