import numpy as np
import numpy.fft as fft

def dft_dot(a1, a2):
    return fft.ifft(fft.fft(a1) * fft.fft(a2))


kernel = np.array([1, 2, 3, 4, 5, 6, 7, 8], dtype=np.float64)

array = np.array([0.1, .3, .6, .7, .7, .6, .3, .1], dtype=np.float64)

# print("dot:", np.convolve(kernel, array))
# print("dft dot:", dft_dot(kernel, array))


class DftConvolver:
    def __init__(self, block_size):
        self.block_size = block_size
        self.input_buffer = []
        self.output_buffer = []
        self.out_buffer_start_time = None
        self.in_buffer_start_time = 0

    def do_opperation(self, data):
        return [f"{self.block_size}:{i}" for i in data]
    

    def push(self, value, time):
        self.input_buffer.append(value)
        if len(self.input_buffer) >= self.block_size:
            self.output_buffer = self.do_opperation(self.input_buffer)
            self.out_buffer_start_time = self.in_buffer_start_time
            self.input_buffer = []
            self.in_buffer_start_time = time + 1
        #print(f"input: {self.in_buffer_start_time}@{self.input_buffer}", f"output: {self.out_buffer_start_time}@{self.output_buffer}")

    def get_at_time(self, t):
        if self.out_buffer_start_time is None:
            return None
        if t < self.out_buffer_start_time:
            return f"{t} out of bounds <{self.out_buffer_start_time}-{self.out_buffer_start_time+self.block_size}>"
            return None
        if t >= self.out_buffer_start_time + self.block_size:
            return f"{t} out of bounds <{self.out_buffer_start_time}-{self.out_buffer_start_time+self.block_size}>"
            return None

        return self.output_buffer[t - self.out_buffer_start_time]


class NaiveConvolver:
    def __init__(self, kernel, out_buffer_size):
        self.kernel = kernel
        self.output_buffer = np.zeros(out_buffer_size)
        self.output_buffer_start = -out_buffer_size
        self.input_buffer = np.zeros(len(kernel))

    def push(self, value, time):
        assert time == self.output_buffer_start + len(self.output_buffer)
        self.input_buffer = np.roll(self.input_buffer, -1)
        self.input_buffer[-1] = value
        
        self.output_buffer = np.roll(self.output_buffer, -1)
        self.output_buffer[-1] = np.dot(self.input_buffer, np.flip(self.kernel))
        
        self.output_buffer_start += 1

    def get_at_time(self, t):
        if t < self.output_buffer_start:
            return f"{t} out of bounds <{self.output_buffer_start}-{self.output_buffer_start+len(self.output_buffer)}>"
        if t >= self.output_buffer_start + len(self.output_buffer):
            return f"{t} out of bounds <{self.output_buffer_start}-{self.output_buffer_start+len(self.output_buffer)}>"

        return self.output_buffer[t - self.output_buffer_start]


N = 4

#dfts = [DftConvolver(N), DftConvolver(2*N), DftConvolver(4*N)]

kernel = np.array([.5, 1, .5, .25])

convolver = NaiveConvolver(kernel, 1)
#convolver2 = NaiveConvolver([0, 0, 0, 0], 5)

data = np.arange(100)

resultA = []
for i in data:
    convolver.push(i, i)
    resultA.append(convolver.get_at_time(i))
    #get_at = i
    #print(f"time: {i}, conv@{get_at}: {convolver.get_at_time(get_at)}")

resultA = np.array(resultA)

print(resultA)

resultB = np.convolve(data, kernel, mode="same")

print(resultB)

resultC = []
padded_data = np.concatenate((np.zeros(len(kernel) - 1), data))
for i in range(len(padded_data) - len(kernel)):
    resultC.append(np.dot(padded_data[i:i+len(kernel)], np.flip(kernel)))

resultC = np.array(resultC)
print(resultC)
