#ifndef config_h_INCLUDED
#define config_h_INCLUDED

/*
 * Configuration
 */

typedef double float_t;

// How long between when the CV & pots are sampled. A lower value means 
// more responsive controls but possibly a less smooth output.
const uint16_t CV_SAMPLING_FREQUENCY = 5;

// The voltage coming through the input transistors when the base is grounded
// on a scale of 0 = 0v, ANALOG_READ_MAX_VALUE = 5v
const uint16_t TRANSISTOR_ZERO_VALUE = 110;
// The voltage coming through the input transistors when the base connected to 5v
const uint16_t TRANSISTOR_5V_VALUE = 1017;

// The maximum value of the Arduino's internal DAC
// see: https://www.arduino.cc/reference/en/language/functions/analog-io/analogread/
const uint16_t ANALOG_READ_MAX_VALUE = 1023;

#define FIRMWARE_MODE_SIMPLEX 0
#define FIRMWARE_MODE_LFO 1
#define FIRMWARE_MODE FIRMWARE_MODE_SIMPLEX 

// A0 = top pot
// A1 = middle pot
// A2 = bottom pot

//Pins
const uint16_t CHIP_SELECT_PIN = 8;
const uint16_t POT_PIN_1 = A0; // SPEED
const uint16_t POT_PIN_2 = A1; // TEXTURE
const uint16_t POT_PIN_3 = A2; // ATTEN
const uint16_t CV_PIN_1 = A4;  // SPEED
const uint16_t CV_PIN_2 = A3;  // TEXTURE

#if FIRMWARE_MODE == FIRMWARE_MODE_SIMPLEX
/**
 * Configuration for Simplex mode
 */

// The range of the speed or "frequency" of the noise.
// Measured in terms of steps per microseccond.
// 0.000001 (1e-6) is something like 1Hz.
const float_t MAX_SPEED = 0.000008;
const float_t MIN_SPEED = 0;

// The range of the texture. Texture is just the coefficient applied to each
// successive octave. So a texture of 1 would stack all octaves with the same
// weight. A texture of 0 would only play the root octave.
const float_t MIN_TEXTURE = 0;
const float_t MAX_TEXTURE = 0.8;

// The number of "octaves" in the noise algorithm -- how many layers of perlin
// noise are stacked to get the result.
const uint16_t NUM_OCTAVES = 4; 
// The distance between the octaves. I.e. the number of times faster octive 2 is
// than octave 1.
const uint16_t OCTAVE_STEP = 4;

// The min, max, and exponential rate for each of the 5 input channels
// (3 pots + 2 CVs)
const float_t ANALOG_READ_RANGES[5][3] = {
    {MIN_SPEED, MAX_SPEED, 8},
    {MIN_TEXTURE, MAX_TEXTURE, 0},
    {0, 1, 0},
    {MIN_SPEED, MAX_SPEED, 0},
    {MIN_TEXTURE, MAX_TEXTURE, 0},
};

#else

/**
 * Configuration for LFO mode
 */

// Frequency range for both oscilators
const float_t MIN_HERTZ = 0.01;
const float_t MAX_HERTZ = 5;

// The behavior of the third knob can be set to either control the amplitude or the
// waveform of the first LFO
#define AUX_MODE_APLITUDE 0
#define AUX_MODE_WAVEFORM 1
#define AUX_MODE_SKEW 2
#define AUX_MODE AUX_MODE_APLITUDE

#if AUX_MODE == AUX_MODE_WAVEFORM
const uint8_t WAVEFORM_SAW = 0;
const uint8_t WAVEFORM_INV_SAW = 1;
const uint8_t WAVEFORM_TRI = 2;
const uint8_t WAVEFORM_SIN = 3;
const uint8_t WAVEFORM_SQUARE = 4;
const uint8_t WAVEFORM_BOUNCE = 5;

const uint8_t NUM_WAVEFORMS = 6;
const uint8_t WAVEFORMS[NUM_WAVEFORMS] = {
    WAVEFORM_SAW,
    WAVEFORM_INV_SAW,
    WAVEFORM_TRI,
    WAVEFORM_SIN,
    WAVEFORM_SQUARE,
    WAVEFORM_BOUNCE,
};

#endif

// The min, max, and exponential rate for each of the 5 input channels
// (3 pots + 2 CVs)
const float_t ANALOG_READ_RANGES[5][3] = {
    {MIN_HERTZ, MAX_HERTZ, 0},
    {MIN_HERTZ, MAX_HERTZ, 0},
    {0, 1, 0},
    {MIN_HERTZ, MAX_HERTZ, 0},
    {MIN_HERTZ, MAX_HERTZ, 0},
};

#endif

/*
 * End config
 */

#endif // config_h_INCLUDED

