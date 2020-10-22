#include <stdint.h>

extern "C" {
    #include "envelope.h"
}

const uint16_t GATE_IN_PIN = 2;
const uint16_t RETRIG_IN_PIN = 3;

void setup() {
    pinMode(GATE_IN_PIN, INPUT_PULLUP);
    pinMode(RETRIG_IN_PIN, INPUT_PULLUP);
    pinMode(13, OUTPUT);

    attachInterrupt(digitalPinToInterrupt(GATE_IN_PIN), handleGateChange, CHANGE);
    attachInterrupt(digitalPinToInterrupt(RETRIG_IN_PIN), handleRetrigChange, CHANGE);

    Serial.begin(9600);
}


void loop() {
    uint32_t currentTime = micros();
    float value = update(currentTime);
    Serial.println(value);
}

void handleGateChange() {
    static uint16_t lastValue = digitalRead(GATE_IN_PIN);
    uint16_t currentValue = digitalRead(GATE_IN_PIN);
    if (currentValue == LOW && lastValue == HIGH) {
        digitalWrite(13, true);
        gate(true);
    } else if (lastValue == LOW && currentValue == HIGH) {
        digitalWrite(13, false);
        gate(false);
    }
    lastValue = currentValue;
}

void handleRetrigChange() {
    static uint16_t lastValue = digitalRead(RETRIG_IN_PIN);
    uint16_t currentValue = digitalRead(RETRIG_IN_PIN);
    if (currentValue == LOW && lastValue == HIGH) {
        ping();
    }
    lastValue = currentValue;
}
