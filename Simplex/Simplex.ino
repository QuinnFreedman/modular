#include <stdint.h>
#include <SPI.h>

#include "config.h"

const uint8_t NUM_CHANNELS = 2;

static_assert(FIRMWARE_MODE == FIRMWARE_MODE_SIMPLEX || FIRMWARE_MODE == FIRMWARE_MODE_LFO,
              "FIRMWARE_MODE must be one of FIRMWARE_MODE_SIMPLEX or FIRMWARE_MODE_LFO");

#if FIRMWARE_MODE == FIRMWARE_MODE_SIMPLEX

extern "C" {
    #include "perlin.h"
}
uint8_t SEEDS[NUM_CHANNELS][NUM_OCTAVES];

#else
    
#include "lfo.hpp"

#if AUX_MODE == AUX_MODE_SKEW

float_t skew = 0.5;
float_t skewWave(float_t x) {
    if (x < 0 || x > 1) return 0;
    if (skew < 0 || skew > 1) return 0;
    if (x < skew) {
        return (1 / skew) * x;
    }
    return 1 - (1 / (1 - skew)) * (x - skew);
}

LFO<float_t> lfos[NUM_CHANNELS] = {
    LFO<float_t>(skewWave),
    LFO<float_t>(Waveforms::tri<float_t>)
};

#else

LFO<float_t> lfos[NUM_CHANNELS] = {
    LFO<float_t>(Waveforms::tri<float_t>),
    LFO<float_t>(Waveforms::tri<float_t>)
};

#endif

#endif


void setup() {
    pinMode(CHIP_SELECT_PIN, OUTPUT);
    pinMode(POT_PIN_1, INPUT);
    pinMode(POT_PIN_2, INPUT);
    pinMode(POT_PIN_3, INPUT);
    pinMode(CV_PIN_1, INPUT_PULLUP);
    pinMode(CV_PIN_2, INPUT_PULLUP);
    digitalWrite(CHIP_SELECT_PIN, HIGH);

    SPI.begin();
    SPI.setBitOrder(MSBFIRST);
    SPI.setDataMode(SPI_MODE0);

    //Serial.begin(9600);

    /*
    for (int i = 0; i < 5; i++) {
        MCP4922_write(CHIP_SELECT_PIN, 0, 1);
        MCP4922_write(CHIP_SELECT_PIN, 1, 1);
        delay(300);
        MCP4922_write(CHIP_SELECT_PIN, 0, 0);
        MCP4922_write(CHIP_SELECT_PIN, 1, 1);
        delay(300);
    }
    */

    #if FIRMWARE_MODE == FIRMWARE_MODE_SIMPLEX
    for (int i = 0; i < NUM_CHANNELS; i++) {
        for (int j = 0; j < NUM_OCTAVES; j++) {
            SEEDS[i][j] = random(0, 256);
        }
    }
    #endif
}

#if FIRMWARE_MODE == FIRMWARE_MODE_SIMPLEX
float simplexLoop(uint16_t channel, float_t speed, float_t texture, float_t amplitude, uint32_t time) {
    static float_t perlinTime = 0;
    static uint32_t lastTime = micros();
    uint32_t dt = time - lastTime;
    lastTime = time;
    perlinTime += dt * speed;
    while (perlinTime > 256) {
        perlinTime -= 256;
    }

    {
        float_t noise = 0;
        float_t maxValue = 0;
        for (uint8_t i = 0; i < NUM_OCTAVES; i++) {
            uint8_t oct = i * OCTAVE_STEP;
            float_t decayValue = pow(texture, oct);
            float_t x = SEEDS[channel][i] + perlinTime * (oct + 1);
            float_t randomValue = noise1d(x);
            noise += randomValue * decayValue;
            maxValue += decayValue;
        }
        noise /= maxValue;
        if (channel == 0) {
            noise *= amplitude;
        }
        return (noise + 1) / 2;
    }    
}
#else
float lfoLoop(uint16_t channel, float_t hertzA, float_t hertzB, float_t aux, uint32_t time) {
    static float_t oldHertz[NUM_CHANNELS] = {0, 0};
    float_t hertz = channel == 0 ? hertzA : hertzB;
    if (hertz != oldHertz[channel]) {
        lfos[channel].setHertz(hertz);
        oldHertz[channel] = hertz;
    }

    #if AUX_MODE == AUX_MODE_SKEW
    skew = aux;
    #endif

    #if AUX_MODE == AUX_MODE_WAVEFORM
    if (channel == 0) {
        static uint8_t oldWave = WAVEFORM_TRI;
        const uint8_t wave = (aux - 0.001) * NUM_WAVEFORMS;
        if (wave != oldWave) {
            oldWave = wave;
            const uint8_t index = WAVEFORMS[wave];
            const waveform_t<float_t> waveFunctions[NUM_WAVEFORMS] = {
                Waveforms::saw,
                Waveforms::inverseSaw,
                Waveforms::tri,
                Waveforms::sin,
                Waveforms::square,
                Waveforms::bounce,
            };
            lfos[channel].setWaveform(waveFunctions[index]);
        }
    }
    #endif

    float_t value = lfos[channel].update(time);
    #if AUX_MODE == AUX_MODE_APLITUDE
    if (channel == 0) {
        value *= aux;
    }
    #endif
    return value;
}
#endif

void loop() {
    static float_t potValue1 = 0; // speed
    static float_t potValue2 = 0; // texture
    static float_t cvValue1 = 0;  // speed
    static float_t cvValue2 = 0;  // texture
    static float_t inputChannel1 = 0; // speed
    static float_t inputChannel2 = 0; // texture
    static float_t inputChannel3 = 0; // amplitude
    static uint16_t channel = 0;
    static uint32_t loopCount = 0;

    uint32_t time = micros();
    #if FIRMWARE_MODE == FIRMWARE_MODE_SIMPLEX
    float_t value = simplexLoop(channel, inputChannel1, inputChannel2, inputChannel3, time);
    #else
    float_t value = lfoLoop(channel, inputChannel1, inputChannel2, inputChannel3, time);
    #endif
    /*
    if (channel == 0) {
        Serial.println(value);
    }
    */
    MCP4922_write(CHIP_SELECT_PIN, channel, value);
        
    channel = (channel + 1) % NUM_CHANNELS;

    const uint16_t NUM_CV_CHANNELS = 5;

    switch (loopCount % (CV_SAMPLING_FREQUENCY * NUM_CV_CHANNELS)) {
        case 0 * CV_SAMPLING_FREQUENCY: {
            const float_t MIN = ANALOG_READ_RANGES[0][0];
            const float_t MAX = ANALOG_READ_RANGES[0][1];
            const float_t EXP = ANALOG_READ_RANGES[0][2];
            potValue1 = analogReadRange(POT_PIN_1, MIN, MAX, EXP);
            inputChannel1 = clamp(cvValue1 + potValue1, MIN, MAX);
        } break;
        case 1 * CV_SAMPLING_FREQUENCY: {
            const float_t MIN = ANALOG_READ_RANGES[1][0];
            const float_t MAX = ANALOG_READ_RANGES[1][1];
            const float_t EXP = ANALOG_READ_RANGES[1][2];
            potValue2 = analogReadRange(POT_PIN_2, MIN, MAX, EXP);
            inputChannel2 = clamp(cvValue2 + potValue2, MIN, MAX);
        } break;
        case 2 * CV_SAMPLING_FREQUENCY: {
            const float_t MIN = ANALOG_READ_RANGES[2][0];
            const float_t MAX = ANALOG_READ_RANGES[2][1];
            const float_t EXP = ANALOG_READ_RANGES[2][2];
            inputChannel3 = analogReadRange(POT_PIN_3, MIN, MAX, EXP);
        } break;
        case 3 * CV_SAMPLING_FREQUENCY: {
            const float_t MIN = ANALOG_READ_RANGES[3][0];
            const float_t MAX = ANALOG_READ_RANGES[3][1];
            const float_t EXP = ANALOG_READ_RANGES[3][2];
            cvValue1 = analogReadRange(CV_PIN_1, MIN, MAX, EXP, true);
            inputChannel1 = clamp(cvValue1 + potValue1, MIN, MAX);
        } break;
        case 4 * CV_SAMPLING_FREQUENCY: {
            const float_t MIN = ANALOG_READ_RANGES[4][0];
            const float_t MAX = ANALOG_READ_RANGES[4][1];
            const float_t EXP = ANALOG_READ_RANGES[4][2];
            cvValue2 = analogReadRange(CV_PIN_2, MIN, MAX, EXP, true);
            inputChannel2 = clamp(cvValue2 + potValue2, MIN, MAX);
        } break;
    }

    loopCount += 1;
}

inline float_t analogReadRange(const uint8_t pin, const float_t min, const float_t max, const float_t exp) {
    return analogReadRange(pin, min, max, exp, false);
}
    
float_t analogReadRange(const uint8_t pin, const float_t min, const float_t max, const float_t exp, const bool transistorAdjust) {
    uint16_t rawValue = analogRead(pin);
    float_t x; 
    if (transistorAdjust) {
        x = ((float_t) rawValue - TRANSISTOR_ZERO_VALUE) / (TRANSISTOR_5V_VALUE - TRANSISTOR_ZERO_VALUE);
        x = clamp(x, 0, 1);
    } else {
        x = ((float_t) rawValue) / ANALOG_READ_MAX_VALUE;
    }
    
    // f(x) = 2^ax
    // g(x) = f(x) - f(0) = f(x) - 1
    // h(x) = g(x)/g(1)
    if (exp != 0) {
        x = (pow(2, x * exp) - 1) / (pow(2, exp) - 1);
    }
    return (1 - x) * min + x * max;
}

inline float_t clamp(const float_t x, const float_t min, const float_t max) {
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
