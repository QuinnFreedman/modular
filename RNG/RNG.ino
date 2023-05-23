#include <stdint.h>
#include <SPI.h>
#include <Tlc5940.h>

/*
 * Pins
 */
const uint8_t BIAS_CV_PIN = A0;
const uint8_t CV_ENABLE_PIN = A1;
const uint8_t ENABLED_LED_PIN = A2;
const uint8_t TRIGGER_LED_PIN = A3;
const uint8_t ENCODER_BUTTON_PIN = A5;
const uint8_t GATE_TRIG_SWITCH_PIN = A6;
const uint8_t RANDOMNESS_CV_PIN = A7;

const uint8_t CLOCK_IN_PIN = 2;
//D3: LED Driver
const uint8_t ENCODER_PIN_B = 4;
const uint8_t ENCODER_PIN_A = 5;
const uint8_t GATE_PIN_B = 6;
const uint8_t GATE_PIN_A = 7;
const uint8_t DAC_CS_PIN = 8;
//D9-13: LED Driver

/*
 * Configuration
 */
// The physical number of LEDs you have attatched to the 
// TLC5940 chip.
const int NUM_LEDS = 7;
// Which LED is the "current" one, i.e. the one that shows the
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
const int POW2_ON_HOLD = false;

// The max value for numbers in the buffer
// This is currently the max value that can be output by the DAC
// AND the max value that can be output to the LED controller.
// If one of those changes, the buffer values will have to be
// adjusted before writing to one or the other.
const uint16_t MAX_VALUE = (2 << 12) - 1;

// Max value from an analogRead()
// see: https://www.arduino.cc/reference/en/language/functions/analog-io/analogread/
const int MAX_READ_VALUE = 1023;

// If true, LEDs will only be on/off, depending on if the corresponding
// buffer value would trigger gate A or not. Otherwise, grayscale values
// for LED brightness are shown
const bool LED_BINARY = true;
// If true, LED brightness will be one of 4 discrete brightness levels
const bool LED_QUANTIZED = false;
// If true, the buffer values will appear to move left-to-right through
// the led display. Otherwise, values will move right-to-left
const bool LED_MOVE_LTR = true;

// The potentiometers can't go all the way to to 0 or 100%. These offsets adjust
// the input values from pots. Min input value from pots:
const uint16_t POT_ZERO_VALUE = 5;
// max input value from pots
const uint16_t POT_MAX_VALUE = MAX_READ_VALUE - 5;

#define DEBUG_ENABLED false
#define DEBUG_AUTOSTEP false

/*
 * Global variables
 */
volatile uint8_t bufferSize = 8;
volatile bool yoyo = false;
volatile uint16_t buffer[MAX_BUFFER_SIZE];
volatile uint16_t randomness = 0;
volatile uint32_t currentTime = 0;
volatile uint32_t lastStepTime = 0;
volatile uint32_t lastBufferLengthChangeTime = 0;
volatile bool encoderButtonDown = false;
volatile uint8_t ptr = 0;
volatile uint16_t outputValue = 0;

void setup() {
    #if DEBUG_ENABLED
    Serial.begin(9600);
    #endif
    // setup pins
    pinMode(BIAS_CV_PIN, INPUT);
    pinMode(CV_ENABLE_PIN, INPUT);
    pinMode(ENABLED_LED_PIN, OUTPUT);
    pinMode(TRIGGER_LED_PIN, OUTPUT);
    pinMode(ENCODER_BUTTON_PIN, INPUT_PULLUP);
    pinMode(GATE_TRIG_SWITCH_PIN, INPUT);
    pinMode(RANDOMNESS_CV_PIN, INPUT);
    pinMode(CLOCK_IN_PIN, INPUT);
    pinMode(ENCODER_PIN_B, INPUT_PULLUP);
    pinMode(ENCODER_PIN_A, INPUT_PULLUP);
    pinMode(GATE_PIN_B, OUTPUT);
    pinMode(GATE_PIN_A, OUTPUT);
    pinMode(DAC_CS_PIN, OUTPUT);

    digitalWrite(DAC_CS_PIN, HIGH);

    Tlc.init();
    Tlc.clear();
    Tlc.update();

    SPI.begin();
    SPI.setBitOrder(MSBFIRST);
    SPI.setDataMode(SPI_MODE0);

    for (int i = 0; i < MAX_BUFFER_SIZE; i++) {
        buffer[i] = random(MAX_VALUE);
    }

    showStartupLEDS();

    currentTime = millis();

    attachInterrupt(digitalPinToInterrupt(CLOCK_IN_PIN), stepInterruptWrapper, CHANGE);
    enableInterrupt(ENCODER_PIN_A);
}

void showStartupLEDS() {
    const uint8_t stepTime = 120;
    Tlc.clear();
    Tlc.update();
    delay(stepTime);
    uint32_t lastLedUpdate = millis();
    for (uint8_t i = 0; i <= NUM_LEDS / 2; i++) {
        Tlc.set(NUM_LEDS / 2 + i, MAX_VALUE);
        Tlc.set(NUM_LEDS / 2 - i, MAX_VALUE);
        Tlc.update();
        uint32_t now = millis();
        uint32_t delayTime = stepTime - (lastLedUpdate - now);
        lastLedUpdate = now;
        delay(delayTime);
    }
}


ISR(PCINT2_vect) {
    static_assert(ENCODER_PIN_A >= 0 && ENCODER_PIN_A <= 7 &&
                  ENCODER_PIN_B >= 0 && ENCODER_PIN_B <= 7, 
                  "This code assumes that the encoder pins are Arduino digital pins 0..7. Look up PCINT vec for more.");
    rotaryEncoderHandler();
}

/**
 * The loop function runs continuously. It just does preparation stuff like
 * always reading from the potentiometers so that when we get a signal from
 * the clock we already have up-to-date values for those things so we don't
 * have to waste time checking them.
 */
void loop() {
    float _randomness = analogReadPot(RANDOMNESS_CV_PIN) * MAX_VALUE;
    boolean freeze = _randomness == 0 || !digitalRead(CV_ENABLE_PIN);
    if (freeze) {
        _randomness = 0;
    }
    randomness = _randomness;
    uint16_t threshold = MAX_VALUE * analogReadPot(BIAS_CV_PIN);
    //TODO use interrupts for these boolean reads
    bool gatesAreTriggers = analogRead(GATE_TRIG_SWITCH_PIN) > MAX_READ_VALUE / 2;
    encoderButtonDown = !digitalRead(ENCODER_BUTTON_PIN);

    // Write the value to our DAC chip to output the main analog value
    static int16_t lastOutput = -1;
    if (lastOutput != outputValue) {
        MCP4922_write(DAC_CS_PIN, 0, buffer[ptr]);
        lastOutput = outputValue;
    }

    currentTime = millis();

    // Update the gate/trigger outputs
    bool inTriggerWindow = currentTime - lastStepTime < TRIG_LENGTH_MS;
    bool outputGateA = buffer[ptr] > threshold;
    bool outputGateB = !outputGateA;
    if (gatesAreTriggers && !inTriggerWindow) {
        outputGateA = false;
        outputGateB = false;
    }
    digitalWrite(GATE_PIN_A, outputGateA);
    digitalWrite(GATE_PIN_B, outputGateB);
    digitalWrite(ENABLED_LED_PIN, !freeze);
    digitalWrite(TRIGGER_LED_PIN, inTriggerWindow);

    // update display
    bool showBufferSize = encoderButtonDown || currentTime - lastBufferLengthChangeTime <= BUFFER_SIZE_SHOW_TIME;
    updateLeds(showBufferSize, threshold);

    #if DEBUG_AUTOSTEP
    if (currentTime - lastStepTime > 2000) {
        step();
    }
    #endif
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
    // maybe randomize the value at the cursor (ptr)
    if (random(MAX_READ_VALUE) < randomness) {
        buffer[ptr] = random(MAX_VALUE);
    }
    
    // Update the ptr/cursor to point to the next value in the buffer
    static bool reverse = false;
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

    // record the time so we know when to turn off trigger pulses
    lastStepTime = currentTime;

    #if DEBUG_ENABLED
    Serial.println("step:");
    for (uint8_t i = 0; i < bufferSize; i++) {
        Serial.print(buffer[i], HEX);
        Serial.print(" ");
    }
    Serial.println();
    Serial.print("ptr ");
    Serial.println(ptr);
    Serial.print("randomness ");
    Serial.println(randomness);
    #endif
}

void onRotaryKnobTurned(int direction) {
    #if DEBUG_ENABLED
    Serial.print("onRotaryKnobTurned(");
    Serial.print(direction);
    Serial.println(")");
    #endif
    if (yoyo) {
        direction *= -1;
    }

    if (bufferSize == 1) {
        if (direction == -1) {
            yoyo = !yoyo;
        } else {
            bufferSize = 2;
        }
    } else if (encoderButtonDown == POW2_ON_HOLD) {
        uint16_t i = 1;
        while (true) {
            uint16_t powerOfTwo = (0x01 << i);
            #if DEBUG_ENABLED
            #endif
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

    lastBufferLengthChangeTime = currentTime;
}

void updateLeds(bool showBufferSize, uint16_t threshold) {
    static uint16_t lastOutput[NUM_LEDS] = {0, 0, 0, 0, 0, 0, 0};
    uint16_t output[NUM_LEDS];
    if (showBufferSize) {
        for (uint8_t i = 0; i < NUM_LEDS - 1; i++) {
            bool ledOn = (bufferSize >> i) & 0x01;
            output[(NUM_LEDS - 1) - i] = ledOn ? MAX_VALUE : 0;
        }
        output[0] = yoyo ? MAX_VALUE : 0;
    } else {
        for (uint8_t led = 0; led < NUM_LEDS; led++) {
            uint8_t index = (ptr + (LED_MOVE_LTR ? -led + LED_OFFSET : led - LED_OFFSET)) % bufferSize;
            if (LED_BINARY) {
                output[led] = buffer[index] > threshold ? MAX_VALUE : 0;
            } else if (LED_QUANTIZED) {
                output[led] = ((buffer[index] >> 9) << 9) | 0b111111111;
            } else {
                output[led] = buffer[index];
            }
        }
    }
    // Only output to the TLC controller if the LEDs have changed
    if (memcmp(output, lastOutput, sizeof(output))) {
        for (uint8_t i = 0; i < NUM_LEDS; i++) {
            lastOutput[i] = output[i];
            Tlc.set(i, output[i]);
        }
        Tlc.update();
    }
}

/**
 * Handler function called on an interrupt when the rotary encoder is turned.
 */
void rotaryEncoderHandler() {
    static bool lastA = digitalRead(ENCODER_PIN_A);
    
    bool a = digitalRead(ENCODER_PIN_A);
    bool b = digitalRead(ENCODER_PIN_B);

    if (a && !lastA) {
        int8_t direction = a == b ? 1 : -1;
        onRotaryKnobTurned(direction);
    }

    lastA = a;
}

inline float clamp(const float x, const float min, const float max) {
    return x < min ? min : x > max ? max : x;
}

float analogReadPot(const uint16_t pin) {
    uint16_t rawValue = analogRead(pin);
    float x = ((float) rawValue - POT_ZERO_VALUE) / (POT_MAX_VALUE - POT_ZERO_VALUE);
    return clamp(x, 0, 1);
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
