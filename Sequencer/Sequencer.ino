#include <stdint.h>
#include "src/lib/MCP23S17.h"
#include "src/led_driver.hpp"

// const uint16_t COL_GROUND_PINS[] = {5, 6, 7, 8};
// const uint16_t ROW_LED_PINS[] = {A2, A3, A4, A5};

const uint16_t INTERRUPT_PIN_A = 2;
const uint16_t INTERRUPT_PIN_B = 3;
const uint16_t POT_ADDR_PINS[] = {4, 5, 6};//{6, 5, 4};
const uint16_t POT_ANALOG_PINS[]  = {A7, A6};
const uint16_t POT_ADDRESSES[16] = {2, 5, 8|2, 8|5,
                                    1, 7, 8|1, 8|7,
                                    0, 6, 8|0, 8|6,
                                    3, 4, 8|3, 8|4};

const uint16_t TRIGGER_BANK_CS_PIN = 7;
const uint16_t LED_DRIVER_CS_PIN = 8;

MCP triggerBank(0, TRIGGER_BANK_CS_PIN);
LedDriver ledDriver(1, LED_DRIVER_CS_PIN);

void setup() {
    pinMode(10, OUTPUT);
    for (int i = 0; i < 3; i++) {
        pinMode(POT_ADDR_PINS[i], OUTPUT);
    }

    for (int i = 0; i < 2; i++) {
        pinMode(POT_ANALOG_PINS[i], INPUT);
    }

    Serial.begin(9600);

    triggerBank.begin();
    ledDriver.setup();

    /*
    for (int i = 1; i <= 16; i++) {
       triggerBank.pinMode(i, INPUT);
       triggerBank.pullupMode(i, true);
       triggerBank.inputInvert(i, false);
    }
    */

    const uint8_t INTPOL = 0b00000010;
    const uint8_t ORD    = 0b00000100;
    const uint8_t HAEN   = 0b00001000;
    const uint8_t DISSLW = 0b00010000;
    const uint8_t SEQOP  = 0b00100000;
    const uint8_t MIRROR = 0b01000000;
    const uint8_t BANK   = 0b10000000;
    const uint8_t config = HAEN; //MIRROR;// | INTPOL;
    triggerBank.byteWrite(0x0A, config);
    triggerBank.byteWrite(0x0B, config);
    
    triggerBank.pinMode(0xFFFF);
    triggerBank.pullupMode(0xFFFF);
    triggerBank.inputInvert(0x0000);
    
    // enable interrupts
    triggerBank.byteWrite(0x04, 0xFF); // GPINTENA
    triggerBank.byteWrite(0x05, 0xFF); // GPINTENB
    // interrupt refference value
    triggerBank.byteWrite(0x06, 0x00); // DEFVALA
    triggerBank.byteWrite(0x07, 0x00); // DEFVALB
    // interrupt condition (change vs high/low)
    triggerBank.byteWrite(0x08, 0x00); // INTCONA 
    triggerBank.byteWrite(0x09, 0x00); // INTCONB 

    pinMode(INTERRUPT_PIN_A, INPUT);
    pinMode(INTERRUPT_PIN_B, INPUT);
    delay(100);
    pinMode(INTERRUPT_PIN_A, INPUT_PULLUP);
    pinMode(INTERRUPT_PIN_B, INPUT_PULLUP);
    attachInterrupt(digitalPinToInterrupt(INTERRUPT_PIN_A), onIOExpanderInterruptA, FALLING);
    attachInterrupt(digitalPinToInterrupt(INTERRUPT_PIN_B), onIOExpanderInterruptB, FALLING);
    
}

#define getBit(byte, addr) ( (byte >> addr) & 1 )

float readPotValue(uint8_t pot) {
    uint8_t addr = POT_ADDRESSES[pot];
    uint8_t bank = getBit(addr, 3);
    pot &= 0b00000111;
    for (uint8_t i = 0; i < 3; i++) {
        digitalWrite(POT_ADDR_PINS[i], getBit(addr, i));
    }
    delay(10);
    return analogRead(POT_ANALOG_PINS[bank]) / 1024.0;
}

void onIOExpanderInterruptA() {
    static uint8_t lastValue = 0xFF;
    uint8_t value = (triggerBank.digitalRead() >> 8);
    if (value == lastValue) return; 
    
    lastValue = value;

    Serial.print("A: ");
    for (int i = 0; i < 8; i++) {
        Serial.print(getBit(value, i));
        Serial.print(" ");
    }
    Serial.println();
}

void onIOExpanderInterruptB() {
    static uint8_t lastValue = 0xFF;
    uint8_t value = triggerBank.digitalRead();
    if (value == lastValue) return; 
    
    lastValue = value;

    Serial.print("B: ");
    for (int i = 0; i < 8; i++) {
        Serial.print(getBit(value, i));
        Serial.print(" ");
    }
    Serial.println();
}


void loop() {
    static uint8_t led = 0;
    static uint32_t lastChange = 0;

    ledDriver.setLED(0);
    ledDriver.setLED(led, HIGH);
    ledDriver.loop();
    
    uint32_t now = millis();
    if (now - lastChange > 1000) {
        lastChange += 1000;
        led++;
        if (led >= 8) { led = 0; }
    }
}
