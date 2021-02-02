#ifndef led_driver_hpp_INCLUDED
#define led_driver_hpp_INCLUDED

#include <stdint.h>
#include "lib/MCP23S17.h"
#include <Arduino.h>

class LedDriver {
    public:
        LedDriver(uint8_t id, uint8_t pin);
        void setLEDs(uint16_t leds);
        void setLED(uint8_t led, bool state);
        void setup();
        void loop();
    private:
        void flashSingleLED(uint8_t i);
        void turnOffAllLEDs();
        MCP device;
        uint16_t ledsOn;
        bool ledIsOn = false;
};

LedDriver::LedDriver(uint8_t id, uint8_t pin) : device(id, pin) {
    this->ledsOn = 0;
}

void LedDriver::setLEDs(uint16_t leds) {
    this->ledsOn = leds;
}

void LedDriver::setLED(uint8_t led, bool state) {
    bitWrite(this->ledsOn, led, state);
}

void LedDriver::setup() {
    this->device.begin();
    device.pinMode(0xFFFF);
}

void LedDriver::loop() {
    for (uint8_t i = 0; i < 16; i++) {
        if (bitRead(ledsOn, i)) {
            flashSingleLED(i);
        } else {
            turnOffAllLEDs();
        }
        delay(1);
    }
    turnOffAllLEDs();
}

void printByte(uint8_t b) {
    for (int i = 7; i >= 0; i--) {
        Serial.print(bitRead(b, i));
    }
}

void LedDriver::flashSingleLED(uint8_t i) {
    //col = 0-3 (gray - ground)
    //row = 4-7 (green - 5v)
    uint16_t row = i / 4;
    uint16_t col = i % 4;
    uint16_t colPin = col + 8;
    uint16_t rowPin = row + 4 + 8;
    // 0 = output, 1 = input
    uint16_t pinModes = 0xFFFF;
    uint16_t colMask = ((uint16_t) 1) << colPin;
    uint16_t rowMask = ((uint16_t) 1) << rowPin;
    pinModes = pinModes ^ colMask;
    pinModes = pinModes ^ rowMask;
    this->device.pinMode(pinModes);
    this->device.digitalWrite(rowMask);
    this->ledIsOn = true;
}

inline void LedDriver::turnOffAllLEDs() {
    if (this->ledIsOn) {
        this->device.pinMode(0xFFFF);
        this->ledIsOn = false;
    }
}

#endif // led_driver_hpp_INCLUDED

