import numpy as np
from numpy.random import normal, random

MAX_DENSITY = 1 / 4
MIN_DENSITY = 4
ONE_DENSITY = (1 - MIN_DENSITY) / (MAX_DENSITY - MIN_DENSITY)
MAX_SPREAD = 1 / 3 # Maximum spread as fraction of grain size

def auto_str(cls):
    def __str__(self):
        return '%s(%s)' % (
            type(self).__name__,
            ', '.join('%s=%s' % item for item in vars(self).items())
        )
    cls.__str__ = __str__
    return cls

@auto_str
class Ringbuffer:
    def __init__(self, length, dtype=int):
        self.length = length
        self.buffer = np.zeros(length, dtype=dtype)
        self.cursor = 0


    def _wrap_index(self, i):
        while i < 0 or i >= self.length:
            if i < 0:
                i = abs(i)
            if i >= self.length:
                i = 2 * (self.length - 1) - i
        return i
        

    def __getitem__(self, i):
        #if i < 0 or i >= self.length:
        #    raise IndexError(f"index {i} out of range for buffer of length {self.length}")

        i = self._wrap_index(i)
        return self.buffer[(self.cursor + i) % self.length]

    def __setitem__(self, i, x):
        #if i < 0 or i >= self.length:
        #    raise IndexError(f"index {i} out of range for buffer of length {self.length}")

        i = self._wrap_index(i)
                
        self.buffer[(self.cursor + i) % self.length] = x

    def __len__(self):
        return self.length

    def __str__(self):
        return "[" + ", ".  join(str(self[i]) for i in range(len(self))) + "]"

    def push_back(self, x):
        self[0] = x
        self.cursor = (self.cursor + 1) % self.length


@auto_str
class GranularSynthParams:
    def __init__(self, params):
        self.position = params.get("position", 0)
        self.size = params.get("size", 10)
        self.density = params.get("density", 0)
        self.spread = params.get("spread", 0)
        self.texture = params.get("texture", 0)
        self.pitch = params.get("pitch", 1)
        self.feedback = params.get("feedback", 0)
        self.blend = params.get("blend", 0)
        self.freeze = params.get("freeze", 0)
        self.buffer_size = params.get("buffer_size", 500)

@auto_str
class Grain():
    def __init__(self, input_time, input_length, output_time, out_length, debug_index=0):
        self.input_time = input_time
        self.input_length = input_length
        self.output_time = output_time
        self.out_length = out_length
        self.debug_index = debug_index

def split_list(items, predicate):
    a, b = [], []
    for x in items:
        if predicate(x):
            a.append(x)
        else:
            b.append(x)
    return a, b

class GranularSynth:
    def __init__(self, params: GranularSynthParams, input):
        self.input_stream = input
        self.params = params
        self.buffer = Ringbuffer(params.buffer_size)
        # self.buffer = np.zeros(params.buffer_size, dtype=int)
        self.time = 0
        self.last_grain_made_at = 0
        self.playing_grains = []

        self.debug_index = 0
        self.debug_input = ""
        self.debug_output_data = ["" for _ in range(10)]
        self.debug_envelopes = ["" for _ in range(10)]
        self.debug_output = ""


    def get_envelope_at(self, x):
        def linear_env(x, k):
            """
            Blends b/t square box envelope and triangle envelope
            by modulating attack/decay equal to k
            """
            assert x >= 0 and x <= 1
            assert k >= 0 and k <= 1/2
            if k == 0:
                return 1.0
                
            if x < k:
                return x / k
            if x < 1 - k:
                return 1.0

            return (1 - x) / k

        def hann(x):
            """
            Hann function
            """
            assert x >= 0 and x <= 1
            return np.sin(np.pi * x) ** 2
                

        b = self.params.texture
        size = self.params.size

        assert b >= 0 and b <= 1
        assert x >= 0 and x <= 1

        if b < 0.5:
            return linear_env(x, b)
        else:
            blend = 2 - 2*b
            return blend * linear_env(x, 1/2) + \
                   (1 - blend) * hann(x)


    def make_grain(self):
        assert self.params.spread >= 0 and self.params.spread <= 1
        
        input_offset = int(round(normal(0, self.params.spread * MAX_SPREAD * self.params.size)))
        self.last_grain_made_at = self.time
        self.playing_grains.append(Grain(
            input_time = self.time + input_offset,
            input_length = self.params.size,
            output_time = self.time,
            out_length = self.params.size * self.params.pitch,
            debug_index = self.debug_index
        ))
        self.debug_index = (self.debug_index + 1) % 10


    def __next__(self):
        input_sample = next(self.input_stream)
        
        # self.buffer = np.roll(self.buffer, -1, axis=0)
        # self.buffer[-1] = input_sample
        
        self.buffer.push_back(input_sample)

        self.debug_input += f"{input_sample:3} "

        position = self.params.position # TODO times buffer length

        ##
        ## 1. Make new grains
        ##

        is_randomly_sewn = self.params.density < 0
        assert abs(self.params.density) <= 1
        grain_spacing = (abs(self.params.density) * MAX_DENSITY +
                         (1 - abs(self.params.density)) * MIN_DENSITY) * self.params.size * self.params.pitch
        
        if is_randomly_sewn:
            if int(random() * grain_spacing) == 0:
                self.make_grain()
        else:
            if self.time - self.last_grain_made_at >= grain_spacing:
                self.make_grain()

        ##
        ## 2. clean up finished grains
        ##
        
        i = 0
        while i < len(self.playing_grains):
            grain = self.playing_grains[i]
            grain_offset = self.time - grain.output_time
            if grain_offset >= grain.out_length:
                self.playing_grains.pop(i)
            else:
                i += 1

        ##
        ## 3. Play grains
        ##
        
        output = 0

        for i in range(len(self.debug_output_data)):
            self.debug_output_data[i] += "  . "
            self.debug_envelopes[i] += "  . "
            
        for grain in self.playing_grains:
            time_since_started_playing = self.time - grain.output_time
            time_since_started_reading = self.time - grain.input_time
            grain_start_index = len(self.buffer) - 1 - position - time_since_started_reading
            fraction_played = time_since_started_playing / grain.out_length
            index = grain_start_index + int(grain.input_length * fraction_played)
            data = self.buffer[index]
            envelope = self.get_envelope_at(fraction_played + 1 / (2 * grain.out_length))
            self.debug_output_data[grain.debug_index] = \
                    self.debug_output_data[grain.debug_index][:-4] + f"{data:3} "
            self.debug_envelopes[grain.debug_index] = \
                    self.debug_envelopes[grain.debug_index][:-4] + f"{envelope:.1f} "
            output += envelope * data

        if self.playing_grains:
            output /= len(self.playing_grains)
        
        self.debug_output += f"{int(round(output)):3} "

        self.time += 1
        return output

    def __iter__(self):
        return self


if __name__ == "__main__":
    params = GranularSynthParams({
        "position": 0,
        "density": ONE_DENSITY,
        "texture": 0.5,
        "buffer_size": 200,
        "pitch": .5,
        "spread": .5
    })

    input = iter(np.arange(1000))

    synth = GranularSynth(params, input)
    list(synth)

    print(synth.debug_input)
    print("-" * len(synth.debug_output_data[0]))
    for line in synth.debug_output_data:
        print(line)
    print("-" * len(synth.debug_output_data[0]))
    for line in synth.debug_envelopes:
        print(line)
    print("-" * len(synth.debug_output_data[0]))
    print(synth.debug_output)
        
