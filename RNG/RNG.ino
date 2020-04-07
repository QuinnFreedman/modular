#include <stdint.h>
#include <SPI.h>
#include <Tlc5940.h>

/*
 * Pins
 */
const uint16_t RANDOMNESS_POT_PIN = A6;
const uint16_t DAC_CS_PIN = 8;
const uint16_t THRESHOLD_POT_PIN = A7;
const uint16_t GATE_PIN_A = 7;
const uint16_t GATE_PIN_B = 6;
const uint16_t RECORD_SWITCH_PIN = 5;
const uint16_t GATE_TRIG_SWITCH_PIN = 4;
const uint16_t CLOCK_IN_PIN = 2;

/*
 * Configuration
 */
const int NUM_LEDS = 7;
const uint16_t LED_OFFSET = 3;
const int TRIG_LENGTH_MS = 100;
const int MAX_BUFFER_SIZE = 32;
const bool YOYO = false;

// The max value for numbers in the buffer (given
// by the max value writable to the external DAC)
const uint16_t MAX_VALUE = pow(2, 12) - 1;

// Max value from an analogRead()
const int MAX_READ_VALUE = 1023;

/*
 * Global variables
 */

int bufferSize = 8;
uint16_t buffer[MAX_BUFFER_SIZE];

uint16_t randomness = 0;
uint16_t threshold = 0;
uint32_t lastRecordedTime;
uint32_t lastStepTime = 0;
bool reverse = false;

void setup() {
    // setup pins
    pinMode(RANDOMNESS_POT_PIN, INPUT);
    pinMode(THRESHOLD_POT_PIN, INPUT);
    pinMode(DAC_CS_PIN, OUTPUT);
    pinMode(GATE_PIN_A, OUTPUT);
    pinMode(GATE_PIN_B, OUTPUT);
    pinMode(GATE_TRIG_SWITCH_PIN, INPUT_PULLUP);
    pinMode(RECORD_SWITCH_PIN, INPUT_PULLUP);
    pinMode(CLOCK_IN_PIN, INPUT);

    digitalWrite(DAC_CS_PIN, HIGH);

    attachInterrupt(digitalPinToInterrupt(CLOCK_IN_PIN), stepInterruptWrapper, CHANGE);

    // Initialize 
    for (int i = 0; i < MAX_BUFFER_SIZE; i++) {
        buffer[i] = random(MAX_VALUE);
    }

    /* Serial.begin(9600); */
    
    Tlc.init();
    Tlc.clear();
    Tlc.update();

    SPI.begin();
    SPI.setBitOrder(MSBFIRST);
    SPI.setDataMode(SPI_MODE0);
    
    lastRecordedTime = millis();

    delay(100);
    loop();
    step();
}

/**
 * The loop function runs continuously. It just does preparation stuff like
 * always reading from the potentiometers so that when we get a signal from
 * the clock we already have up-to-date values for those things so we don't
 * have to waste time checking them.
 */
void loop() {
    randomness = analogRead(RANDOMNESS_POT_PIN);
    threshold = analogRead(THRESHOLD_POT_PIN) * (MAX_VALUE / MAX_READ_VALUE);
    bool gatesAreTriggers = digitalRead(GATE_TRIG_SWITCH_PIN);
    //TODO record analog input as soon as I can free up some pins with an LED driver
    /* bool record = digitalRead(RECORD_SWITCH_PIN); */
    
    lastRecordedTime = millis();

    if (gatesAreTriggers && lastRecordedTime - lastStepTime > TRIG_LENGTH_MS) {
        digitalWrite(GATE_PIN_A, LOW);
        digitalWrite(GATE_PIN_B, LOW);
    }
}

/**
 * Little wrapper function to call the `step()` function on a CHANGE
 * interrupt from the clock pin. Maybe we should just bind step directly
 * to a RISE trigger but I have gotten less false positives with this
 * method
 */
void stepInterruptWrapper() {
    static int lastValue = digitalRead(CLOCK_IN_PIN);
    const int value = digitalRead(CLOCK_IN_PIN);
    if (lastValue == LOW && value == HIGH) {
        step();
    }
    lastValue = value;
}

/**
 * This function is called via an interrupt whenever the module gets a clock pulse.
 * It does all the core work of stepping through the ringbuffer, doing the random
 * stuff, and setting the LEDs.
 */
void step() {
    static uint16_t ptr = 0;

    // maybe randomize the value at the cursor (ptr)
    if (random(MAX_READ_VALUE) < randomness) {
        buffer[ptr] = random(MAX_VALUE);
    }
    
    // Write the value to our DAC chip to output the main analog value
    MCP4922_write(DAC_CS_PIN, 0, buffer[ptr]);

    // handle the two gate outputs.
    bool gate;
    if (threshold <= 5) {
        gate = true;
    } else if (threshold >= MAX_VALUE - 10) {
        gate = false;
    } else {
        gate = buffer[ptr] > threshold;
    }
    digitalWrite(GATE_PIN_A, gate ? HIGH : LOW);
    digitalWrite(GATE_PIN_B, gate ? LOW : HIGH);

    // handle LEDs via TLC5940 driver
    for (uint16_t led = 0; led < NUM_LEDS; led++) {
        uint16_t index = (ptr + led - LED_OFFSET) % bufferSize;
        Tlc.set(led, buffer[index]);
    }
    Tlc.update();

    // Update the ptr/cursor to point to the next value in the buffer
    if (!YOYO) {
        ptr = (ptr + 1) % bufferSize;
        reverse = false;
    } else {
        if (reverse) {
            if (ptr == 0) {
                reverse = false;
            } else {
                ptr--;
            }
        } else {
            if (ptr == bufferSize - 1) {
                reverse = true;
            } else {
                ptr++;
            }
        }
    }

    // remember when we were here so we know when to turn off any trigger pulses
    lastStepTime = lastRecordedTime;
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
