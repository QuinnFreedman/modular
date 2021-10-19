#include <stdint.h>
#include <EEPROM.h>
#include <SPI.h>
#include <Tlc5940.h>
#include "lib/digitalWriteFast.h"
#include "src/bitVector12.hpp"

#include "config.h"

// TODO: Currently, I save the whole state to EEPROM every time it is updated. This is
//       very slow and uses up limited EEPROM writes. It would be pretty easy to only
//       update the part that is acutally changing each time.

#define DEBUG false
#define DEBUG2 false

uint16_t reverse12(uint16_t x) {
    uint16_t result = 0;
    for (uint8_t i = 0; i < 12; i++) {
        bitWrite(result, 11 - i, bitRead(x, i));
    }
    return result;
}

const uint8_t LED_INDEX[12] = {12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1};
const BitVector12 SCALES[4] = {
    BitVector12(reverse12(0b111111111111)), // chromatic
    BitVector12(reverse12(0b101011010101)), // major/minor
    BitVector12(reverse12(0b101010010100)), // pentatonic
    BitVector12(reverse12(0b100010010000)), // triad
};

enum TriggerMode {TRIGGER_OUTPUT = false, TRIGGER_INPUT = true };

float BUTTON_LADDER_CUTOFFS[12];

bool showMenu = false;

typedef struct {
    BitVector12 notes;
    TriggerMode triggers[2];
    BitVector12 userProfiles[4];
} State;

State state = {
    notes: BitVector12(0xFFF),
    triggers: {TRIGGER_OUTPUT, TRIGGER_OUTPUT},
    userProfiles: {
        BitVector12(0),
        BitVector12(0),
        BitVector12(0),
        BitVector12(0),
    },
};

bool SHOULD_UPDATE_UI = true;
bool INVALIDATE_HYSTERESIS_CACHE = false;
uint8_t activeNotes[] = {0, 0};

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
    pinMode(MENU_BUTTON_PIN, INPUT_PULLUP);
    pinMode(6, INPUT); // TODO bufix

    Tlc.init();
    Tlc.clear();
    delay(10);
    Tlc.update();
    
    SPI.begin();
    SPI.setBitOrder(MSBFIRST);
    SPI.setDataMode(SPI_MODE0);
    
    Serial.begin(9600);
    Serial.println("init");

    initButtonCutoffs();

    if (EEPROM.read(0) != 255) {
        #if DEBUG
        Serial.println("Reading state from EEPROM");
        #endif
        EEPROM.get(0, state);
    }

    delay(100);
    
    pinMode(TRIG_PIN_A, state.triggers[0] == TRIGGER_OUTPUT ? OUTPUT : INPUT);
    pinMode(TRIG_PIN_B, state.triggers[1] == TRIGGER_OUTPUT ? OUTPUT : INPUT);
    enableInterrupt(TRIG_PIN_A);
    enableInterrupt(TRIG_PIN_B);
}

inline bool pinIsRising(uint16_t pin, byte oldValues, byte newValues) {
    const uint8_t bitMask = digitalPinToBitMask(pin);
    const uint8_t wasHigh = oldValues & bitMask;
    const uint8_t isHigh  = newValues & bitMask;
    return isHigh && !wasHigh;
}

ISR(PCINT2_vect) {
    #ifndef __AVR__
    static_assert(false, "Interrupts are programmed assuming that all interrupt pins are in PORTD. This may not be true on non-AVR boards.");
    #endif
    volatile static byte oldValues = 0;
    const byte newValues = PIND;
    if (state.triggers[0] == TRIGGER_INPUT &&
            pinIsRising(TRIG_PIN_A, oldValues, newValues)) {
        doQuantizeChannel(0, false);
    }
    if (state.triggers[1] == TRIGGER_INPUT &&
            pinIsRising(TRIG_PIN_B, oldValues, newValues)) {
        doQuantizeChannel(1, false);
    }
        
    oldValues = newValues;
}

inline void enableInterrupt(byte pin) {
    *digitalPinToPCMSK(pin) |= bit (digitalPinToPCMSKbit(pin));  // enable pin
    PCIFR  |= bit (digitalPinToPCICRbit(pin)); // clear any outstanding interrupt
    PCICR  |= bit (digitalPinToPCICRbit(pin)); // enable interrupt for the group
}

inline int16_t modulo(const int16_t x, const int16_t m) {
    int16_t mod = x % m;
    if (mod < 0) {
        mod += m;
    }
    return mod;
}

int16_t mod12(const int16_t x) { return modulo(x, 12); }

float readInputVoltage(const uint8_t pin) {
    uint16_t rawValue = analogRead(pin);
    const float value = (rawValue / (float) ANALOG_READ_MAX_VALUE) * 20.0 - 10.0;
    return value;
}

void writeOutputVoltage(const uint8_t csPin, const int8_t dacChannel, const float voltage) {
    const float minVoltage = -10;
    const float maxVoltage = 10;
    float dacOutput = (voltage - minVoltage) / (maxVoltage - minVoltage);
    MCP4922_write(csPin, dacChannel, dacOutput);
}

bool allSame(const int8_t* buffer, const int8_t bufferSize, int8_t* value) {
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
    #if DEBUG
    Serial.print("buttonPressed normal ");
    Serial.println(button);
    #endif
    state.notes.toggle(button);
    if (state.notes == BitVector12(0)) {
        state.notes.toggle(button);
    }
    SHOULD_UPDATE_UI = true;
    INVALIDATE_HYSTERESIS_CACHE = true;
    for (uint8_t channel = 0; channel < 2; channel++) {
        if (state.triggers[channel] == TRIGGER_INPUT) {
            doQuantizeChannel(channel, false);
        }
    }
    EEPROM.put(0, state);
}

void onButtonReleasedNormal(int8_t button, uint32_t time) {
    #if DEBUG
    Serial.print("buttonReleased normal ");
    Serial.print(button);
    Serial.print(" ");
    Serial.println(time);
    #endif
}

void onButtonPressedMenu(int8_t button) {
    #if DEBUG
    Serial.print("buttonPressed menu ");
    Serial.println(button);
    #endif
    switch (button) {
    case 0:
    case 1: 
        state.triggers[button] = (TriggerMode) !state.triggers[button];
        pinMode(button ? TRIG_PIN_A : TRIG_PIN_B, state.triggers[button] == TRIGGER_OUTPUT ? OUTPUT : INPUT);
        EEPROM.put(0, state);
        break;
    case 2:
        state.notes.shiftUp();
        EEPROM.put(0, state);
        break;
    case 3:
        state.notes.shiftDown();
        EEPROM.put(0, state);
        break;
    case 4:
    case 5:
    case 6:
    case 7:
        state.notes = SCALES[button - 4];
        EEPROM.put(0, state);
        break;
    
    default:
        break;
    }
    SHOULD_UPDATE_UI = true;
}

void onButtonReleasedMenu(int8_t button, uint32_t time) {
    #if DEBUG
    Serial.print("buttonReleased menu ");
    Serial.print(button);
    Serial.print(" ");
    Serial.println(time);
    #endif
    button -= 8;
    if (button >= 0 && button < 4) {
        if (time < LONG_PRESS_TIME_MILLIS) {
            state.notes = state.userProfiles[button];
            EEPROM.put(0, state);
        } else {
            state.userProfiles[button] = state.notes;
            // const size_t offset = (size_t) &(state.userProfiles[button]) - (size_t) &state;
            EEPROM.put(0, state);
        }
    }
}

void transpose(bool * data, int8_t delta) {
    for (int8_t i = 0; i < 12; i++) {
        data[i] = data[mod12(i + delta)];
    }
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

bool scalesEqualWithTranspositions(const BitVector12 a, const BitVector12 b) {
    if (a == b) return true;
    BitVector12 transposed = a;
    for (uint8_t i = 1; i < 12; i++) {
        transposed.shiftUp();
        if (transposed == b) return true;
    }
    return false;
}

uint16_t generateMenuUI() {
    uint16_t result = 0;
    bitWrite(result, 0, state.triggers[0]);
    bitWrite(result, 1, state.triggers[1]);
    const BitVector12 scales[8] = {
        SCALES[0],
        SCALES[1],
        SCALES[2],
        SCALES[3],
        state.userProfiles[0],
        state.userProfiles[1],
        state.userProfiles[2],
        state.userProfiles[3],
    };
    for (uint8_t i = 0; i < 8; i++) {
        bool equal = scalesEqualWithTranspositions(state.notes, scales[i]);
        bitWrite(result, i + 4, equal);
    }
    return result;
}

void doQuantizeChannel(const uint8_t channel, const bool shouldUpdateTrigger) {
    static float inputVoltageBuffer[2] = {0, 0};
    static int8_t outputNotes[2] = {0, 0};
    static bool triggerOut[2] = {0, 0};
    static uint32_t lastTriggerTime[2] = {0, 0};

    if (INVALIDATE_HYSTERESIS_CACHE) {
        outputNotes[0] = -1;
        outputNotes[1] = -1;
    }
    
    const uint8_t INPUT_PINS[2] = {ANALOG_INPUT_PIN_A, ANALOG_INPUT_PIN_B};
    const uint8_t TRIGGER_PINS[2] = {TRIG_PIN_A, TRIG_PIN_B};
    const uint32_t now = millis();
    const uint8_t i = channel;
    
    float inputVoltage = readInputVoltage(INPUT_PINS[i]);
    inputVoltageBuffer[i] = (inputVoltageBuffer[i] + inputVoltage) / 2.0;
    inputVoltage = inputVoltageBuffer[i];
    const int8_t semitones = quantizeNote(inputVoltage, outputNotes[i]);
    const float outputVoltage = semitonesToVoltage(semitones);
    if (DEBUG2 && i == 0) {
        Serial.print(inputVoltage);
        Serial.print(",");
        Serial.print(outputVoltage);
        Serial.println();
    }
    if (semitones != outputNotes[i]) {
        outputNotes[i] = semitones;
        writeOutputVoltage(DAC_CS_PIN, i, outputVoltage);
        lastTriggerTime[i] = now;
        if (shouldUpdateTrigger) {
            triggerOut[i] = true;
            digitalWrite(TRIGGER_PINS[i], HIGH);
        }
        activeNotes[i] = mod12(semitones);
        
        SHOULD_UPDATE_UI = true;
    } else if (shouldUpdateTrigger && triggerOut[i] && now - lastTriggerTime[i] > TRIGGER_TIME_MS) {
        triggerOut[i] = false;
        digitalWrite(TRIGGER_PINS[i], LOW);
    }
}

void loop() {
    {
        const bool menuButton = !digitalReadFast(MENU_BUTTON_PIN);
        SHOULD_UPDATE_UI |= menuButton != showMenu;
        showMenu = menuButton;
    }
    handleButtons();

    for (uint8_t i = 0; i < 2; i++) {
        if (state.triggers[i] == TRIGGER_OUTPUT) {
            doQuantizeChannel(i, true);
        }
    }

    if (SHOULD_UPDATE_UI) {
        delay(1);
        SHOULD_UPDATE_UI = false;
        if (showMenu) {
            const uint16_t ui = generateMenuUI();
            for (int i = 0; i < 12; i++) {
                Tlc.set(LED_INDEX[i], bitRead(ui, i) ? LED_DIM : LED_OFF);
            }
            Tlc.update();
        } else {
            for (int i = 0; i <= 12; i++) {
                const bool isActive = (i == activeNotes[0]
                #if SHOW_BOTH_CHANNELS
                    || i == activeNotes[1]
                #endif
                );
                Tlc.set(LED_INDEX[i],
                    isActive ? LED_BRIGHT :
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
