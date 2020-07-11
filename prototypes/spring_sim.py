import wave
import sys
import numpy as np
from tqdm import tqdm
import matplotlib.pyplot as plt
from matplotlib.widgets import Slider

def load(filename):
    print("loading", filename)
    
    wav_file = wave.open(filename, 'rb')

    nchannels = wav_file.getnchannels()
    depth     = wav_file.getsampwidth()
    framerate = wav_file.getframerate()
    nframes   = wav_file.getnframes()

    bytestr = wav_file.readframes(nframes)

    wav_file.close()

    print("depth:", depth)
    print("nframes:", nframes)
    print("nchannels:", nchannels)
    print("depth * nframes * nchannels:", depth * nframes * nchannels)
    print("len(bytestr):", len(bytestr))

    #if depth =
    #output = np.frombuffer(data,'b').reshape(-1,3) #[:,1:].flatten().view('i4')

    if depth == 3:
        data = np.frombuffer(bytestr, dtype=np.uint8)
        a = np.empty((len(data)//3, 4), dtype=np.uint8)
        a[:, :3] = data.reshape((-1, 3))
        a[:, 3:] = (a[:, 3 - 1:3] >> 7) * 255
        data = a.view(np.int32).reshape(a.shape[:-1])

        print("raw max", np.max(data))
        print("raw min", np.min(data))
        data = data.astype(np.float64) / 8388607
    else:
        assert False
        data = np.frombuffer(bytestr, dtype=np.uint16)
        print("input raw max", np.max(data))
        print("input raw min", np.min(data))
        data = data.astype(np.float64) / 32767 - 1
        
    
    return data, framerate




# data, rate = load(sys.argv[1])

# data = data[::2]

data = np.zeros(200, dtype=np.float64)
data[10] = 1

print("max(data)", np.max(data))
print("min(data)", np.min(data))

output = np.zeros_like(data)

SPRING_LEN = 100
spring_pos = np.zeros(SPRING_LEN, dtype=np.float64)
spring_velocity = np.zeros_like(spring_pos)

DAMPING = 0.1
STIFFNESS = 0.55

data_to_graph = []

for j in tqdm(range(len(data))):
    spring_pos[0] = data[j]
    for i in range(1, SPRING_LEN - 1):
        force = spring_pos[i - 1] - 3 * spring_pos[i] + spring_pos[i + 1]
        #if i == 1:
            #tqdm.write(f"force: {force}")
        if abs(spring_velocity[i]) < DAMPING:
            spring_velocity[i] = 0
        else:
            spring_velocity[i] -= np.sign(spring_velocity[i]) * DAMPING
        spring_velocity[i] += force * STIFFNESS
        #if i == 1:
            #tqdm.write(f"velocity: {spring_velocity[i]}")
        spring_pos[i] = spring_velocity[i]
    output[j] = spring_pos[-2]
    data_to_graph.append(spring_pos.copy())


print("output max", np.max(output))
print("output min", np.min(output))

fig, ax = plt.subplots()
ax.set_ylim((-1, 1))

graph, = plt.plot(data_to_graph[0])

ax_time = plt.axes([0.25, 0.15, 0.65, 0.03])
s_time = Slider(ax_time, 'time', 0, len(output - 2), valinit=0)
def update_time(time):
    graph.set_ydata(data_to_graph[int(time)])
    fig.canvas.draw_idle()
    
s_time.on_changed(update_time)
plt.show()


#plt.plot(output)
#plt.show()
sys.exit(0)

output /= np.max(output)

output = (np.clip(output, -1, 1) * 32767).astype(np.int16)

writer = wave.open("output.wav", 'wb')
writer.setnchannels(1)
writer.setsampwidth(2)
writer.setframerate(rate)
writer.setnframes(len(data) // 2)
writer.writeframes(output)
