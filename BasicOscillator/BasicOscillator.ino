#include <cstdint>
#include <SPI.h>


// PINS
const uint16_t PITCH_CV_PIN = A0;
const uint16_t WAVE_SELECT_PIN = A1;
const uint16_t chip_select = 4;

const int PITCH_CV_SAMPLE_RATE = 50;
const int WAVE_SELECT_SAMPLE_RATE = 1;
//const int WAVE_SELECT_SAMPLE_RATE = 100;

void setup() {
    {
        pinMode(chip_select, OUTPUT);
        digitalWrite(chip_select, HIGH);

        pinMode(1, OUTPUT);
        for (int i = 0; i < 5; i++) {
            digitalWrite(1, HIGH);
            delay(100);
            digitalWrite(1, LOW);
            delay(100);
        }
        digitalWrite(1, HIGH);

        SPI.begin();
        SPI.setBitOrder(MSBFIRST);
        SPI.setDataMode(SPI_MODE0);

        pinMode(PITCH_CV_PIN, INPUT);
        pinMode(WAVE_SELECT_PIN, INPUT);
    }

    Serial.begin(9600);

    uint32_t periodStart = micros();

    float hertz = 500;
    int mode = 2;
    uint32_t periodMicros = 0;

    for (int i = 0;; i++) {
        if (i % PITCH_CV_SAMPLE_RATE == 0) {
            // sample pitch cv
            uint16_t potValue = analogRead(PITCH_CV_PIN);
            float newHertz = potValue + 100;
            float hertzDelta = (newHertz - hertz) * 0.1;
            hertz += hertzDelta;
            periodMicros = (uint32_t) (1.0 / hertz * 1000000);
        }
        
        if ((i + 79) % WAVE_SELECT_SAMPLE_RATE == 0) {
            // sample wave select
            uint16_t potValue = analogRead(WAVE_SELECT_PIN);
            mode = potValue / 256;
        }


        uint32_t currentTime = micros();
        uint32_t elapsed = currentTime - periodStart;
        while (elapsed >= periodMicros) {
            periodStart = currentTime;
            elapsed -= periodMicros;
        }

        float elapsedFraction = ((float) elapsed) / ((float) periodMicros);

        uint16_t value = (uint16_t) (waveFunction(elapsedFraction, mode) * 4095);

        MCP4922_write(chip_select, 0, value);
    }

}

void loop() {
    // This function should never run
}

const int SAW = 0;
const int TRI = 1;
const int SIN = 2;
const int SQR = 3;

/*
 * mode - {1, 2, 3, 4} - which wave function to use.
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
            return (1 + sin(x * 2 * PI)) / 2;
        case SQR:
            if (x < .5) {
                return 0;
            } else {
                return 1;
            }
    }
    return 0;
}


void MCP4922_write(int cs_pin, byte dac, uint16_t value) {
    byte low = value & 0xff;
    byte high = (value >> 8) & 0x0f;
    dac = (dac & 1) << 7;
    digitalWrite(cs_pin, LOW);
    SPI.transfer(dac | 0x30 | high);
    SPI.transfer(low);
    digitalWrite(cs_pin, HIGH);
}
