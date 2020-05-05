import wave
import sys
import numpy as np
from tqdm import tqdm

def graph_fft(data):
    import matplotlib.pyplot as plt
    from matplotlib.widgets import Slider
    chunk_size = 1000
    chunks = []
    for i in range(0, len(data), chunk_size):
        chunks.append(data[i:i+chunk_size])
    num_chunks = len(chunks)
    fts = []
    for chunk in tqdm(chunks):
        # ft = np.fft.rfft(data)
        # fts.append(ft)

        aud_data = data
        
        rate = 44100
        ii = np.arange(0, 9218368)
        t = ii / rate

        # From here down, everything else can be the same
        len_data = len(aud_data)

        channel_1 = np.zeros(2**(int(np.ceil(np.log2(len_data)))))
        channel_1[0:len_data] = aud_data

        fourier = np.fft.fft(channel_1)
        w = np.linspace(0, 44000, len(fourier))

        # First half is the real component, second half is imaginary
        fourier_to_plot = fourier[0:len(fourier)//2]
        w = w[0:len(fourier)//2]
        plt.plot(w, fourier_to_plot)
        
        plt.xlabel('frequency')
        plt.ylabel('amplitude')
        plt.show()
 
  

    fig, ax = plt.subplots()
    
    graph, = plt.plot(chunks[0])
    
    ax_chunk = plt.axes([0.25, 0.15, 0.65, 0.03])
    s_chunk = Slider(ax_chunk, 'Chunk', 0, num_chunks, valinit=0)
    def update_chunk(chunk):
        graph.set_ydata(chunks[int(chunk)])
        fig.canvas.draw_idle()
        
    s_chunk.on_changed(update_chunk)
    plt.show()
    

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


kernel, rate = load(sys.argv[1])

# graph_fft(kernel)
# sys.exit(0)

kernel = np.flip(kernel)

data, _ = load(sys.argv[2])

data = data[::2]
kernel = kernel[::2]


print("max(kernel)", np.max(kernel))
print("min(kernel)", np.min(kernel))
print("sum(kernel)", np.sum(kernel))
print("max(data)", np.max(data))
print("min(data)", np.min(data))

output = np.zeros_like(data)

kernel_sample = (np.random.rand(10000) * len(kernel)).astype(np.int64)
new_kernel = np.zeros_like(kernel)
new_kernel[kernel_sample] = kernel[kernel_sample]
new_kernel = kernel
for i in tqdm(range(len(data))):
    if i == 0:
        continue
    length = min(i, len(kernel))
    # print("length is:", length)
    # print("kernel:", kernel[-length:].shape)
    # print("data:", data[i-length:i].shape)
    output[i] = np.sum(new_kernel[-length:] * data[i-length:i])
    # sum = 0
    # for j in kernel_sample:
        # sum += kernel[-j] * data[i-j]
    # output[i] = sum

print(output[100:200])

print("output max", np.max(output))
print("output min", np.min(output))

output /= np.max(output)

output = (np.clip(output, -1, 1) * 32767).astype(np.int16)

writer = wave.open("output.wav", 'wb')
writer.setnchannels(1)
writer.setsampwidth(2)
writer.setframerate(rate)
writer.setnframes(len(data) // 2)
writer.writeframes(output)
