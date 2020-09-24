#include <stdint.h>
#include <SPI.h>

const int NUM_KEYS = 12;

const int KEY_PINS[NUM_KEYS] = {
  A0,
  9,
  8,
  7,
  6,
  5,
  4,
  3,
  2,
  A4,
  A1,
  A2
};

const uint16_t LED_PIN = A3;
const uint16_t GATE_PIN = 10;
const uint16_t TRIG_PIN = A0;
const uint16_t CHIP_SELECT_PIN = A5;
const uint16_t OCTAVE_SELECT_POT_PIN = A6;
const uint16_t FINE_TUNE_POT_PIN = A7;

const float FINE_TUNE_SEMITONES = 6;

bool KEYS_PRESSED[NUM_KEYS];

int currentKey = -1;

void setup() {
  pinMode(CHIP_SELECT_PIN, OUTPUT);
  digitalWrite(CHIP_SELECT_PIN, HIGH);
  
  pinMode(LED_PIN, OUTPUT);
  pinMode(GATE_PIN, OUTPUT);
  pinMode(TRIG_PIN, OUTPUT);
  /* pinMode(OCTAVE_SELECT_POT_PIN, INPUT); */
  /* pinMode(FINE_TUNE_POT_PIN, INPUT); */
  
  for (int i = 0; i < NUM_KEYS; i++) {
    if (KEY_PINS[i] != -1) {
      pinMode(KEY_PINS[i], INPUT);
    }
    KEYS_PRESSED[i] = false;
  }

  /* Serial.begin(9600); */

  SPI.begin();
  SPI.setBitOrder(MSBFIRST);
  SPI.setDataMode(SPI_MODE0);
}

void loop() {
  int newPressedKey = -1;
  bool atLeastOneKeyDown = false;

  // Loop through all 12 keys
  for (int i = 0; i < NUM_KEYS; i++) {
    if (KEY_PINS[i] == -1) continue;

    bool keyIsPressed = digitalRead(KEY_PINS[i]) == HIGH;
    bool keyWasDown = KEYS_PRESSED[i];

    // If the key is pressed this loop but wasn't pressed last time, record that
    if (keyIsPressed && !keyWasDown) {
      newPressedKey = i;
    }

    // update whether the key is down
    KEYS_PRESSED[i] = keyIsPressed;
    // Keep track if at least one key is currently being held (for gate)
    atLeastOneKeyDown |= keyIsPressed;
  }

  // If at least one key is being held down, keep the gate open
  int gateValue = atLeastOneKeyDown ? HIGH : LOW;
  digitalWrite(GATE_PIN, gateValue);
  digitalWrite(LED_PIN, gateValue);

  // If 'newPressedKey' was changed, that means at least one key was pressed
  bool atLeastOneKeyPressed = (newPressedKey != -1);

  int octave = ((float) analogRead(OCTAVE_SELECT_POT_PIN)) / ((float) 1024) * 5;

  // If the currently pressed key changed in this loop:
  if (atLeastOneKeyPressed) {
    currentKey = newPressedKey;
    float oneVoltDACValue = 4096.0 / 5.0;
    float octaveMinDACValue = oneVoltDACValue * (float) octave;
    float oneSemitoneDACValue = oneVoltDACValue / 12.0;
    float dacValue = octaveMinDACValue + oneSemitoneDACValue * newPressedKey;
    uint16_t dacUint16 = round(dacValue);
    MCP4922_write(CHIP_SELECT_PIN, 0, dacUint16);
  }
}

void MCP4922_write(int cs_pin, byte dac, uint16_t value) {
  byte low = value & 0xff;
  byte high = (value >> 8) & 0x0f;
  dac = (dac & 1) << 7;
  digitalWrite(cs_pin, LOW);
  delay(100);
  SPI.transfer(dac | 0x30 | high);
  SPI.transfer(low);
  delay(100);
  digitalWrite(cs_pin, HIGH);
}
