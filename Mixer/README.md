# Mixer

A simple 5-channel mixer for audio or CV.

![mixer faceplate](images/mixer_faceplate.svg)

## Manual

The top 4 channels are passed to the mixer via a mute switch and an attenuator knob. The 5th channel has a mute switch but no volume knob.

The volume knobs go from basically 0v (totally off) to nearly 100% volume (this mixer cannot provide any gain boost). In reality, the inputs are not buffered before they are attenuated so the volume may depend very slightly on the output impedance of the modules feeding into the mixer. Also, some potentiometers can't actually go all the way to zero so if you want the sound to be completely muted just use the switch, which will totally disconnect the signal.

The output *is* buffered, so this is an active (not passive) mixer.

Based on the [1163 Mini Mixer](https://www.lookmumnocomputer.com/projects#/1163-mini-mixer) by Look Mum No Computer.
