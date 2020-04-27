import numpy as np
from numpy.random import normal

MAX_ATTACK = 50

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

    def __getitem__(self, i):
        return self.buffer[(self.cursor + i) % self.length]

    def __setitem__(self, i, x):
        self.buffer[(self.cursor + i) % self.length] = x

    def __len__(self):
        return self.length

    def __str__(self):
        return "[" + ", ".  join(str(self[i]) for i in range(len(self))) + "]"

    def push_back(self, x  ):
        self[0] = x
        self.cursor = (self.cursor + 1) % self.length


@auto_str
class GranularSynthParams:
    def __init__(self, params):
        self.position = params.get("position", 0)
        self.size = params.get("size", 10)
        self.density = params.get("density", 0)
        self.randomness = params.get("randomness", 0)
        self.texture = params.get("texture", 0)
        self.pitch = params.get("pitch", 0)
        self.feedback = params.get("feedback", 0)
        self.blend = params.get("blend", 0)
        self.freeze = params.get("freeze", 0)
        self.buffer_size = params.get("buffer_size", 500)

@auto_str
class Grain():
    def __init__(self, input_time, input_length, output_time, out_length):
        self.input_time = input_time
        self.input_length = input_length
        self.output_time = output_time
        self.out_length = out_length

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
        self.time = 0
        self.playing_grains = []
        self.future_grains = []
        for i in range(5):
            self.make_grain(round(i * self.get_grain_offset()))

        # self.debug_input = []
        self.debug_output_data = ["" for _ in range(10)]
        self.debug_envelopes = ["" for _ in range(10)]

        print([str(x) for x in self.future_grains])

    def get_envelope_at(self, offset):
        size = self.params.size
        attack = self.params.texture * MAX_ATTACK
        if attack == 0: return 1.0
        slope = 1.0 / attack
        if offset < attack and offset < size / 2:
            return offset * slope
        elif offset > size - attack and offset > size / 2:
            return 1 - (offset - size + attack) * slope
        else:
            return 1.0

    def get_grain_offset(self):
        x = self.params.density
        # y = -np.log2(2 * x) / 2 + 1
        if x < 0.5:
            y = 3 - 4 * x
        else:
            y = (4 - 2 * x) / 3
        return self.params.size * y

    def make_grain(self, input_time):
        self.future_grains.append(Grain(
            input_time = input_time,
            input_length = self.params.size,
            output_time = round(normal(input_time, self.params.randomness * self.params.size)),
            out_length = self.params.size * self.params.pitch
        ))
        print(self.future_grains[-1].output_time)

    def __next__(self):
        self.buffer.push_back(next(self.input_stream))

        position = self.params.position # times buffer length
        
        output = 0

        self.future_grains, grains_to_play = split_list(
            self.future_grains,
            lambda grain: self.time - grain.output_time >= 0
        )

        self.playing_grains += grains_to_play

        for i in range(len(self.debug_output_data)):
            self.debug_output_data[i] += "  . "
            self.debug_envelopes[i] += "  . "
            
        for i, grain in enumerate(self.playing_grains):
            # how far into playing the grain we should be
            grain_offset = self.time - grain.output_time
            data = self.buffer[grain.input_time - self.time - position]
            envelope = self.get_envelope_at(grain_offset)
            self.debug_output_data[i] = \
                    self.debug_output_data[i][:-4] + f"{data:3} "
            self.debug_envelopes[i] = \
                    self.debug_envelopes[i][:-4] + f"{envelope:.3} "
            output += envelope * data

        # clean up finished grains
        i = 0
        while i < len(self.playing_grains):
            grain = self.playing_grains[i]
            grain_offset = self.time - grain.output_time
            if grain_offset >= grain.out_length:
                self.playing_grains.pop(i)
            else:
                i += 1
                

        self.time += 1
        return output

    def __iter__(self):
        return self


if __name__ == "__main__":
    params = GranularSynthParams({
        "position": 500,
        "density": 1
    })

    input = iter(np.arange(1000))

    synth = GranularSynth(params, input)
    list(synth)

    for line in synth.debug_output_data:
        print(line)
