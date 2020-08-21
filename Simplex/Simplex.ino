#include <stdint.h>
#include <SPI.h>

extern "C" {
    #include "perlin.h"
}

/*
 * Configuration
 */

// The range of the speed or "frequency" of the noise.
// Measured in terms of microsecconds per step.
// 0.000001 (1e-6) is something like 1Hz.
const double MAX_SPEED = 0.000008;//0.001;
const double MIN_SPEED = 0;

// The range of the texture. Texture is just the coefficient applied to each
// successive octave. So a texture of 1 would stack all octaves with the same
// weight. A texture of 0 would only play the root octave.
const double MIN_TEXTURE = 0;
const double MAX_TEXTURE = 0.8;

// The number of "octaves" in the noise algorithm -- how many layers of perlin
// noise are stacked to get the result.
const uint16_t NUM_OCTAVES = 4; 
// The distance between the octaves. I.e. the number of times faster octive 2 is
// than octave 1.
const uint16_t OCTAVE_STEP = 4;

// How long between when the CV, POTS are sampled. A lower value means 
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

//Pins
const uint16_t CHIP_SELECT_PIN = 8;
const uint16_t SPEED_POT_PIN = A0;
const uint16_t TEXTURE_POT_PIN = A1;
const uint16_t ATTENUATION_POT_PIN = A2;
const uint16_t SPEED_CV_PIN = A4;
const uint16_t TEXTURE_CV_PIN = A3;

/*
 * End config
 */

const uint8_t SEEDS[NUM_OCTAVES] = {
    random() * 256,
    random() * 256,
    random() * 256,
    random() * 256,
};

void setup() {
    pinMode(CHIP_SELECT_PIN, OUTPUT);
    pinMode(SPEED_POT_PIN, INPUT);
    pinMode(TEXTURE_POT_PIN, INPUT);
    pinMode(ATTENUATION_POT_PIN, INPUT);
    pinMode(SPEED_CV_PIN, INPUT);
    pinMode(TEXTURE_CV_PIN, INPUT);
    digitalWrite(CHIP_SELECT_PIN, HIGH);

    SPI.begin();
    SPI.setBitOrder(MSBFIRST);
    SPI.setDataMode(SPI_MODE0);

    Serial.begin(9600);
    
    for (int i = 0; i < 5; i++) {
        MCP4922_write(CHIP_SELECT_PIN, 0, 1);
        MCP4922_write(CHIP_SELECT_PIN, 1, 1);
        delay(300);
        MCP4922_write(CHIP_SELECT_PIN, 0, 0);
        MCP4922_write(CHIP_SELECT_PIN, 1, 1);
        delay(300);
    }
}

void loop() {
    static double texturePotValue = 0;
    static double speedPotValue = 0;
    static double textureCvValue = 0;
    static double speedCvValue = 0;
    static double texture = 0;
    static double speed = 0;
    static double amplitude = 0;

    static double perlinTime = 0;
    static uint32_t loopCount = 0;
    static uint32_t lastTime = micros();
    uint32_t now = micros();
    uint32_t dt = now - lastTime;
    lastTime = now;
    perlinTime += dt * speed;
    while (perlinTime > 256) {
        perlinTime -= 256;
    }

    for (int i = 0; i < 2; i++) {
        double noise = 0;
        double maxValue = 0;
        for (uint8_t i = 0; i < NUM_OCTAVES; i++) {
            uint8_t oct = i * OCTAVE_STEP;
            double decayValue = pow(texture, oct);
            double x = SEEDS[i] + perlinTime * (oct + 1);
            double randomValue = noise1d(x);
            noise += randomValue * decayValue;
            maxValue += decayValue;
        }
        noise /= maxValue;
        noise *= amplitude;
        MCP4922_write(CHIP_SELECT_PIN, i, (noise + 1) / 2);
    }

    const uint16_t NUM_CV_CHANNELS = 5;

    switch (loopCount % (CV_SAMPLING_FREQUENCY * NUM_CV_CHANNELS)) {
        case 0 * CV_SAMPLING_FREQUENCY: {
            speedPotValue = analogReadRange(SPEED_POT_PIN, MIN_SPEED, MAX_SPEED, 8);
            speed = clamp(speedCvValue + speedPotValue, MIN_SPEED, MAX_SPEED);
        } break;
        case 1 * CV_SAMPLING_FREQUENCY: {
            texturePotValue = analogReadRange(TEXTURE_POT_PIN, MIN_TEXTURE, MAX_TEXTURE, 0);
            texture = clamp(textureCvValue + texturePotValue, MIN_TEXTURE, MAX_TEXTURE);
        } break;
        case 2 * CV_SAMPLING_FREQUENCY: {
            amplitude = analogReadRange(ATTENUATION_POT_PIN, 0, 1, 0);
        } break;
        case 3 * CV_SAMPLING_FREQUENCY: {
            speedCvValue = analogReadRange(SPEED_CV_PIN, MIN_SPEED, MAX_SPEED, 0, true);
            speed = clamp(speedCvValue + speedPotValue, MIN_SPEED, MAX_SPEED);
        } break;
        case 4 * CV_SAMPLING_FREQUENCY: {
            textureCvValue = analogReadRange(TEXTURE_CV_PIN, MIN_TEXTURE, MAX_TEXTURE, 0, true);
            texture = clamp(textureCvValue + texturePotValue, MIN_TEXTURE, MAX_TEXTURE);
        } break;
    }

    loopCount += 1;
}

inline double analogReadRange(const uint8_t pin, const double min, const double max, const double exp) {
    return analogReadRange(pin, min, max, exp, false);
}
    
double analogReadRange(const uint8_t pin, const double min, const double max, const double exp, const bool transistorAdjust) {
    uint16_t rawValue = analogRead(pin);
    double x; 
    if (transistorAdjust) {
        x = ((double) rawValue - TRANSISTOR_ZERO_VALUE) / (TRANSISTOR_5V_VALUE - TRANSISTOR_ZERO_VALUE);
        x = clamp(x, 0, 1);
    } else {
        x = ((double) rawValue) / ANALOG_READ_MAX_VALUE;
    }
    
    // f(x) = 2^ax
    // g(x) = f(x) - f(0) = f(x) - 1
    // h(x) = g(x)/g(1)
    if (exp != 0) {
        x = (pow(2, x * exp) - 1) / (pow(2, exp) - 1);
    }
    return (1 - x) * min + x * max;
}

inline double clamp(const double x, const double min, const double max) {
    return x < min ? min : x > max ? max : x;
}

/*
 * Writes a given value to a MCP4922 DAC chip to be output as
 * a voltage.
 *
 * cs_pin - which Arduino pin to use as the CHIP SELECT pin
 *     (should be connected to the CS pin of the DAC)
 * dac - 0 or 1 - Which of the MCP4922's internal DAC channels
 *     to output to (see MCP4922 datasheet for pinout diagram)
 * value - {0..1} - The value to output as a fraction of the
 *     DAC's max/reference voltage. Converted to a 12-bit int.
 */
void MCP4922_write(int cs_pin, byte dac, float value) {
    uint16_t value12 = (uint16_t) (value * 4095);
    byte low = value12 & 0xff;
    byte high = (value12 >> 8) & 0x0f;
    dac = (dac & 1) << 7;
    digitalWrite(cs_pin, LOW);
    SPI.transfer(dac | 0x30 | high);
    SPI.transfer(low);
    digitalWrite(cs_pin, HIGH);
}
