#ifndef button_debouncer_hpp_INCLUDED
#define button_debouncer_hpp_INCLUDED

#include <Arduino.h>

#define STATE_WAIT_KEY  0
#define STATE_CHECK_KEY 1
#define STATE_WAIT_KEY_RELEASE 2
class ButtonDebouncer {
    const static uint32_t DEBOUNCE_TIME = 30000; // micros
    byte state = STATE_WAIT_KEY;
    uint16_t pin;
    void (*callback)(void);
    uint32_t startKeyPressTime;
    public:
       ButtonDebouncer(const uint16_t pin, void (*callback)(void));
       void loop(const uint32_t currentTime); 
       void pinChanged(const uint32_t currentTime, const bool pinValue);
};

ButtonDebouncer::ButtonDebouncer(const uint16_t pin, void (*callback)(void)) {
    this->pin = pin;
    this->callback = callback;
}

void ButtonDebouncer::loop(const uint32_t currentTime) {
    if (state == STATE_CHECK_KEY && currentTime - startKeyPressTime >= DEBOUNCE_TIME) {
        bool buttonStillDown = !digitalRead(pin);
        if (buttonStillDown) {
            callback();
            state = STATE_WAIT_KEY_RELEASE;
        } else {
            state = STATE_WAIT_KEY;
        }
    }
}

void ButtonDebouncer::pinChanged(const uint32_t currentTime, const bool pinValue) {
    bool buttonDown = !pinValue;
    if (state == STATE_WAIT_KEY && buttonDown) {
        startKeyPressTime = currentTime;
        state = STATE_CHECK_KEY;
    } else if (state == STATE_WAIT_KEY_RELEASE && !buttonDown) {
        state = STATE_WAIT_KEY;
    } /*else if (state == STATE_CHECK_KEY && currentTime - startKeyPressTime >= DEBOUNCE_TIME) {
        if (buttonDown) {
            callback();
            state = STATE_WAIT_KEY_RELEASE;
        } else {
            state = STATE_WAIT_KEY;
        }
    }*/
}

#endif // button_debouncer_hpp_INCLUDED

