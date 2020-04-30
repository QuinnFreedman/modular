import wave
import sys
import numpy as np
from tqdm import tqdm

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

        print("kernel raw max", np.max(data))
        print("kernel raw min", np.min(data))
        data = data.astype(np.float64) / 8388607
    else:
        data = np.frombuffer(bytestr, dtype=np.uint16)
        print("input raw max", np.max(data))
        print("input raw min", np.min(data))
        data = data.astype(np.float64) / 32767 - 1
        
    
    return data, framerate


kernel, rate = load(sys.argv[1])

data, _ = load(sys.argv[2])


print("max(kernel)", np.max(kernel))
print("min(kernel)", np.min(kernel))
print("sum(kernel)", np.sum(kernel))
print("max(data)", np.max(data))
print("min(data)", np.min(data))

output = np.zeros_like(data)

kernel = np.flip(kernel)

for i in tqdm(range(len(data))):
    length = min(i, 1000)#len(kernel))
    # print("length is:", length)
    # print("kernel:", kernel.shape)
    # print("data:", data[i-length:i].shape)
    output[i] = data[i]#np.sum(data[i-length:i])# np.sum(kernel[:length] * data[i-length:i])

print(output[100:200])

print("output max", np.max(output))
print("output min", np.min(output))
output = ((np.clip(output, -1, 1) + 1) * 32767).astype(np.int16)

writer = wave.open("output.wav", 'wb')
writer.setnchannels(2)
writer.setsampwidth(2)
writer.setframerate(rate)
writer.setnframes(len(data) // 2)
writer.writeframes(output)
