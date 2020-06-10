import numpy as np
import numpy.fft as fft
from collections import UserList
from tqdm import tqdm
import sys

np.set_printoptions(suppress=True)

class ZeroPaddedBuffer(UserList):
    """
    A standard Python list except that if you access any index beyond
    the bounds of the list, 0 will be returned instead of an exception.
    """
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
    """
    A standard Python list except that if you access any negative index,
    you will get 0.
    """
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
    """
    An object to handle convolution on a stream at a given offset.
    The convolution is done in a normal, straightforward way. if an
    offset is set, the output is buffered by that much so that
    getting element 0 of the output buffer always gets the value
    at the "current time".

    Args:
        kernel (list): a FIR to convolve with the input buffer
        input_buffer (list): the input data or signal to colvoleve. This is
            treated like a list by the Convolver (i.e. it is indexed at
            ceretain values) but all the data does not need to be there at
            the start of convolution. The buffer can start empy as long as
            you append at least one value to it before each time you call
            `update()`. In practice, you should probably use a
            ZeroPrepaddedBuffer for this purpose.
        offset (int): an offset into the input buffer. A Convolver with
            offset N will have its output be "delayed" by N values relative
            to a Convolver with offset 0.
    """
    def __init__(self, kernel, input_buffer, offset):
        self.kernel = kernel
        self.input_buffer = input_buffer
        self.input_buffer_ptr = 0
        self.output_buffer = np.zeros(1 + offset)

    def update(self):
        """
        Causes the convolver to read one new value from the input stream
        and compute one new output which is shifted into the output buffer.
        """
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
    """
    Same as NaiveConvolver except that the input stream is buffered and the
    convolution is done in chunks instead of one value at a time. There is
    no practical reason to do this. It is just a conceptual lead-in to
    implementing DFTBufferedConvolver which takes advantage of this chunking
    to get much faster performance.
    """
    def __init__(self, kernel, input_buffer, offset, block_size):
        assert block_size <= offset
        self.kernel = kernel
        self.input_buffer = input_buffer
        self.block_size = block_size
        self.block_start_ptr = 0
        self.input_length = 0
        self.output_buffer = np.zeros(offset)

    def _convolve_block(self):
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
        """
        Reads one new value from the input buffer. If it does not have a full
        block to process yet, nothing will happen -- the inputvalue will just
        be buffered. Once `block_size` values will be buffered, the
        convolution with that whole block will be computed and shifted into
        the output buffer.
        """
        self.input_length += 1
        self.output_buffer = np.roll(self.output_buffer, -1)
        self.output_buffer[-1] = 0
        if self.input_length - self.block_start_ptr > self.block_size:
            result = self._convolve_block()
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

        if False:
            print("Creating DFT convolver")
            print("  offset:", offset)
            print("  N:", self.N)
            print("  M:", self.M)
            print("  L:", self.L) 
            print("  kernel:", len(kernel))
            print("  h:", len(self.h))
        
        assert self.L <= offset
        assert self.M <= self.N

        self.block_start_ptr = 0
        self.input_length = 0
        self.output_buffer = np.zeros(offset)

        self.last_block = None

    @staticmethod
    def circ_conv(signal, kernel):
        return np.real(np.fft.ifft( np.fft.fft(signal) * np.fft.fft(kernel)  ))

    def _convolve_block(self):
        block = np.array(self.input_buffer[self.block_start_ptr-1:self.block_start_ptr+self.L-1])
        if self.last_block is None:
            padded_block = ([0] * (self.M - 1)) + block.tolist()
        else:
            last_block_index = -(self.M-1)
            if last_block_index == 0:
                padded_block = block.tolist()
            else:
                padded_block = self.last_block.tolist()[last_block_index:] + block.tolist()
        self.last_block = np.array(padded_block)
        return DFTBufferedConvolver.circ_conv(padded_block, self.h)[self.M-1:]

    def update(self):
        self.input_length += 1
        self.output_buffer = np.roll(self.output_buffer, -1)
        self.output_buffer[-1] = -1
        if self.input_length - self.block_start_ptr >= self.L:
            result = self._convolve_block()
            self.block_start_ptr += self.L
            self.output_buffer[-self.L:] = result
        #print("time t =", self.input_length, "| output:", self.output_buffer)

    def get_current_value(self):
        return self.output_buffer[0]


class CompositeConvolver:
    """
    Given an FIR (kernel), this class makes a convolver out of multiple
    staggered DFT and naive convolvers in order to perform zero-point delay
    convolution
    Args:
        input (stream): input stream of data to convolve with kernel (i.e.
            the signal)
        kernel (list): array to convolve with the input (i.e. the FIR)
        SBS (int, optional): Starting block size. This is the size of the
            smallest block used for DFT convolution. Only effects speed.
            Should probably be 32 or 64.
    """
    def __init__(self, input, kernel, SBS=32, debug=False):
        self.input = input
        self.kernel = kernel
        self.data = ZeroPrepaddedBuffer()

        if debug:
            print("len(kernel)", len(kernel))
        
        # How many SBS-sized chunks do we need to cover the kernel?
        kernel_size = len(kernel)
        
        # The first block will be convolved plain colvolution
        self.convolvers = [NaiveConvolver(kernel[:SBS], self.data, 0)]
        kernel_size -= SBS

        # Allocate each convolver with size 2**n starts at offset 2**n
        n = 0
        while kernel_size > 0:
            length = 2**n * SBS
            offset = length
            self.convolvers.append(
                DFTBufferedConvolver(kernel[offset:offset+length], self.data, offset=offset, N=length))
            n += 1
            kernel_size -= length

        if debug:
            print("convovers:")
            for c in self.convolvers:
                if isinstance(c, NaiveConvolver):
                    print("    Naive")
                elif isinstance(c, DFTBufferedConvolver):
                    print("    DFT: N =", c.N)

    def __next__(self):
        """
        When an iterator requests the next value, read a value in from
        the input and then pass it to every convolver.
        """
        input_value = next(self.input)
        # `data` is the shared input buffer for all convolvers
        self.data.append(input_value)
        sum = 0
        for conv in self.convolvers:
            conv.update()
            sum += conv.get_current_value()
        return sum

    def __iter__(self):
        return self


def do_tests():
    #
    # Basic tests
    #
    
    print("Testing basic stacked convolution...")
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

    #
    # DFT tests
    #
    
    print("Testing Gardner stacked DFT convolution...")
    SBS = 32  # or 64 -- starting block size, aka N in G.95
    for test in tqdm(range(100)):
        seed = np.random.randint(9999)
        np.random.seed(seed)
        
        kernel_length = np.random.randint(low=1, high=500)
        data_length = np.random.randint(low=1, high=1000)
        kernel = np.random.randint(low=0, high=100, size=kernel_length, dtype=np.int)
        data = np.random.randint(low=0, high=100, size=data_length, dtype=np.int)

        try:
            dft_result = []
            convolver = CompositeConvolver(iter(data), kernel)
            for value in convolver:
                dft_result.append(value)
                
        except Exception as e:
            print("test faild for seed", seed)
            print(f"  Error on line {sys.exc_info()[-1].tb_lineno}: {e}")
            sys.exit(1)
        
        naive_result = []
        prepadded_data = ZeroPrepaddedBuffer(data)
        for i in range(1, len(prepadded_data) + 1):
            naive_result.append(np.dot(prepadded_data[i-len(kernel):i], np.flip(kernel)))

        numpy_result = np.convolve(data, kernel, mode="full")[:-(len(kernel)-1)]

        assert np.allclose(naive_result, numpy_result)
        if not np.allclose(naive_result, dft_result):
            print("test faild for seed", seed)
            print("kernel == ", kernel)
            print("dft_result:")
            print(dft_result)
            print("correct:")
            print(naive_result)
            sys.exit(1)

"""
kernel = np.array([-3, -2, 5, 0, 1, 2, 3, 4])
#input = [1,2,3,4,5,6,7,8,9,1,2,3,4,5,6,7,8,9,1,2,3,4,5,6,7]
input = np.arange(25) + 1

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
"""

def overlap_save(kernel, input, N=8, debug=False):
    def conv_circ( signal, ker ):
        """
        signal: real 1D array
        ker: real 1D array
        signal and ker must have same shape
        """
        return np.real(np.fft.ifft( np.fft.fft(signal) * np.fft.fft(ker) ))

    M = len(kernel)
    L = N - (M - 1)
    h = kernel + [0] * (L - 1)
    
    if debug:
        print("convolving using overlap-save")
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
            
    if debug: print("xs:", blocks)

    ys = []
    for x in blocks:
        ys.append(np.round(conv_circ(x, h)[M-1:]).astype(int).tolist())

    if debug: print("ys:", ys)

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
    input = [1,2,3,4,5,6,7,8,9,1,2,3,4,5,6,7,8,9,1,2,3,4,5,6,7]
    os1_result = overlap_save(kernel, input)
    #print("overlap-save result:         ", os1_result)
    os2_result = overlap_save_stream(kernel, chunk_stream(input, 5))
    os2_result = [value for block in os2_result for value in block]
    #print("overlap-save (stream) result:", os2_result)
    np_result = np.convolve(kernel, input)[:len(input)]
    #print("numpy result:                ", np_result.tolist())
    assert np.array_equal(np_result, os1_result) 
    assert np.array_equal(np_result, os2_result)


if __name__ == "__main__":
    do_tests()
