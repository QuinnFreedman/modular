#include <stdint.h>

const int NUM_KEYS = 12;

const uint16_t KEY_PINS[NUM_KEYS] = {
    1,
    2,
    3,
    4,
    5,
    6,
    7,
    8,
    9,
    10,
    11,
    12,
};

const uint16_t LED_PIN = A2;
const uint16_t GATE_PIN = A1;
const uint16_t TRIG_PIN = A0;

bool KEYS_PRESSED[NUM_KEYS];

int currentKey = 1;

void setup() {
  Serial.begin(9600);
    for (int i = 0; i < NUM_KEYS; i++) {
        pinMode(KEY_PINS[i], INPUT);
        KEYS_PRESSED[i] = false;
    }

    pinMode(LED_PIN, OUTPUT);
    pinMode(GATE_PIN, OUTPUT);
    pinMode(TRIG_PIN, OUTPUT);
}

void loop() {
    int newPressedKey = -1;

    // Loop through all 12 keys
    for (int i = 0; i < NUM_KEYS; i++) {
        // Check if this key is being pressed right now
        bool keyIsPressed = !digitalRead(KEY_PINS[i]);

        // If it *is* being pressed but *wasn't* being pressed last
        // time we checked, remember that
        if (keyIsPressed && !KEYS_PRESSED[i]) {
            Serial.print("Key ");
            Serial.println(i);
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
