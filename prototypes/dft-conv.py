import numpy as np
import numpy.fft as fft
from collections import UserList
from tqdm import tqdm

np.set_printoptions(suppress=True)

class ZeroPaddedBuffer(UserList):
    def __getitem__(self, index):
        if isinstance(index, int):
            assert index < len(self)
            if index < 0 or index >= len(self):
                return 0
            return super().__getitem__(index)

        if isinstance(index, slice):
            assert index.start < len(self)
            result = []
            for i in range(index.start, index.stop):
                if i < 0 or i >= len(self):
                    result.append(0)
                else:
                    result.append(super().__getitem__(i))
            return result
            
        return super().__getitem__(index)

class ZeroPrepaddedBuffer(UserList):
    def __getitem__(self, index):
        if isinstance(index, int):
            if index < 0:
                return 0
            return super().__getitem__(index)

        if isinstance(index, slice):
            result = []
            for i in range(index.start, index.stop):
                if i < 0:
                    result.append(0)
                else:
                    result.append(super().__getitem__(i))
            return result
            
        return super().__getitem__(index)

class NaiveConvolver:
    def __init__(self, kernel, input_buffer, offset):
        self.kernel = kernel
        self.input_buffer = input_buffer
        if True:
            self.input_buffer_ptr = 0
            self.output_buffer = np.zeros(1 + offset)
        else:
            self.input_buffer_ptr = -offset
            self.output_buffer = np.zeros(1)

    def update(self):
        self.input_buffer_ptr += 1
        output_val = np.dot(
            self.input_buffer[
                self.input_buffer_ptr - len(self.kernel):
                self.input_buffer_ptr
            ],
            np.flip(self.kernel)
        )
        self.output_buffer = np.roll(self.output_buffer, -1)
        self.output_buffer[-1] = output_val

    def get_current_value(self):
        return self.output_buffer[0]


class BufferedConvolver:
    def __init__(self, kernel, input_buffer, offset, block_size):
        assert block_size <= offset
        self.kernel = kernel
        self.input_buffer = input_buffer
        self.block_size = block_size
        self.block_start_ptr = 0
        self.input_length = 0
        self.output_buffer = np.zeros(offset)

    def convolve_block(self):
        result = np.empty(self.block_size)
        for i in range(self.block_size):
            input_ptr = i + self.block_start_ptr + 1
            result[i] = np.dot(
                self.input_buffer[
                    input_ptr - len(self.kernel):
                    input_ptr],
                np.flip(self.kernel)
            )

        return result

    def update(self):
        self.input_length += 1
        self.output_buffer = np.roll(self.output_buffer, -1)
        self.output_buffer[-1] = 0
        if self.input_length - self.block_start_ptr > self.block_size:
            result = self.convolve_block()
            self.block_start_ptr += self.block_size
            self.output_buffer[-self.block_size:] = result

    def get_current_value(self):
        return self.output_buffer[0]

        
class DFTBufferedConvolver:
    def __init__(self, kernel, input_buffer, offset, N):
        self.input_buffer = input_buffer
        
        self.N = N
        self.M = len(kernel)
        self.L = N - (self.M - 1)
        self.h = np.array(kernel.tolist() + ([0] * (self.L - 1)))

        """
        print("offset:", offset)
        print("N:", self.N)
        print("M:", self.M)
        print("L:", self.L) 
        print("kernel:", len(kernel))
        print("h:", len(self.h))
        """
        
        assert self.L <= offset
        assert self.M <= self.N

        self.block_start_ptr = 0
        self.input_length = 0
        self.output_buffer = np.zeros(offset)

        self.last_block = None

    @staticmethod
    def circ_conv(signal, kernel):
        return np.real(np.fft.ifft( np.fft.fft(signal) * np.fft.fft(kernel)  ))

    def convolve_block(self):
        block = np.array(self.input_buffer[self.block_start_ptr-1:self.block_start_ptr+self.L-1])
        #print("input:", block)
        if self.last_block is None:
            padded_block = ([0] * (self.M - 1)) + block.tolist()
        else:
            padded_block = self.last_block.tolist()[-(self.M-1):] + block.tolist()
        self.last_block = np.array(padded_block)
        return DFTBufferedConvolver.circ_conv(padded_block, self.h)[self.M-1:]

    def update(self):
        self.input_length += 1
        self.output_buffer = np.roll(self.output_buffer, -1)
        self.output_buffer[-1] = -1
        if self.input_length - self.block_start_ptr >= self.L:
            result = self.convolve_block()
            self.block_start_ptr += self.L
            self.output_buffer[-self.L:] = result
        #print("time t =", self.input_length, "| output:", self.output_buffer)

    def get_current_value(self):
        return self.output_buffer[0]

"""
if __name__ == "__main__":
    print("Running tests...")
    for test in tqdm(range(100)):
        seed = np.random.randint(9999)
        np.random.seed(seed)
        kernel_length = np.random.randint(3, 30)
        kernel = np.random.randint(low=0, high=100, size=kernel_length, dtype=np.int)
        
        data = ZeroPrepaddedBuffer()
        split = np.random.randint(1, kernel_length - 1)
        convolver1 = NaiveConvolver(kernel[:split], data, 0)
        convolver2 = BufferedConvolver(kernel[split:], data, split, split)

        result = []
        for i in range(1, 100):
            data.append(i)
            convolver1.update()
            convolver2.update()
            result1 = convolver1.get_current_value()
            result2 = convolver2.get_current_value()
            result.append(result1 + result2)
        result = np.array(result, dtype=np.int)

        numpy_result = np.convolve(np.array(data.data), kernel, mode="full")[:-(len(kernel)-1)]

        naive_result = []
        for i in range(1, len(data) + 1):
            naive_result.append(np.dot(data[i-len(kernel):i], np.flip(kernel)))
        naive_result = np.array(naive_result, dtype=np.int)

        assert np.array_equal(numpy_result, naive_result)
        if not np.array_equal(numpy_result, result):
            print("test faild for seed", seed)
            print("kernel == ", kernel)
            print("split == ", split)
            print("result:")
            print(result)
            print("correct:")
            print(numpy_result)
            sys.exit(1)
"""

#SBS = 64 # or 32 -- starting block size, aka N

kernel = np.array([-3, -2, 5, 0, 1, 2, 3, 4])
#input = [1,2,3,4,5,6,7,8,9,1,2,3,4,5,6,7,8,9,1,2,3,4,5,6,7]
input = np.arange(25) + 1
"""
#kernel = np.array([.5, 1, .5, .25, .2, .15, .1, .01, .8, .7, .2])
kernel = np.array([0, 0, 0, 0, 0, 0, .1, .01, .8, .7, .2, .1, .3, .4 ])
kernel = (kernel * 25).astype(np.int)
"""

data = ZeroPrepaddedBuffer()
split = 4
convolver1 = NaiveConvolver(kernel[:split], data, 0)
convolver2 = DFTBufferedConvolver(kernel[split:], data, offset=split, N=4)


resultA = []
for i in input:#range(1, 100):
    data.append(i)
    convolver1.update()
    convolver2.update()
    result1 = int(round(convolver1.get_current_value()))
    result2 = int(round(convolver2.get_current_value()))
    resultA.append(result1 + result2)

print(np.array(resultA))


resultB = np.convolve(np.array(data.data), kernel, mode="full")[:-(len(kernel)-1)]
print(resultB)


resultC = []
for i in range(1, len(data) + 1):
    resultC.append(np.dot(data[i-len(kernel):i], np.flip(kernel)))

resultC = np.array(resultC)
print(resultC)


def overlap_save(kernel, input, N=8):
    def conv_circ( signal, ker ):
        """
        signal: real 1D array
        ker: real 1D array
        signal and ker must have same shape
        """
        return np.real(np.fft.ifft( np.fft.fft(signal) * np.fft.fft(ker) ))

    print("convolving using overlap-save")
    M = len(kernel)
    L = N - (M - 1)
    h = kernel + [0] * (L - 1)
    print("M:", M)
    print("N:", N)
    print("L:", L)
    print("h:", h)
    
    blocks = [[]]
    for value in input:
        if len(blocks[-1]) < L:
            blocks[-1].append(value)
        else:
            blocks.append([value])
            
    if len(blocks[-1]) != L:
        blocks = blocks[:-1]

    for i, b in enumerate(blocks):
        if i == 0:
            blocks[i] = ([0] * (M - 1)) + b
        else:
            blocks[i] = blocks[i - 1][-(M-1):] + b
            
    print("xs:", blocks)

    ys = []
    for x in blocks:
        ys.append(np.round(conv_circ(x, h)[M-1:]).astype(int).tolist())

    print("ys:", ys)

    return [value for block in ys for value in block]

    
def overlap_save_stream(kernel, input_blocks_stream, N=8):
    def conv_circ( signal, ker ):
        """
        signal: real 1D array
        ker: real 1D array
        signal and ker must have same shape
        """
        return np.real(np.fft.ifft( np.fft.fft(signal)*np.fft.fft(ker) ))

    M = len(kernel)
    L = N - (M - 1)
    h = kernel + [0] * (L - 1)
    last_block = None
    
    for block_unpadded in input_blocks_stream:
        assert len(block_unpadded) == L
        if last_block is None:
            block = ([0] * (M - 1)) + block_unpadded
        else:
            block = last_block[-(M-1):] + block_unpadded
        last_block = block

        yield np.round(conv_circ(block, h)[M-1:]).astype(int).tolist()

def chunk_stream(input_stream, chunk_size):
    block = []
    for value in input_stream:
        block.append(value)
        if len(block) == chunk_size:
            yield block
            block = []


if __name__ == "__main__":
    kernel = [1, 2, 3, 4]
    #input = [1,2,3,4,5,6,7,8,9,1,2,3,4,5,6,7,8,9,1,2,3,4,5,6,7]
    os1_result = overlap_save(kernel, input)
    print("overlap-save result:         ", os1_result)
    os2_result = overlap_save_stream(kernel, chunk_stream(input, 5))
    os2_result = [value for block in os2_result for value in block]
    print("overlap-save (stream) result:", os2_result)
    np_result = np.convolve(kernel, input)[:len(input)]
    print("numpy result:                ", np_result.tolist())
    assert np.array_equal(np_result, os1_result) 
    assert np.array_equal(np_result, os2_result) 

