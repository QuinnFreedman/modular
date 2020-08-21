# Simplex

Simplex generates two channels of smooth random voltage based on the simplex noise* algorithm. Simplex noise produces values that are alway in motion but never change abruptly and never repeat. It's great for adding some subtle, organic parameter moodulation to your patch or for generating random melodies and arpeggios.

## Manual

```
+-------------+
| o Simplex o |
|             |
|             |
|     ___     |
|    /  /\    |
| X |  *  |   | <-- Potentiometers
|    \___/    |
|             |
|     ___     |
|    /\  \    |
| Y |  *  |   |
|    \___/    |
|             |
|     ___     |
|    /  /\    |
| Z |  *  |   |
|    \___/    |
|             |
|             |
|             |
|   _     _   |
|y (_) z (_)  | <-- Jack sockets
|             |
|             |
|  (*)   (*)  | <-- LEDs
|   _     _   |
|A (_) B (_)  | <-- Jack sockets
|             |
|             |
|             |
| o         o |
+-------------+
```

### Interface

* **X: Amplitude**. Attenuates the range of output channel A. (Channel B is fixed).
* **Y: Speed**: Controls how quickly both channels move through time. This is like the frequency on an LFO except that the output will never repeat. A higher speed value means that the noise will varry more sharply.
* **Z: Texture**: Controls how "smooth" or "rough" the noise is for both channels. Texture works by adding more layers of noise with higher frequency and lower amplitude -- essentailly like overtones or harmonics in audio. If you just want "basic" smooth noise, put texture all the way to the left. If you want to have some large fluctuations over a very slow time scale while still having some quick organic jitter on top, move it towards the right. Here is a graph of output values with texture low (left) and high (right).

|Simplex noise with texture off | Simplex noise with texture high |
|-------------------------------|---------------------------------|
|![Graph](./images/simplex_smooth.jpg) | ![Graph](./images/simplex_textured.jpg)

### Inputs

* **y: Speed CV**: CV control for speed (both channels). 0-5v. Input is added to the speed potentiomeer value, capped at 5v.
* **z: Texture CV**: CV control for texture (both channels). 0-5v. Input is added to the texture potentiomeer value, capped at 5v.

### Outputs

Both channels will have the same speed and texture, but will be completely separate from each other.

* **A**: Channel A. -10v - +10v, attenuated by **Amplitude (X)**
* **B**: Channel B. 0-5v

## Assembly

See [general assembly instructions](https://github.com/QuinnFreedman/modular/wiki/Assembly).

### Components

See [components page](https://github.com/QuinnFreedman/modular/wiki/Components) for more info.

* Resistors
  * 9x 1kohm
  * 2x 1Mohm
  * 2x 220ohm
* 4 Jacks
* 2 LEDs
* 2 PNP transistors
* 2 100nf fixed capacitors (very optional)
* 1 Arduino Nano
* 1 MCP4922
* 1 TL072

### Instructions

The capacitors are intended to remove noise from the power supply. But, with my power supply, I was not able to even measure a difference. You can just leave them out if you want, or put in multiple capacitors of different values in parallel.

The long jumper wire in the PCB design is to connect the signal to the left LED. If you want the 2 LEDs to be identical, connect it with a wire as shown below. If you want to use a bipolar LED for the left channel (so it will be a different color when the output is negative), instead connect the bottom jumper wire terminal with the top of the 1k resistor right next to it. You will probably also want to replace the adjacent 220k resistor with something larger (like 880 or 1k).

![PCB](images/Simplex_PCB_middle_pcb.jpg)

## Requirements

Arduino IDE 1.8.10 

|Library                | Author   | Version |
|-----------------------|----------|---------|
| [SPI][1]              | Arduino  |         |

[1]: https://www.arduino.cc/en/reference/SPI
