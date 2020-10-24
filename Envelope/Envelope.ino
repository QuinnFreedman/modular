#include <stdint.h>
#include "config.h"

extern "C" {
    #include "envelope.h"
}

void setup() {
    pinMode(GATE_IN_PIN, INPUT_PULLUP);
    pinMode(RETRIG_IN_PIN, INPUT_PULLUP);
    pinMode(BUTTON_PIN, INPUT_PULLUP);
    
    #if GATE_PASSTHROUGH_ENABLED
    pinMode(GATE_OUT_PIN, OUTPUT);
    #endif

    #if LED_MODE_INDICATOR_ENABLED
    pinMode(LED_MODE_INDICATOR_PIN, OUTPUT);
    digitalWrite(LED_MODE_INDICATOR_PIN, HIGH);
    #endif

    attachInterrupt(digitalPinToInterrupt(GATE_IN_PIN), handleGateChange, CHANGE);
    attachInterrupt(digitalPinToInterrupt(RETRIG_IN_PIN), handleRetrigChange, CHANGE);
    enableInterrupt(BUTTON_PIN);

    pinMode(CV_PIN_A, INPUT);
    pinMode(CV_PIN_D, INPUT);
    pinMode(CV_PIN_S, INPUT);
    pinMode(CV_PIN_R, INPUT);

    for (int i = 0; i < 4; i++) {
        pinMode(LED_PINS[i], OUTPUT);
    }
    digitalWrite(LED_PINS[DEFAULT_MODE], HIGH);

    Serial.begin(9600);
}

void loop() {
    uint32_t currentTime = micros();
    float value = update(currentTime);
    Serial.println(value);
}

#if BUTTON_PIN >= 0 && BUTTON_PIN <= 7
#define PIN_VEC PCINT2_vect
#elif BUTTON_PIN >= 8 && BUTTON_PIN <= 13
#define PIN_VEC PCINT0_vect
#else
#define PIN_VEC PCINT1_vect
#endif
ISR(PIN_VEC) {
    handleButtonPress();
}

void handleGateChange() {
    static uint16_t lastValue = digitalRead(GATE_IN_PIN);
    uint16_t currentValue = digitalRead(GATE_IN_PIN);
    if (currentValue == LOW && lastValue == HIGH) {
        #if GATE_PASSTHROUGH_ENABLED
        digitalWrite(GATE_OUT_PIN, true);
        #endif
        gate(true);
    } else if (lastValue == LOW && currentValue == HIGH) {
        #if GATE_PASSTHROUGH_ENABLED
        digitalWrite(GATE_OUT_PIN, false);
        #endif
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

inline void handleButtonPress() {
    static uint16_t lastValue = digitalRead(BUTTON_PIN);
    uint16_t currentValue = digitalRead(BUTTON_PIN);
    if (currentValue == LOW && lastValue == HIGH) {
        cycleModes();
    }
    lastValue = currentValue;
}

inline void enableInterrupt(byte pin) {
    *digitalPinToPCMSK(pin) |= bit (digitalPinToPCMSKbit(pin));  // enable pin
    PCIFR  |= bit (digitalPinToPCICRbit(pin)); // clear any outstanding interrupt
    PCICR  |= bit (digitalPinToPCICRbit(pin)); // enable interrupt for the group
}
