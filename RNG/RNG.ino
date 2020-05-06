#include <stdint.h>
#include <SPI.h>
#include <Tlc5940.h>

/*
 * Pins
 */
const uint16_t THRESHOLD_POT_PIN = A7;
const uint16_t RANDOMNESS_POT_PIN = A6;
const uint16_t RECORD_INPUT_PIN = A5;
//A4 bias cv
//A3 randomness CV 
const uint16_t ENCODER_BUTTON_PIN = A2;
const uint16_t ENCODER_PIN_B = A1;
const uint16_t ENCODER_PIN_A = A0;
//D9-13: LED Driver
const uint16_t DAC_CS_PIN = 8;
const uint16_t GATE_PIN_A = 7;
const uint16_t GATE_PIN_B = 6;
const uint16_t RECORD_SWITCH_PIN = 5;
const uint16_t GATE_TRIG_SWITCH_PIN = 4;
//D3: LED Driver
const uint16_t CLOCK_IN_PIN = 2;

/*
 * Configuration
 */
// The physical number of LEDs you have attatched to the 
// TLC5940 chip.
const int NUM_LEDS = 7;
// Which LED is the "current" one, i.e. the one thatshows the
// currently playing value and the one where randomness in added.
const uint16_t LED_OFFSET = 3;
// If in "trigger" mode, this is how many millisecons the trigger
// should stay on for.
const int TRIG_LENGTH_MS = 100;
// The maximum buffer size. AKA the maximul size of the looping
// sequence of random numbers. Can be more or less than NUM_LEDS but
// must allow enough digits to display the buffer size in binary
// plus one for a sign bit. So, if you have a max buffer size of
// 32, you need 2^(6)=32, 6+1=7 LEDs.
const int MAX_BUFFER_SIZE = 32;
// How many miliseconds to show the buffer size after changing it
// before going back to showing the buffer values
const int BUFFER_SIZE_SHOW_TIME = 1000;
// If true, holding down the buffer size dial will increment values
// in powers of two. If false, powers of two are the default and
// holding it down allows you to select other numbers
const int POW2_ON_HOLD = true;

// The max value for numbers in the buffer (given
// by the max value writable to the external DAC)
const uint16_t MAX_VALUE = pow(2, 12) - 1;

// Max value from an analogRead()
const int MAX_READ_VALUE = 1023;

/*
 * Global variables
 */

volatile bool showBufferSize = false;
volatile int bufferSize = 8;
volatile bool yoyo = false;
volatile uint32_t showBufferSizeTimerStart = 0;

uint16_t buffer[MAX_BUFFER_SIZE];

uint16_t randomness = 0;
uint16_t threshold = 0;
uint32_t lastRecordedTime;
uint32_t lastStepTime = 0;
bool reverse = false;
bool encoderButtonDown = false;
bool recording = false;

void setup() {
Serial.begin(9600);
    // setup pins
    pinMode(RANDOMNESS_POT_PIN, INPUT);
    pinMode(THRESHOLD_POT_PIN, INPUT);
    pinMode(DAC_CS_PIN, OUTPUT);
    pinMode(GATE_PIN_A, OUTPUT);
    pinMode(GATE_PIN_B, OUTPUT);
    pinMode(GATE_TRIG_SWITCH_PIN, INPUT_PULLUP);
    pinMode(RECORD_SWITCH_PIN, INPUT_PULLUP);
    pinMode(CLOCK_IN_PIN, INPUT);
	pinMode(ENCODER_PIN_A, INPUT_PULLUP);
	pinMode(ENCODER_PIN_B, INPUT_PULLUP);
	pinMode(ENCODER_BUTTON_PIN, INPUT_PULLUP);

    digitalWrite(DAC_CS_PIN, HIGH);

    attachInterrupt(digitalPinToInterrupt(CLOCK_IN_PIN), stepInterruptWrapper, CHANGE);
    enableInterrupt(ENCODER_PIN_A);

    // Initialize 
    for (int i = 0; i < MAX_BUFFER_SIZE; i++) {
        buffer[i] = random(MAX_VALUE);
    }

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

// Set up additional interrupt listeners
ISR(PCINT1_vect) {
    rotaryEncoderHandler();
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
    //TODO use interrupts for these boolean reads
    bool gatesAreTriggers = digitalRead(GATE_TRIG_SWITCH_PIN);
    encoderButtonDown = digitalRead(ENCODER_BUTTON_PIN) == LOW;
    /*
    recording = digitalRead(RECORD_SWITCH_PIN);
    if (recording) {
        lastRecordedInputValue = analogRead(RECORD_INPUT_PIN) * (MAX_VALUE / MAX_READ_VALUE);
    }
    */
    
    lastRecordedTime = millis();

    if (gatesAreTriggers && lastRecordedTime - lastStepTime > TRIG_LENGTH_MS) {
        digitalWrite(GATE_PIN_A, LOW);
        digitalWrite(GATE_PIN_B, LOW);
    }

    if (showBufferSize && lastRecordedTime - showBufferSizeTimerStart > BUFFER_SIZE_SHOW_TIME) {
        showBufferSize = false;
        delay(10);
        updateLeds();
    } else if (showBufferSize) {
        updateLeds();
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

uint16_t ptr = 0;
/**
 * This function is called via an interrupt whenever the module gets a clock pulse.
 * It does all the core work of stepping through the ringbuffer, doing the random
 * stuff, and setting the LEDs.
 */
void step() {
    //int bufferSize = bufferSize;

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

    // update display
    updateLeds();

    // Update the ptr/cursor to point to the next value in the buffer
    if (!yoyo) {
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

void onRotaryKnobTurned(int direction, int counter) {
    if (yoyo) {
        direction *= -1;
    }

    if (bufferSize == 1) {
        if (direction == -1) {
            yoyo = !yoyo;
        } else {
            bufferSize = 2;
        }
    } else if (encoderButtonDown && POW2_ON_HOLD) {
        uint16_t i = 1;
        while (true) {
            uint16_t powerOfTwo = (0x01 << i);
            if (powerOfTwo == bufferSize) {
                bufferSize = 0x01 << (i + direction);
                break;
            } else if (powerOfTwo > bufferSize) {
                if (direction == -1) {
                    bufferSize = 0x01 << (i - 1);
                } else {
                    bufferSize = powerOfTwo;
                }
                break;
            }
            i++;
        }
    } else {
        bufferSize += direction;
    }
    if (bufferSize > MAX_BUFFER_SIZE) {
        bufferSize = MAX_BUFFER_SIZE;
    }

    showBufferSize = true;
    showBufferSizeTimerStart = lastRecordedTime;
}

void updateLeds() {
    //TODO cache led output values to reduce the number of
    //Tlc.update() calls
    if (showBufferSize) {
        for (uint16_t i = 0; i < NUM_LEDS - 1; i++) {
            bool ledOn = (bufferSize >> i) & 0x01;
            Tlc.set(i, ledOn ? MAX_VALUE : 0);
        }
        Tlc.set(NUM_LEDS - 1, yoyo ? MAX_VALUE : 0);
        Tlc.update();
    } else {
        for (uint16_t led = 0; led < NUM_LEDS; led++) {
            uint16_t index = (ptr + led - LED_OFFSET) % bufferSize;
            Tlc.set(led, buffer[index]);
        }
        Tlc.update();
    }

}

/**
 * Handler function called on an interrupt when the rotary encoder is turned.
 */
void rotaryEncoderHandler() {
    static int lastCLK = digitalRead(ENCODER_PIN_A);
    static int counter = 0;
    
    const int a = digitalRead(ENCODER_PIN_A);
    const int b = digitalRead(ENCODER_PIN_B);

    int currentStateCLK = a;
    if (currentStateCLK != lastCLK && currentStateCLK == 1) {
        int direction;
        if (b != currentStateCLK) {
            direction = -1;
        } else {
            direction = 1;
        }
        counter += direction;

        onRotaryKnobTurned(direction, counter);
    }

    lastCLK = currentStateCLK;
}

inline void enableInterrupt(byte pin) {
    *digitalPinToPCMSK(pin) |= bit (digitalPinToPCMSKbit(pin));  // enable pin
    PCIFR  |= bit (digitalPinToPCICRbit(pin)); // clear any outstanding interrupt
    PCICR  |= bit (digitalPinToPCICRbit(pin)); // enable interrupt for the group
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
