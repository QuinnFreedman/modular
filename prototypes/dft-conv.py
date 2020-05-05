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
        

N = 4

dfts = [DftConvolver(N), DftConvolver(2*N), DftConvolver(4*N)]

for i in range(100):
    dfts[0].push(i, i)
    get_at = i - 3
    print(f"time: {i}, conv@{get_at}: {dfts[0].get_at_time(get_at)}")
