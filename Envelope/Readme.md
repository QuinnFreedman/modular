# Envelope

A simple but powerful envelope generator with 4 modes. Can be used as a linear ADSR or exponential AD, looping or not.

<img src="images/envelope_faceplate.svg" />


# Manual

### (A) Mode Select Button

This button lets you cycle between the four modes. While pressing the button, the LEDs will illuminate to indicate which mode is selected.

### (B) LEDs

While selecting modes, the LEDs will show the active mode:
1. **(yellow)** ADSR mode
2. **(blue)** AARR mode
3. **(red)** AARR Loop mode
4. **(green)** TRAP mode

Once a mode is selected, the LEDs will show which phase the envelope is in:
1. **(yellow)** Attack
2. **(blue)** Decay
3. **(red)** Sustain
4. **(green)** Relase

### (C) Knobs & CV input

There are 4 knobs, each with CV control from 0-5v. They control different parameters of the envelope. Their function changes based on the mode of operation. See the diagrams below for more. They are colored (1) Yellow, (2) Blue, (3) Red, (4) Green.


| Knob       | ADSR            | AARR            | AARR Loop       | TRAP Loop       |
| ---------- | --------------- | --------------- | --------------- | --------------- |
| (1) Yellow | Attack time     | Attack time     | Attack time     | Attack time     |
| (2) Blue   | Decay time      | Attack rate     | Attack rate     | Sustain *time*  |
| (3) Red    | Sustain *value* | Release time    | Release time    | Release time    |
| (4) Green  | Release time    | Release rate    | Release rate    | Delay time      |


### (D) Gate [input]

In a non-looping mode, the gate is the the main way you interact with the envelope. When the gate is HIGH, the envelope will start to open up and will stay open (high) until the gate is set LOW.

In a looping mode, by default, the gate will stop the envelope looping as long as it is HIGH. The firmware can also be configured to *only* loop while the gate is high.

### (E) Ping (aka retrigger) [input]

This is a trigger input. When it recieves a trigger, it will cause the envelope to open. If the gate is closed, sending a trigger will just cause the envelope to fully open and then close again. If the gate is on, sending a trigger to ping will keep the envelope open, but will "re-trigger" it. For example, if the envelope was in a Sustain phase, re-trigger will briefly set the envelope back to 100% and then it will decau back to the Sustain level.

### (F) Out [output]

The main output. Outputs analog values between 0-5v.

### (G) Inv [output]

Outputs an inverted version of the main out, but transposed to also be between 0-5v. I.e. INV will he HIGH when OUT is LOW and LOW when OUT is HIGH. This is useful for sidechaining and other ducking.

# Modes

## ADSR mode

<p align="center" width="100%">
    <img alt="ADSR graph" src="images/ADSR.svg" width=500 />
</p>

ADSR is a "standard" linear envelope mode with attack, decay, sustain, and release controlled by the 4 inputs, respectively.

## AARR mode

<p align="center" width="100%">
    <img alt="AARR graph" src="images/AARR.svg" width=500 />
</p>

AARR (or A,A',D,D') mode gives up control of the attack and decay. Instead, knobs (2) and (4) are used to control the exponential rate of the attack & decay. These controls can go through zero, so both attack and decay can be either exponential or logarithmic.

## AARR Loop mode

AARR Loop is exactly like AARR mode except that it loops continuously. It goes back and forth between Attack and Release (no sustain), so it is always rising or falling.

## TRAP Loop mode

<p align="center" width="100%">
    <img alt="TRAP graph" src="images/TRAP.svg" width=500 />
</p>

This mode could be called ASRD for Attack/Sustain/Release/Delay, but I call it TRAP for "trapezoid" to remove confusion. This mode is another looping mode where you can control how long the envelope stays HIGH and LOW between Attack/Decay in the loop. This lets you create a pulse-width modulated square wave or saw wave, or any other simple waveform.
