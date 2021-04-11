#include <stdint.h>
#include <SPI.h>
#include <Tlc5940.h>
#include "lib/digitalWriteFast.h"

const uint8_t BUTTON_LADDER_PIN = A0;
const uint8_t ANALOG_INPUT_PIN_A = A6;
const uint8_t ANALOG_INPUT_PIN_B = A7;
const uint8_t MENU_BUTTON_PIN = 2;
const uint8_t DAC_CS_PIN = 8;

const uint8_t TRIG_PIN_A = 4;
const uint8_t TRIG_PIN_B = 5;

const uint16_t LED_BRIGHT = (pow(2, 12) - 1) / 10;
const uint16_t LED_DIM = LED_BRIGHT / 4;
const uint16_t LED_OFF = 0;

const uint16_t ANALOG_READ_MAX_VALUE = 1023;

const uint8_t LED_INDEX[12] = {12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1};
float BUTTON_LADDER_CUTOFFS[12];

bool MENU_ON = true;
bool SHOULD_UPDATE_UI = false;
bool NOTES[12] = {1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1};

void initButtonCutoffs() {
    for (uint8_t i = 0; i < 12; i++) {
        BUTTON_LADDER_CUTOFFS[i] = 1 / (((float) i) + 1.5);
    }
}

void setup() {
    pinMode(BUTTON_LADDER_PIN, INPUT);
    pinMode(DAC_CS_PIN, OUTPUT);
    pinMode(ANALOG_INPUT_PIN_A, INPUT);
    pinMode(ANALOG_INPUT_PIN_B, INPUT);
    pinMode(TRIG_PIN_A, OUTPUT);
    pinMode(TRIG_PIN_B, OUTPUT);
    pinMode(MENU_BUTTON_PIN, INPUT_PULLUP);
    // attachInterrupt(digitalPinToInterrupt(MENU_BUTTON_PIN), handleMenuButton, CHANGE);

    Tlc.init();
    Tlc.clear();
    delay(10);
    Tlc.update();
    
    SPI.begin();
    SPI.setBitOrder(MSBFIRST);
    SPI.setDataMode(SPI_MODE0);
    
    Serial.begin(9600);

    initButtonCutoffs();
    Serial.println("voltage,quantized");
}

inline int16_t modulo(int16_t x, int16_t m) {
    int16_t mod = x % m;
    if (mod < 0) {
        mod += m;
    }
    return mod;
}

int16_t mod12(int16_t x) { return modulo(x, 12); }

float readInputVoltage(uint8_t pin) {
    uint16_t rawValue = analogRead(pin);
    return (rawValue / (float) ANALOG_READ_MAX_VALUE) * 20.0 - 10.0;
}

void writeOutputVoltage(uint8_t csPin, uint8_t dacChannel, float voltage) {
    const float minVoltage = -10;
    const float maxVoltage = 10;
    float dacOutput = (voltage - minVoltage) / (maxVoltage - minVoltage);
    MCP4922_write(csPin, dacChannel, dacOutput);
}

inline void handleButtons() {
    const uint32_t debounceTimeMilis = 10;
    static int8_t lastButtonDown = -1;
    static bool unstable = false;
    static uint32_t stabilizingStartTime = 0;
    static int8_t stabilizingValue = -1;
    static float oldButtonValue = 0;
    
    float buttonValue = analogRead(BUTTON_LADDER_PIN) / 1023.0;
    oldButtonValue = (oldButtonValue + 2 * buttonValue) / 3;
    buttonValue = oldButtonValue;
    int8_t buttonDown = -1;
    for (uint8_t i = 0; i < 12; i++) {
        if (buttonValue > BUTTON_LADDER_CUTOFFS[i]) {
            buttonDown = i;
            break;
        }
    }
    Serial.print(buttonValue * 12);
    Serial.print(",");
    Serial.print(buttonDown);
    for (uint8_t i = 0; i < 12; i++) {
        Serial.print(",");
        Serial.print(BUTTON_LADDER_CUTOFFS[i] * 12);
    }
    Serial.println();

    uint32_t now = millis();
    
    if (buttonDown != lastButtonDown) {
        stabilizingStartTime = now;
        unstable = true;
        stabilizingValue = buttonDown;
    }

    else if (unstable && now - stabilizingStartTime > debounceTimeMilis) {
        unstable = false;
        if (stabilizingValue == buttonDown && buttonDown != -1) {
            // Serial.print("Button pressed:");
            // Serial.println(buttonDown);
            if (MENU_ON) {
                // TODO
            } else {
                onButtonPressedNormal(buttonDown);
            }
        }
    }
    lastButtonDown = buttonDown;
}

void handleMenuButton() {
    MENU_ON = !digitalRead(MENU_BUTTON_PIN);
    SHOULD_UPDATE_UI = true;
}

void onButtonPressedNormal(uint8_t button) {
    NOTES[button] = !NOTES[button];
    SHOULD_UPDATE_UI = true;
}

inline float quantize(float inputVoltage) {
    const float a4_semitones = 33;
    const float semitones = inputVoltage * 12.0f - a4_semitones;
    const int16_t semitonesInt = round(semitones);
    int16_t nearestNote = -127;
    if (NOTES[mod12(semitonesInt)]) {
        nearestNote = semitonesInt;
    } else {
        for (int16_t i = 0; i < 12; i++) {
            if (NOTES[mod12(semitonesInt + i)]) {
                nearestNote = semitonesInt + i;
                break;
            }
            if (NOTES[mod12(semitonesInt - i)]) {
                nearestNote = semitonesInt - i;
                break;
            }
        }
    }
    return (nearestNote + a4_semitones) / 12.0f;
}

void loop() {
    {
        const bool menuButton = !digitalReadFast(MENU_BUTTON_PIN);
        SHOULD_UPDATE_UI = menuButton != MENU_ON;
        MENU_ON = menuButton;
    }
    handleButtons();

    /*
    {
        static float output[2] = {0, 0};
        const uint8_t INPUT_PINS[2] = {ANALOG_INPUT_PIN_A, ANALOG_INPUT_PIN_B};
        for (uint8_t i = 0; i < 2; i++) {
            float inputVoltage = readInputVoltage(INPUT_PINS[i]);
            float outputVoltage =  quantize(inputVoltage);
            writeOutputVoltage(DAC_CS_PIN, i, outputVoltage);
        }
    }
    */

    if (SHOULD_UPDATE_UI) {
        SHOULD_UPDATE_UI = false;
        if (MENU_ON) {
            for (int i = 0; i <= 12; i++) {
                Tlc.set(LED_INDEX[i], LED_DIM);
            }
            Tlc.update();
        } else {
            for (int i = 0; i <= 12; i++) {
                Tlc.set(LED_INDEX[i], NOTES[i] ? LED_DIM : LED_OFF);
            }
            Tlc.update();
        }
    }
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
