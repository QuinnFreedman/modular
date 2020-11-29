# Mixer

A simple 5-channel mixer for audio or CV.

![mixer faceplate](images/mixer_faceplate.svg)

## Manual

The top 4 channels are passed to the mixer via a mute switch and an attenuator knob. The 5th channel has a mute switch but no volume knob.

The volume knobs go from basically 0v (totally off) to nearly 100% volume (this mixer cannot provide any gain boost). In reality, the inputs are not buffered before they are attenuated so the volume may depend very slightly on the output impedance of the modules feeding into the mixer. Also, some potentiometers can't actually go all the way to zero so if you want the sound to be completely muted just use the switch, which will totally disconnect the signal.

The output *is* buffered, so this is an active (not passive) mixer.

Based on the [1163 Mini Mixer](https://www.lookmumnocomputer.com/projects#/1163-mini-mixer) by Look Mum No Computer.

### Daisy Chaining

Any number of mixer modules can be daisy chained together using the "chain in" and "chain out" connectors on the back. Make sure to keep the orentation (i.e. connect the top output to the top input and the bottom to the bottom). When chained together, the normalled output of one mixer is treated as a 6th input to the other. This means that when no patch cable is connected to the output of the first mixer, it's output is routed to the next mixer. When a jack is plugged into the output of the first mixer, it is removed from the seccond mix.

## Assembly 

### Components

See [components page](https://github.com/QuinnFreedman/modular/wiki/Components) for more info.

* Resistors
  * 9x 100kohm
  * 1x 1kohm
  * 1x 470ohm (controls LED brightness)
* 6 Jacks
* 5 SPST switches
* 1 LED (bi-directional if you want to be able to see negative outputs)
* 2 Potentiometers (B100k but any value is probably fine)
* 1 TL072
* 2 100nf capacitors (optional)
* 2 10uf capacitors (optional)

### Instructions

See [general assembly instructions](https://github.com/QuinnFreedman/modular/wiki/Assembly).

The capacitors are intended to remove noise from the power supply, but they're not that important. You can just leave them out if you want.

The 470ohm resistor controls the brightness of the LED. That will probably be quite bright. You can use a higher value if you want a less bright LED (maybe &ge;1k).

The LED should be 3mm. The voltage across the LED will be the same as the output voltage, so it will sometimes be negative. If you mostly plan on mixing audio, a normal LED will be fine because it will flash so quickly that it will look like it is measuring volume. But, if you plan on mixing CV values, a bi-directional LED might be better. Or, just leave it out if you don't want an LED at all.

If your switches only have two legs, connect the top two holes for each switch (marked with a line). If you don't want switches, you can just bridge those holes with a wire.
