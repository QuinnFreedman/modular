"""
import numpy as np
import matplotlib.pyplot as plt

NUM_SAMPLES = 1000

data = np.arange(NUM_SAMPLES, dtype=float)

data = np.sin(data / NUM_SAMPLES * 4 * np.pi)

MAX_FOLD = 1
MIN_FOLD = -1
FOLD_AMOUNT = 1.2

data *= FOLD_AMOUNT

for i in range(len(data)):
    while data[i] > MAX_FOLD or data[i] < MIN_FOLD:
        if data[i] > MAX_FOLD:
            data[i] = MAX_FOLD - (data[i] - MAX_FOLD)
        elif data[i] < MIN_FOLD:
            data[i] = MIN_FOLD - (data[i] - MIN_FOLD)

plt.plot(data)
plt.show()
"""

import numpy as np
import matplotlib.pyplot as plt
from matplotlib.widgets import Slider

NUM_SAMPLES = 1000

input_data = np.arange(NUM_SAMPLES, dtype=float)
input_data = np.sin(input_data / NUM_SAMPLES * 4 * np.pi)

MAX_FOLD = 1
MIN_FOLD = -1
FOLD_AMOUNT = 1.2
SIGMOID_SCALE = 10
OFFSET = 0


def sigmoid(x):
    return 1 / (1 + np.e ** -x) * 2 - 1
    
def sigmoid2(x):
    return np.tanh(x)

def process_data(data):
    data = data + OFFSET
    data = data * FOLD_AMOUNT
    for i in range(len(data)):
        x = data[i]
        while x > MAX_FOLD or x < MIN_FOLD:
            if x > MAX_FOLD:
                x = MAX_FOLD - (x - MAX_FOLD)
            elif x < MIN_FOLD:
                x = MIN_FOLD - (x - MIN_FOLD)

        data[i] = sigmoid(x * SIGMOID_SCALE)
        data[i] /= sigmoid(1 * SIGMOID_SCALE)
    return data

fig, ax = plt.subplots()
plt.subplots_adjust(left=0.25, bottom=0.25)

graph, = plt.plot(process_data(input_data))

ax_fold = plt.axes([0.25, 0.1, 0.65, 0.03])
ax_smooth = plt.axes([0.25, 0.15, 0.65, 0.03])
ax_offset = plt.axes([0.25, 0.05, 0.65, 0.03])

s_fold = Slider(ax_fold, 'Fold', 1, 15, valinit=FOLD_AMOUNT)
s_smooth = Slider(ax_smooth, 'Smooth', 1, 15, valinit=SIGMOID_SCALE)
s_offset = Slider(ax_offset, 'Offset', -1, 1, valinit=OFFSET)

def update():
    y = process_data(input_data)
    graph.set_ydata(y)
    fig.canvas.draw_idle()

def update_fold(val):
    global FOLD_AMOUNT
    FOLD_AMOUNT = val
    update()

def update_smooth(val):
    global SIGMOID_SCALE
    SIGMOID_SCALE = val
    update()
    
def update_offset(val):
    global OFFSET
    OFFSET = val
    update()

s_fold.on_changed(update_fold)
s_smooth.on_changed(update_smooth)
s_offset.on_changed(update_offset)

plt.show()
