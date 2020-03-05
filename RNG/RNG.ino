#include <stdint.h>
#include <assert.h>
#include <SPI.h>

const int NUM_LEDS = 6;
const uint16_t LED_PINS[NUM_LEDS] = {
    A0, A1, A2, A3, A4, A5
};
const uint16_t RANDOMNESS_POT_PIN = A6;
const uint16_t DAC_CS_PIN = 2;
const uint16_t THRESHOLD_POT_PIN = A7;
const uint16_t GATE_PIN = 3;

const int LED_OFFSET = 2;

const int MAX_BUFFER_SIZE = 32;

int bufferSize = 6;
uint16_t buffer[MAX_BUFFER_SIZE];

// The max value for numbers in the buffer (given
// by the max value writable to the external DAC)
const uint16_t MAX_VALUE = pow(2, 12) - 1;

const int MAX_READ_VALUE = 1023; // Max value from an analogRead()

void setup() {
    for (int i = 0; i < NUM_LEDS; i++) {
        pinMode(LED_PINS[i], OUTPUT);
    }
    pinMode(RANDOMNESS_POT_PIN, INPUT);
    pinMode(THRESHOLD_POT_PIN, INPUT);
    pinMode(DAC_CS_PIN, OUTPUT);
    pinMode(GATE_PIN, OUTPUT);

    digitalWrite(DAC_CS_PIN, HIGH);

    for (int i = 0; i < MAX_BUFFER_SIZE; i++) {
        buffer[i] = random(MAX_VALUE);
    }
    buffer[0] = MAX_VALUE;

    SPI.begin();
    SPI.setBitOrder(MSBFIRST);
    SPI.setDataMode(SPI_MODE0);
}

int randomness = 0;
int threshold = 0;

void loop() {
    randomness = analogRead(RANDOMNESS_POT_PIN);
    threshold = (analogRead(THRESHOLD_POT_PIN)) * (MAX_VALUE / MAX_READ_VALUE);
    step();
    delay(500);
}

int ptr = 0;
void step() {
    if (random(MAX_READ_VALUE) < randomness) {
        buffer[ptr] = random(MAX_VALUE);
    }
    
    MCP4922_write(DAC_CS_PIN, 0, buffer[ptr]);

    bool gate;
    if (threshold <= 5) {
        gate = true;
    } else if (threshold >= MAX_VALUE - 10) {
        gate = false;
    } else {
        gate = buffer[ptr] > threshold;
    }
    
    digitalWrite(GATE_PIN, gate ? HIGH : LOW);

    for (int led = 0; led < NUM_LEDS; led++) {
        int index = (ptr + led) % bufferSize;
        analogWrite(LED_PINS[(led + LED_OFFSET) % NUM_LEDS], (float) buffer[index] / (MAX_VALUE / 255));
    }

    ptr = (ptr + 1) % bufferSize;
}

void MCP4922_write(int cs_pin, byte dac, uint16_t value) {
    byte low = value & 0xff;
    byte high = (value >> 8) & 0x0f;
    dac = (dac & 1) << 7;
    digitalWrite(cs_pin, LOW);
    delay(100);
    SPI.transfer(dac | 0x30 | high);
    SPI.transfer(low);
    delay(100);
    digitalWrite(cs_pin, HIGH);
}
