#include <stdint.h>
#include <SPI.h>
#include "button_debouncer.hpp"
#include "config.h"

extern "C" {
    #include "envelope.h"
}

ButtonDebouncer debouncer(BUTTON_PIN, cycleModes);

#define DEBUG false

void setup() {
    pinMode(GATE_IN_PIN, INPUT);
    pinMode(RETRIG_IN_PIN, INPUT);
    pinMode(BUTTON_PIN, INPUT_PULLUP);
    pinMode(DAC_CS_PIN, OUTPUT);
    digitalWrite(DAC_CS_PIN, HIGH);

    SPI.begin();
    SPI.setBitOrder(MSBFIRST);
    SPI.setDataMode(SPI_MODE0);
    
    #if GATE_PASSTHROUGH_ENABLED
    pinMode(GATE_OUT_PIN, OUTPUT);
    #endif

    #if LED_MODE_INDICATOR_ENABLED
    pinMode(LED_MODE_INDICATOR_PIN, OUTPUT);
    digitalWrite(LED_MODE_INDICATOR_PIN, HIGH);
    #endif

    #if EOR_TRIGGER_ENABLED
    pinMode(EOR_TRIGGER_PIN, OUTPUT);
    #endif

    #if EOF_TRIGGER_ENABLED
    pinMode(EOF_TRIGGER_PIN, OUTPUT);
    #endif

    pinMode(CV_PIN_A, INPUT);
    pinMode(CV_PIN_D, INPUT);
    pinMode(CV_PIN_S, INPUT);
    pinMode(CV_PIN_R, INPUT);

    for (int i = 0; i < 4; i++) {
        pinMode(LED_PINS[i], OUTPUT);
    }
    digitalWrite(LED_PINS[DEFAULT_MODE], HIGH);

    #if DEBUG
    Serial.begin(9600);
    #endif
}

uint32_t currentTimeMicros = 0;
void loop() {
    currentTimeMicros = micros();
    float value = update(currentTimeMicros);
    #if DEBUG
    Serial.println(value);
    #endif
    MCP4922_write(DAC_CS_PIN, 0, value);
    MCP4922_write(DAC_CS_PIN, 1, 1 - value);
    updateLEDs();
    
    debouncer.loop(currentTimeMicros);

    {
        static bool oldGate = false;
        const bool newGate = !digitalRead(GATE_IN_PIN);
        if (newGate != oldGate) {
            gate(newGate);
        }
        oldGate = newGate; 
    }

    {
        static bool oldTrig = false;
        const bool newTrig = !digitalRead(RETRIG_IN_PIN);
        if (newTrig && !oldTrig) {
            ping();
        }
        oldTrig = newTrig;
    }
    
    {
        static bool oldButton = false;
        const bool newButton = digitalRead(BUTTON_PIN);
        if (newButton != oldButton) {
            debouncer.pinChanged(currentTimeMicros, newButton);
        }
        oldButton = newButton; 
    }
}

inline void handleGateChange(int8_t change) {
    const bool gateValue = change < 0;
    #if GATE_PASSTHROUGH_ENABLED
    digitalWrite(GATE_OUT_PIN, gateValue);
    #endif
    gate(gateValue);
}

inline void enableInterrupt(byte pin) {
    *digitalPinToPCMSK(pin) |= bit (digitalPinToPCMSKbit(pin));  // enable pin
    PCIFR  |= bit (digitalPinToPCICRbit(pin)); // clear any outstanding interrupt
    PCICR  |= bit (digitalPinToPCICRbit(pin)); // enable interrupt for the group
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
