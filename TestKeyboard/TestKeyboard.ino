#include <cstdint>

const int NUM_KEYS = 12;

const uint16_t KEY_PINS[NUM_KEYS] = {
    D1,
    D2,
    D3,
    D4,
    D5,
    D6,
    D7,
    D8,
    D9,
    D10,
    D11,
    D12,
}

const uint16t LED_PIN = A1;
const uint16t GATE_PIN = A2;
const uint16t TRIG_PIN = A3;
const uint16t VPO_PIN = A3;

const bool KEYS_PRESSED[NUM_KEYS];

int currentKey = 1;

void setup() {
    for (int i = 0; i < NUM_KEYS; i++) {
        pinMode(KEY_PINS[i], INPUT);
        KEYS_PRESSED[i] = false;
    }

    pinMode(LED_PIN, OUTPUT);
    pinMode(GATE_PIN, OUTPUT);
    pinMode(TRIG_PIN, OUTPUT);
    pinMode(VPO_PIN, OUTPUT);
}

void loop() {
    int newPressedKey = -1;

    // Loop through all 12 keys
    for (int i = 0; i < NUM_KEYS; i++) {
        // Check if this key is being pressed right now
        bool keyIsPressed = digitalRead(KEY_PINS[i]);

        // If it *is* being pressed but *wasn't* being pressed last
        // time we checked, remember that
        if (keyIsPressed && !KEYS_PRESSED[i]) {
            newPressedKey = i;
        }

        // Update our reccord of which keys are currently pressed
        KEYS_PRESSED[i] = keyIsPressed;
    }
    
    // If 'newPressedKey' was changed, that means at least one key was pressed
    bool atLeastOneKeyPressed = (newPressedKey != -1);

    // If at least one key is being pressed, keep the gate open
    digitalWrite(GATE_PIN, atLeastOneKeyPressed);
    digitalWrite(LED_PIN, atLeastOneKeyPressed);
    
    // If the currently pressed key changed in this loop:
    if (atLeastOneKeyPressed && newPressedKey != currentKey) {
        currentKey = newPressedKey;
        //TODO update DAC
    }
}
