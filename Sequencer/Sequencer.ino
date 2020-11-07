#include <stdint.h>
#include "src/lib/MCP23S17.h"

// const uint16_t COL_GROUND_PINS[] = {5, 6, 7, 8};
// const uint16_t ROW_LED_PINS[] = {A2, A3, A4, A5};

const uint16_t INTERRUPT_PIN_A = 2;
const uint16_t INTERRUPT_PIN_B = 3;
const uint16_t POT_ADDR_PINS[] = {6, 5, 4};
const uint16_t POT_ANALOG_PINS[]  = {A6, A7};
const uint16_t POT_ADDRESSES[2][8] = {{5, 7, 6, 4, 0, 0, 0, 0},
                                      {4, 4, 4, 4, 0, 0, 0, 0}};

// volatile bool ledValues[4][4] = {{HIGH, HIGH, HIGH, HIGH}, {HIGH, HIGH, HIGH, HIGH}};

MCP io(0, A0);

void setup() {
    for (int i = 0; i < 3; i++) {
        pinMode(POT_ADDR_PINS[i], OUTPUT);
    }

    for (int i = 0; i < 2; i++) {
        pinMode(POT_ANALOG_PINS[i], INPUT);
    }

    Serial.begin(9600);
    io.begin();

    /*
    for (int i = 1; i <= 16; i++) {
       io.pinMode(i, INPUT);
       io.pullupMode(i, true);
       io.inputInvert(i, false);
    }
    */
    io.pinMode(0xFFFF);
    io.pullupMode(0xFFFF);
    io.inputInvert(0x0000);
    
    // enable interrupts
    io.byteWrite(0x04, 0xFF); // GPINTENA
    io.byteWrite(0x05, 0xFF); // GPINTENB
    // interrupt refference value
    io.byteWrite(0x06, 0x00); // DEFVALA
    io.byteWrite(0x07, 0x00); // DEFVALB
    // interrupt condition (change vs high/low)
    io.byteWrite(0x08, 0x00); // INTCONA 
    io.byteWrite(0x09, 0x00); // INTCONB 
    const uint8_t INTPOL = 0b00000010;
    const uint8_t ORD    = 0b00000100;
    const uint8_t HAEN   = 0b00001000;
    const uint8_t DISSLW = 0b00010000;
    const uint8_t SEQOP  = 0b00100000;
    const uint8_t MIRROR = 0b01000000;
    const uint8_t BANK   = 0b10000000;
    const uint8_t config = 0; //MIRROR;// | INTPOL;
    io.byteWrite(0x0A, config);
    io.byteWrite(0x0B, config);

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
    uint8_t bank = getBit(pot, 3);
    pot &= 0b00000111;
    uint8_t addr = POT_ADDRESSES[bank][pot];
    for (uint8_t i = 0; i < 3; i++) {
        digitalWrite(POT_ADDR_PINS[i], getBit(addr, i));
    }
    delay(10);
    return analogRead(POT_ANALOG_PINS[bank]) / 1024.0;
}

void onIOExpanderInterruptA() {
    static uint16_t lastValue = 0xFFFF;
    uint16_t value = io.digitalRead();
    if (value == lastValue) return; 
    
    lastValue = value;

    Serial.print("A: ");
    for (int i = 0; i < 16; i++) {
        Serial.print(getBit(value, i));
        Serial.print(i == 7 ? "  " : " ");
    }
    Serial.println();
}

void onIOExpanderInterruptB() {
    static uint16_t lastValue = 0xFFFF;
    uint16_t value = io.digitalRead();
    if (value == lastValue) return; 
    
    lastValue = value;

    Serial.print("B: ");
    for (int i = 0; i < 16; i++) {
        Serial.print(getBit(value, i));
        Serial.print(i == 7 ? "  " : " ");
    }
    Serial.println();
}

void loop() {
    /*
    Serial.print(readPotValue(0));
    Serial.print(" ");
    Serial.print(readPotValue(1));
    Serial.print(" ");
    Serial.print(readPotValue(2));
    Serial.print(" ");
    Serial.print(readPotValue(3));
    Serial.print(" ");
    Serial.print(readPotValue(8));
    Serial.println();
    */

    /*
    for (int col = 0; col < 4; col++) {
        pinMode(COL_GROUND_PINS[col], OUTPUT);
        digitalWrite(COL_GROUND_PINS[col], LOW);
        for (int row = 0; row < 4; row++) {
            digitalWrite(ROW_LED_PINS[row], ledValues[col][row]);
            delay(1);
            digitalWrite(ROW_LED_PINS[row], LOW);
        }
        pinMode(COL_GROUND_PINS[col], INPUT);
    }
    */
    // for (int i = 9; i <=16; i++) {
    //     io.digitalWrite(i, value);
    // }
}
