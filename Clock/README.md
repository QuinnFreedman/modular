# Clock

A clock module sends rythmic voltage pulses at different subdevisions or multiples of a given tempo. If you want to have any kind a syncronous tempo to your music, use a clock to drive all your other modules and keep them in sync. You can use a clock output to step a sequencer or plug it directly into gate and trigger inputs.

## Manual

> TODO

PW: Pulse Width
PS: Phase Shift
SW: Swing

## Assembly

See [general assembly instructions](https://github.com/QuinnFreedman/modular/wiki/Assembly).

### Components

See [components page](https://github.com/QuinnFreedman/modular/wiki/Components) for more info.

* 16 Resistors
* 8 Jacks
* 8 LEDs
* 1 I2C 128x64 OLED display
* 1 Rotary encoder

### Resistor values

The resistors in this module can be basically any value you want. The vertical resitors (the ones that connect directly to the LEDs) control the brightness of the LEDs. They can be as low as 220 Ohms for a very bright LED to 10k for a very dim one. The best value might depend on the LEDs you have. I used a value of 470Ohm in my prototype which was pretty retina melting with the 3mm LEDs I had but good for some generic 5mm LEDs I tried.

The horizontal LEDs control the output imedance. This just makes it so that if the output ever gets shorted to ground (like when plugging in a cable) the Arduino doesn't get fried. Again, I used 470Ohm in my prototype, which worked fine. A very low value would just waste power while a very high value might give a slightly less precise output for some modules. I would probably recommend a slightly higher value, maybe 1k.

## Extra features

> TODO

## Requirements

Arduino IDE 1.8.10 

|Library                | Author   | Version |
|-----------------------|----------|---------|
| [Adafruit SSD1306][1] | Adafruit | 2.2.0   |
| [Adafruit GFX][2]     | Adafruit | 1.7.5   |

[1]: https://github.com/adafruit/Adafruit_SSD1306
[2]: https://github.com/adafruit/Adafruit-GFX-Library
