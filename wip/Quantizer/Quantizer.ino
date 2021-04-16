#include <stdint.h>
#include <EEPROM.h>
#include <SPI.h>
#include <Tlc5940.h>
#include "lib/digitalWriteFast.h"

// TODO: Input/output triggers. When triggerMode is TRIGGER_OUTPUT, a short trigger
//       should be sent to each trigger pin every time the note changes. If it is
//       TRIGGER_INPUT, the main loop should wait to look at the input until it recieves
//       a trigger on the trigger pins, effectively like a sample-and-hold. pinMode()
//       will need to be changed accordingly.
// TODO: Currently, I save the whole state to EEPROM every time it is updated. This is
//       very slow and uses up limited EEPROM writes. It would be pretty easy to only
//       update the part that is acutally changing each time.
// TODO: Show selected options when in menu mode.

#define SHOW_BOTH_CHANNELS true

const uint16_t LED_BRIGHT = (pow(2, 12) - 1) / 10;
const uint16_t LED_DIM = LED_BRIGHT / 10;
const uint16_t LED_OFF = 0;

const uint32_t LONG_PRESS_TIME_MILLIS = 500;
const float HYSTERESIS_THRESHOLD = 0.7;

const uint32_t TRIGGER_TIME_MS = 50;

const uint8_t BUTTON_LADDER_PIN = A0;
const uint8_t ANALOG_INPUT_PIN_A = A6;
const uint8_t ANALOG_INPUT_PIN_B = A7;
const uint8_t MENU_BUTTON_PIN = 2;
const uint8_t DAC_CS_PIN = 8;

const uint8_t TRIG_PIN_A = 4;
const uint8_t TRIG_PIN_B = 5;

const uint16_t ANALOG_READ_MAX_VALUE = 1023;

const uint8_t LED_INDEX[12] = {12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1};
const bool SCALES[4][12] = {
    {1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1}, // chromatic
    {1, 0, 1, 0, 1, 1, 0, 1, 0, 1, 0, 1}, // major
    {1, 0, 1, 1, 0, 1, 0, 1, 1, 0, 1, 0}, // minor
    {1, 0, 1, 0, 1, 0, 0, 1, 0, 1, 0, 0}, // pentatonic
};

enum TriggerMode {TRIGGER_OUTPUT = false, TRIGGER_INPUT = true };

float BUTTON_LADDER_CUTOFFS[12];

bool showMenu = false;

typedef struct {
    bool notes[12];
    TriggerMode triggers[2];
    bool userProfiles[4][12];
} State;

State state = {
    notes: {1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1},
    triggers: {TRIGGER_OUTPUT, TRIGGER_OUTPUT},
    userProfiles: {
        {0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0},
        {0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0},
        {0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0},
        {0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0},
    },
};

bool SHOULD_UPDATE_UI = true;

void initButtonCutoffs() {
    for (uint8_t i = 0; i < 12; i++) {
        BUTTON_LADDER_CUTOFFS[i] = 1 / (((float) i) + 1.5);
    }
}

void setup() {
    pinMode(BUTTON_LADDER_PIN, INPUT);
    pinMode(DAC_CS_PIN, OUTPUT);
    pinMode(ANALOG_INPUT_PIN_A, INPUT);
    pinMode(ANALOG_INPUT_PIN_B, INPUT);
    pinMode(TRIG_PIN_A, OUTPUT);
    pinMode(TRIG_PIN_B, OUTPUT);
    pinMode(MENU_BUTTON_PIN, INPUT_PULLUP);

    Tlc.init();
    Tlc.clear();
    delay(10);
    Tlc.update();
    
    SPI.begin();
    SPI.setBitOrder(MSBFIRST);
    SPI.setDataMode(SPI_MODE0);
    
    Serial.begin(9600);

    initButtonCutoffs();

    if (EEPROM.read(0) != 255) {
        Serial.println("Reading state from EEPROM");
        EEPROM.get(0, state);
    }
}

inline int16_t modulo(int16_t x, int16_t m) {
    int16_t mod = x % m;
    if (mod < 0) {
        mod += m;
    }
    return mod;
}

int16_t mod12(int16_t x) { return modulo(x, 12); }

float readInputVoltage(uint8_t pin) {
    uint16_t rawValue = analogRead(pin);
    return (rawValue / (float) ANALOG_READ_MAX_VALUE) * 20.0 - 10.0;
}

void writeOutputVoltage(uint8_t csPin, uint8_t dacChannel, float voltage) {
    const float minVoltage = -10;
    const float maxVoltage = 10;
    float dacOutput = (voltage - minVoltage) / (maxVoltage - minVoltage);
    MCP4922_write(csPin, dacChannel, dacOutput);
}

bool allSame(int8_t* buffer, uint8_t bufferSize, int8_t* value) {
    int8_t first = buffer[0];
    for (uint8_t i = 1; i < bufferSize; i++) {
        if (buffer[i] != first) return false;
    }
    *value = first;
    return true;
}

inline void handleButtons() {
    const uint32_t debounceTimeMilis = 3;
    const uint8_t SHIFT_REGISTER_SIZE = 4;
    static int8_t lastStableButton = -1;
    static bool stable = true;
    static uint32_t stabilizingStartTime = 0;
    static uint32_t buttonPressStartTime = 0;
    static uint8_t shiftRegisterPtr = 0;
    static int8_t shiftRegister[SHIFT_REGISTER_SIZE] = {-1, -1, -1, -1};
    
    const float buttonValue = analogRead(BUTTON_LADDER_PIN) / 1023.0;
    int8_t buttonDown = -1;
    for (uint8_t i = 0; i < 12; i++) {
        if (buttonValue > BUTTON_LADDER_CUTOFFS[i]) {
            buttonDown = i;
            break;
        }
    }

    const uint32_t now = millis();

    if (stable && buttonDown != lastStableButton) {
        // we might be pressing a new button -- start to stabalize
        shiftRegister[shiftRegisterPtr] = buttonDown;
        shiftRegisterPtr = (shiftRegisterPtr + 1) % SHIFT_REGISTER_SIZE;
        stable = false;
        stabilizingStartTime = now;
        if (buttonDown != -1) {
            buttonPressStartTime = now;
        }
    }

    else if (!stable && now - stabilizingStartTime > debounceTimeMilis) {
        shiftRegister[shiftRegisterPtr] = buttonDown;
        shiftRegisterPtr = (shiftRegisterPtr + 1) % SHIFT_REGISTER_SIZE;
        stabilizingStartTime = now;
        if (allSame(shiftRegister, SHIFT_REGISTER_SIZE, &buttonDown)) {
            stable = true;
            if (buttonDown != lastStableButton) {
                if (buttonDown == -1) {
                    if (showMenu) {
                        onButtonReleasedMenu(lastStableButton, now - buttonPressStartTime);
                    } else {
                        onButtonReleasedNormal(lastStableButton, now - buttonPressStartTime);
                    }
                } else {
                    if (showMenu) {
                        onButtonPressedMenu(buttonDown);
                    } else {
                        onButtonPressedNormal(buttonDown);
                    }
                }
            }
            lastStableButton = buttonDown;
        }
    }
}

void onButtonPressedNormal(int8_t button) {
    Serial.print("buttonPressed normal ");
    Serial.println(button);
    state.notes[button] = !state.notes[button];
    SHOULD_UPDATE_UI = true;
    EEPROM.put(0, state);
}

void onButtonReleasedNormal(int8_t button, uint32_t time) {
    Serial.print("buttonReleased normal ");
    Serial.print(button);
    Serial.print(" ");
    Serial.println(time);
}

void onButtonPressedMenu(int8_t button) {
    Serial.print("buttonPressed menu ");
    Serial.println(button);
    switch (button) {
    case 0:
    case 1: 
        state.triggers[button] = (TriggerMode) !state.triggers[button];
        EEPROM.put(0, state);
        break;
    case 2:
        transpose(1);
        break;
    case 3:
        transpose(-1);
        break;
    case 4:
    case 5:
    case 6:
    case 7:
        loadScale(SCALES[button - 4]);
        break;
    
    default:
        break;
    }
    SHOULD_UPDATE_UI = true;
}

void onButtonReleasedMenu(int8_t button, uint32_t time) {
    Serial.print("buttonReleased menu ");
    Serial.print(button);
    Serial.print(" ");
    Serial.println(time);
    button -= 8;
    if (button >= 0 && button < 4) {
        if (time < LONG_PRESS_TIME_MILLIS) {
            loadScale(state.userProfiles[button]);
        } else {
            for (int8_t i = 0; i < 12; i++) {
                state.userProfiles[button][i] = state.notes[i];
            }
            // const size_t offset = (size_t) &(state.userProfiles[button]) - (size_t) &state;
            EEPROM.put(0, state);
        }
    }
}

void loadScale(const bool * from) {
    for (int8_t i = 0; i < 12; i++) {
        state.notes[i] = from[i];
    }
    EEPROM.put(0, state);
}

void transpose(int8_t delta) {
    bool notes[12];
    for (int8_t i = 0; i < 12; i++) {
        notes[i] = state.notes[mod12(i + delta)];
    }
    loadScale(notes);
}

inline int8_t quantizeHelper(float semitones) {
    const int8_t semitonesInt = round(semitones);
    if (state.notes[mod12(semitonesInt)]) {
        return semitonesInt;
    }
    
    for (int8_t i = 0; i < 12; i++) {
        if (state.notes[mod12(semitonesInt + i)]) {
            return semitonesInt + i;
        }
        if (state.notes[mod12(semitonesInt - i)]) {
            return semitonesInt - i;
        }
    }

    return -127;
}
 
const int8_t A4_SEMITONES = 33;
int8_t quantizeNote(float inputVoltage, int8_t lastNote) {
    const float semitones = inputVoltage * 12.0f - A4_SEMITONES;
    const int8_t target = quantizeHelper(semitones);
    const float deltaCutoff = abs((target - lastNote) / 2.0);
    const float delta = abs(semitones - lastNote);
    if (delta > deltaCutoff + HYSTERESIS_THRESHOLD) {
        return target;
    }
    return lastNote;
}

float semitonesToVoltage(int8_t semitones) {
    return (semitones + A4_SEMITONES) / 12.0f;
}

void loop() {
    {
        const bool menuButton = !digitalReadFast(MENU_BUTTON_PIN);
        SHOULD_UPDATE_UI |= menuButton != showMenu;
        showMenu = menuButton;
    }
    handleButtons();

    bool activeNotes[12] = {0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0};

    {
        static float inputVoltageBuffer[2] = {0, 0};
        static int8_t outputNotes[2] = {0, 0};
        static uint32_t lastTriggerTime[2] = {0, 0};
        const uint8_t INPUT_PINS[2] = {ANALOG_INPUT_PIN_A, ANALOG_INPUT_PIN_B};
        const uint8_t TRIGGER_PINS[2] = {TRIG_PIN_A, TRIG_PIN_B};
        const uint32_t now = millis();
        
        for (uint8_t i = 0; i < 2; i++) {
            float inputVoltage = readInputVoltage(INPUT_PINS[i]);
            inputVoltageBuffer[i] = (inputVoltageBuffer[i] + inputVoltage) / 2.0;
            inputVoltage = inputVoltageBuffer[i];
            int8_t semitones = quantizeNote(inputVoltage, outputNotes[i]);
            float outputVoltage = semitonesToVoltage(semitones);
            if (semitones != outputNotes[i]) {
                outputNotes[i] = semitones;
                writeOutputVoltage(DAC_CS_PIN, i, outputVoltage);
                lastTriggerTime[i] = now;
                digitalWrite(TRIGGER_PINS[i], HIGH);
                #if !SHOW_BOTH_CHANNELS
                if (i == 0)
                #endif
                    activeNotes[mod12(semitones)] = true;
                
                SHOULD_UPDATE_UI = true;
            }
            if (i == 1) {
                Serial.print(outputNotes[1]);
                Serial.print(",");
                Serial.print(inputVoltage * 12.0f - A4_SEMITONES);
                Serial.println();
            }
        }
    }

    //TODO turn of trigger

    if (SHOULD_UPDATE_UI) {
        delay(1);
        SHOULD_UPDATE_UI = false;
        if (showMenu) {
            for (int i = 0; i <= 12; i++) {
                Tlc.set(LED_INDEX[i], LED_OFF);
            }
            Tlc.update();
        } else {
            for (int i = 0; i <= 12; i++) {
                Tlc.set(LED_INDEX[i],
                    activeNotes[i] ? LED_BRIGHT :
                    state.notes[i] ? LED_DIM :
                    LED_OFF);
            }
            Tlc.update();
        }
        delay(1);
    }
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
