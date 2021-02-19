#include <stdint.h>
#include <SPI.h>
#include "src/lib/MCP23S17.h"
#include "src/led_driver.hpp"
#include "src/sequencer.hpp"

// const uint16_t COL_GROUND_PINS[] = {5, 6, 7, 8};
// const uint16_t ROW_LED_PINS[] = {A2, A3, A4, A5};

const uint16_t INTERRUPT_PIN_A = 3;
const uint16_t INTERRUPT_PIN_B = 2;
const uint16_t POT_ADDR_PINS[] = {4, 5, 6};//{6, 5, 4};
const uint16_t POT_ANALOG_PINS[]  = {A7, A6};
const uint16_t POT_ADDRESSES[16] = {2, 5, 8|2, 8|5,
                                    1, 7, 8|1, 8|7,
                                    0, 6, 8|0, 8|6,
                                    3, 4, 8|3, 8|4};

const uint16_t TRIGGER_BANK_CS_PIN = 7;
const uint16_t LED_DRIVER_CS_PIN = 8;
const uint16_t DAC_CS_PIN_A = 10;
const uint16_t DAC_CS_PIN_B = 9;

MCP triggerBank(0, TRIGGER_BANK_CS_PIN);
LedDriver ledDriver(1, LED_DRIVER_CS_PIN);
Sequencer sequencer;

void setup() {
    pinMode(DAC_CS_PIN_A, OUTPUT);
    pinMode(DAC_CS_PIN_B, OUTPUT);
    
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

void onIOExpanderInterruptB() {
    static uint8_t lastValue = 0;
    uint8_t value = ~(triggerBank.digitalRead() >> 8);
    if (value == lastValue) return; 
    
    uint8_t changed = lastValue ^ value;
    
    lastValue = value;
    
    if (changed == 0) return;

    /*
    Serial.print("B: ");
    for (int i = 0; i < 8; i++) {
        Serial.print(getBit(value, i));
        Serial.print(" ");
    }
    Serial.println();
    */

    if (changed >> 6) {
        uint8_t switchState = value >> 6;
        switch (switchState) {
            case 0: {
                Serial.println("middle");
            } break;
            case 1: {
                Serial.println("right");
            } break;
            case 2: {
                Serial.println("left");
            } break;
        }
    }
    
    if ((changed >> 1) & 0b11111) {
        uint8_t modeSelectState = (value >> 1) & 0b11111;
        uint8_t modeSelectNumber = 0;
        for (uint8_t i = 0; i < 5; i++) {
            modeSelectNumber += ((modeSelectState >> i) & 1) * (i + 1);
        }
        modeSelectNumber = 5 - modeSelectNumber;
        Serial.print("Mode select: ");
        Serial.println(modeSelectNumber);
    }
}

void onIOExpanderInterruptA() {
    static uint8_t lastValue = 0;
    uint8_t value = ~triggerBank.digitalRead();
    if (value == lastValue) return; 

    uint8_t diff = lastValue ^ value;
    uint8_t newRising = diff & value;
    
    lastValue = value;

    if (newRising == 0) return;

    /*
    Serial.print("A: ");
    for (int i = 0; i < 8; i++) {
        Serial.print(getBit(newRising, i));
        Serial.print(" ");
    }
    Serial.println();
    */

    uint8_t channelBits = newRising >> 4;
    if (channelBits) {
        bool channels[4] = {
            (channelBits >> 0) & 1,
            (channelBits >> 1) & 1,
            (channelBits >> 2) & 1,
            (channelBits >> 3) & 1,
        };
        sequencer.advance(channels);
    }

    if (getBit(newRising, 3)) {
        // Serial.println("Jump 1");
        sequencer.jump(0);
    } else if (getBit(newRising, 2)) {
        // Serial.println("Jump 2");
        sequencer.jump(1);
    } else if (getBit(newRising, 1)) {
        // Serial.println("Jump 3");
        sequencer.jump(2);
    } else if (getBit(newRising, 0)) {
        // Serial.println("Jump 4");
        sequencer.jump(3);
    }
}

/*

B7: RANDOM
A7: STEP 4
A6: STEP 3
A5: STEP 2
A4: STEP 1
A3: JUMP 1
A2: JUMP 2
A1: JUMP 3
A0: JUMP 4

 */

inline uint16_t arrayToUint16(bool * data) {
    uint16_t result = 0;
    for (uint8_t i = 0; i < 16; i++) {
        result = result | (data[i] << i);
    }
    return result;
}

void loop() {
    uint16_t leds = arrayToUint16(sequencer.getLedState());
    ledDriver.setLEDs(leds);
    noInterrupts();
    ledDriver.loop();
    interrupts();
    /*
    static uint8_t led = 0;
    static uint32_t lastChange = 0;

    ledDriver.setLEDs(0);
    ledDriver.setLED(led, HIGH);
    ledDriver.loop();
    
    uint32_t now = millis();
    if (now - lastChange > 1000) {
        lastChange += 1000;
        //led = random(0, 16);
        led++;
        if (led >= 16) { led = 0; }
    }
    return;
    MCP4922_write(DAC_CS_PIN_A, 0, 1);
    MCP4922_write(DAC_CS_PIN_A, 1, 1);
    MCP4922_write(DAC_CS_PIN_B, 0, 1);
    MCP4922_write(DAC_CS_PIN_B, 1, 1);
    delay(1000);
    MCP4922_write(DAC_CS_PIN_A, 0, 0);
    MCP4922_write(DAC_CS_PIN_A, 1, 0);
    MCP4922_write(DAC_CS_PIN_B, 0, 0);
    MCP4922_write(DAC_CS_PIN_B, 1, 0);
    delay(1000);
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
