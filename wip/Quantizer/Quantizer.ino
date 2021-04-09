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
bool NOTES[12] = {0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0};

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
}

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

inline void quantizeVoltage(uint8_t inputPin, uint8_t dacCSPin, uint8_t dacChannel) {
    float inputVoltageA = readInputVoltage(ANALOG_INPUT_PIN_A);
    Serial.print("Input voltage: ");
    Serial.println(inputVoltageA);
    float a4_semitones = 33;
    float semitones = inputVoltageA * 12.0f - a4_semitones;
    int8_t semitonesInt = round(semitones);
    int8_t nearestNote = -999;
    if (NOTES[semitonesInt % 12]) {
        nearestNote = semitonesInt;
    } else {
        for (int8_t i = 0; i < 12; i++) {
            if (NOTES[(semitonesInt + i) % 12]) {
                nearestNote = semitonesInt + i;
                continue;
            } else if (NOTES[(semitonesInt - i) % 12]) {
                nearestNote = semitonesInt - i;
                continue;
            }
        }
    }
    float outputVoltage = (nearestNote + a4_semitones) / 12.0f;
    writeOutputVoltage(dacCSPin, dacChannel, outputVoltage);
}

void loop() {
    {
        const bool menuButton = !digitalReadFast(MENU_BUTTON_PIN);
        SHOULD_UPDATE_UI = menuButton != MENU_ON;
        MENU_ON = menuButton;
    }
    handleButtons();

    //float voltInA = readInputVoltage(ANALOG_INPUT_PIN_A);
    //float voltInB = readInputVoltage(ANALOG_INPUT_PIN_B);
    /*
    Serial.print(voltInA);
    Serial.print(" ");
    Serial.print(voltInB);
    Serial.println();
    */

    /*
    static int i = 0;
    i++;
    if (i > 100) { i = 0; }
    MCP4922_write(DAC_CS_PIN, 0, i / 100.0);
    MCP4922_write(DAC_CS_PIN, 1, i / 100.0);
    Serial.println(i / 100.0);
    delay(10);
    */

    /*
    bool gateA = digitalRead(TRIG_PIN_A);
    bool gateB = digitalRead(TRIG_PIN_B);
    MCP4922_write(DAC_CS_PIN, 0, gateA);
    MCP4922_write(DAC_CS_PIN, 1, gateB);
    Serial.print(gateA);
    Serial.print(",");
    Serial.println(gateB);
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
    /*
    delay(300);
    for (int i = 0; i <= 12; i++) {
        Tlc.set(i, LED_DIM);
    }
    Tlc.update();
    delay(300);
    for (int i = 0; i <= 12; i++) {
        Tlc.set(i, LED_OFF);
    }
    Tlc.update();
    delay(300);
    */
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
