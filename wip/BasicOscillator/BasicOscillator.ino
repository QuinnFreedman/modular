#include <stdint.h>
#include <SPI.h>

const float TWELFTH_ROOT_TWO = pow(2.0, 1.0 / 12.0);

// PINS
/*
// Arduino NANO
const uint16_t PITCH_CV_PIN = A0;
const uint16_t WAVE_SELECT_PIN = A1;
const uint16_t CHIP_SELECT_PIN = A4;
const uint16_t LED_PIN = A3;
*/

// Teensy 4.0
const uint16_t PITCH_CV_PIN = A0;
const uint16_t WAVE_SELECT_PIN = A1;
const uint16_t CHIP_SELECT_PIN = 4;
const uint16_t LED_PIN = 1;

// If set, the control voltage values for period and wave shape will
// be sampled at the beginning of every period of the sound wave.
// Sampling these values takes time, so it will create a small flat
// spot in the output waveform. If you are using a slow microcontroler, 
// having this irregularity at the same part of the wave every period
// might make the sound seem less "buzzy"
const bool SAMPLE_CV_AT_PERIOD_START = true;

// If SAMPLE_CV_AT_PERIOD_START is *false*, then the CV values will
// be sampled with every nth audio sample, where n is one of the
// values given below. There should be hundreds of audio samples
// per period, so the values below will sample the CV multiple
// times per audio loop. The synth code can gracefully handle changing
// frequency in the middle of a period, but you may find that you
// don't need to sample CV that frequently.
const int PITCH_CV_SAMPLE_RATE = 50;
const int WAVE_SELECT_SAMPLE_RATE = 100;

const int LARGE_PRIME = 79;

void setup() {
    {
        pinMode(CHIP_SELECT_PIN, OUTPUT);
        digitalWrite(CHIP_SELECT_PIN, HIGH);
 
        pinMode(LED_PIN, OUTPUT);
        flashLights();

        SPI.begin();
        SPI.setBitOrder(MSBFIRST);
        SPI.setDataMode(SPI_MODE0);

        pinMode(PITCH_CV_PIN, INPUT);
        pinMode(WAVE_SELECT_PIN, INPUT);
    }

    Serial.begin(9600);

    uint32_t periodStart = micros();

    /* float hertz = 500; */
    int mode = 2;
    uint32_t periodMicros = 0;
    float waveSelectPotValue = 0;
    bool periodReset = true;

    for (int i = 0;; i++) {
        bool shouldSamplePitchCV = false;
        bool shouldSampleWaveformCV = false;
        if (SAMPLE_CV_AT_PERIOD_START) {
            if (periodReset) {
                shouldSamplePitchCV = true;
                shouldSampleWaveformCV = true;
            }
        } else {
            if (i % PITCH_CV_SAMPLE_RATE == 0) {
                shouldSamplePitchCV = true;
            }
            // Add an offset (LARGE_PRIME) to reduce the likelyhood that
            // both CV parms will be sampled in the same loop.
            if ((i + LARGE_PRIME) % WAVE_SELECT_SAMPLE_RATE == 0) {
                shouldSampleWaveformCV = true;
            }

        }

        if (shouldSamplePitchCV) {
            // sample pitch cv
            /* uint16_t potValue = analogRead(PITCH_CV_PIN); */
            /* float newHertz = potValue + 100; */
            /* float hertzDelta = (newHertz - hertz) * 0.1; */
            /* hertz += hertzDelta; */
            /* periodMicros = (uint32_t) (1.0 / hertz * 1000000); */
            const uint16_t rawValue = analogRead(PITCH_CV_PIN);
            const float volts = rawValue * 5.0 / 1024.0;
            Serial.print("volts: ");
            Serial.println(volts);
            // semitones relative to a4, given 0v == c3
            const float semitones = volts * 12 - 21;
            Serial.print("semitones: ");
            Serial.println(semitones);
            const float hertz = TWELFTH_ROOT_TWO * semitones + 440;
            Serial.print("hertz: ");
            Serial.println(hertz);
            periodMicros = (uint32_t) (1.0 / hertz * 1000000);
        }
        
        if (shouldSampleWaveformCV) {
            // sample wave select
            uint16_t potValue = analogRead(WAVE_SELECT_PIN);
            float potValueDelta = (potValue - waveSelectPotValue) * 0.1;
            waveSelectPotValue += potValueDelta;
            mode = (uint16_t) (waveSelectPotValue / 256);
        }

        periodReset = false;
        uint32_t currentTime = micros();
        uint32_t elapsed = currentTime - periodStart;
        // assume this loop will only ever run once, but use `while` instead of `if` to
        // handle it slightly more gracefully if the chip freezes for more than a full
        // period (although that would still sound bad no matter what we do).
        // But using a loop here lets us keep the invarient 0 <= elapsed < periodMicros
        while (elapsed >= periodMicros) {
            elapsed -= periodMicros;
            periodStart = currentTime - elapsed;
            periodReset = true;
        }

        float elapsedFraction = ((float) elapsed) / ((float) periodMicros);

        float value = waveFunction(elapsedFraction, mode);

        MCP4922_write(CHIP_SELECT_PIN, 0, value);
    }

}

void loop() {
    // This function should never run
    flashLights();
}

void flashLights() {
    for (int i = 0; i < 5; i++) {
        digitalWrite(LED_PIN, HIGH);
        delay(100);
        digitalWrite(LED_PIN, LOW);
        delay(100);
    }
    digitalWrite(LED_PIN, HIGH);
}

const int SAW = 0;
const int TRI = 1;
const int SIN = 2;
const int SQR = 3;

/*
 * mode - {0, 1, 2, 3} - which wave function to use.
 * x - {0..1} - Time as a fraction of the period. 
 * returns {0..1} wave function value at time x.
 */
float waveFunction(const float x, const int mode) {
    switch(mode) {
        case SAW:
            return x;
        case TRI:
            if (x < .5) {
                return 2 * x;
            } else {
                return 1 - 2 * (x - .5);
            }
        case SIN:
            // TODO use a lookup table to make this faster
            return (1 + cos(x * 2 * PI)) / 2;
        case SQR:
            if (x < .5) {
                return 0;
            } else {
                return 1;
            }
    }
    flashLights();
    return 0;
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
