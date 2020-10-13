# RNG

RNG stands for Random Number Generator. The ability to generate random control voltages is essential to creating generative music. However, purely random values can be noisy and chaotic. RNG lets you blend between looping old values and mutating them into new, random ones in a probabalistic way. It is inspired by Music Thing Modular's [Turing Machine](https://musicthing.co.uk/pages/turing.html) but with a lot of added functionality. In addition to outputting stepped random and/or looping values, RNG lets you blend between two channels of triggers/gates based on those values.

### Example uses

* Random melodies: Set Chaos to a low value and set the length to a small power of 2. Then, patch the output into a quantizer and use it as a pitch CV to generate looping but randomly generating melodies, synced with a clock.
* Drum multiplexer: Plug a drum trigger into Clock and patch A and B to trigger two different drum sounds. Then, when a drum trigger comes in it will be randomly sent to one channel or the other. This can be totally random or looping.
* Probabilistic sequencer: same as above but leave one output unconnected. This allows a sequencer without built in probability functionality to have some randomness.
* Random clock divider: Patch a steady clock into **Clock** and patch **B** out as the new clock with a high bias. This way, the clock pulses will occasionally not register, allowing two sequences to fall out of phase for syncopated polyrythms.
* Hook multiple RNGs together. Use the probabilistic clock from one of them to advance a random CV sequence from the other.

## Manual

```
+-----------------------------+
| o           RNG           o |
|                             |
| (*) (*) (*) (*) (*) (*) (*) | <-- LEDs
|      ____                   | 
|     /   /\         ___      |
|    /   /  \       /  /\     |
|    \      /      |  *  |    | <-- Potentiometers
|     \____/        \___/     |
|      Chaos        Length    |
|                             |
|      ___           ___      |
|     /\  \    |    /  /\     |
|    |  *  |   |   |  *  |    |
|     \___/    |    \___/     |
|     Range    |     Bias     |
|              |              |
|              |              |
|      \       |     \        |
|    [  *  ]   |   [  *  ]    | <- Switches
|  Bi/unipolar | Trigger/Gate |
|              |              |
|    _     _   |      _       |
|   (_)   (_)  |     (_)      | <-- Jack sockets
|  Clock  CV   |     Bias     |
|       _      | (*)     (*)  |
|      (_)     |  (_)   (_)   |
|      OUT     |   A     B    |
| o                         o |
+-----------------------------+
```

### Interface

* **LEDs**. A row of LEDs across the top show the current sequence being played. Brighter LEDs correspond to higher values. The middle LED is the value currently being played.
* **Chaos**: Controls how much randomness in injected into the sequence. At zero (all the way left) the sequence is locked in a loop and no randomness is added. All the way to the right, each value played is completely random. In the middle, values will be played from the saved loop but with a chance of mutating each time they are played.
* **Length**: A rotary encoder that allows you to control the length of the sequence. When you turn the knob, the length of the sequence will be displayed in binary on the top row of LEDs. Hold the knob down to quickly skip between powers of twos. If set to a negative number (indicated with a HIGH value on the far left bit) then the sequence will "yoyo," playing alternatingly forward and backward.
* Left Side: stepped voltage mode. This half of the module outputs a stepped random voltage 
  * **Range**: The range/spread of the random voltages output.
  * **Bi/unipolar**: Whether the values are unipolar (0 to +10v) or bipolar (-10v to +10v)
  * **Clock** (input): Whenever HIGH, the values are advanced by one, a new value from the loop is played (and maybe mutated).
  * **CV** (input): Control voltage to control the Chaos parameter. There are two configurable modes. It can either act as a direct CV (i.e. higher values = more randomness) or as a lock gate so it has no effect when LOW and then locks the sequence in place and prevents any mutation when HIGH.
* Right Side: Binary choice mode. This side of the module randomly chooses between two output trigger/gates based on the value of the left side.
  * **Bias**: The probability of outputting to channel A vs B.
  * **Trigger/Gate**: Chooses whether the A/B outputs should be triggers (a short voltage spike with every new clock pulse) or gates (remain steadily high until the output channel is changed).
  * **Bias** (input): CV control for the Bias knob
  * **A** & **B** (output): Two channels of gate/trigger output. A corresponding LED lights up when each gate is open.

## Assembly

See [general assembly instructions](https://github.com/QuinnFreedman/modular/wiki/Assembly).

### Components

See [components page](https://github.com/QuinnFreedman/modular/wiki/Components) for more info.

* Resistors
  * 8x 1kohm
  * 1x 2khm
  * 2x 10khm
  * 3x 1Mohm
  * 9x 220ohm (or any value -- to control LED brightness)
* 6 Jacks
* 9 LEDs (7x5mm and 2x3mm -- or just drill the faceplate holes to fit what you have)
* 3 PNP transistors
* 2 100nf fixed capacitors (optional)
* 1 Arduino Nano
* 1 MCP4922
* 1 TL072
* 1 TLC5940NT

## Requirements

Arduino IDE 1.8.10 

|Library                | Author          | Version |
|-----------------------|-----------------|---------|
| [SPI][1]              | Arduino         |         |
| [TLC5940][2]          | Paul Stoffregen | 1.1.1   |

[1]: https://www.arduino.cc/en/reference/SPI
[2]: https://github.com/PaulStoffregen/Tlc5940 