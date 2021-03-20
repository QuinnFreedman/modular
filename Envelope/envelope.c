#include "envelope.h"
#include "config.h"
#include <Arduino.h>

typedef enum Mode {ADSR, AARR, AARR_LOOP, TRAP_LOOP} Mode;

typedef enum Phase {
    ATTACK = 0,
    DECAY = 1,
    SUSTAIN = 2,
    RELEASE = 3,
    OFF = 4
} Phase;

typedef enum LEDMode {
    SHOW_MODE,
    SHOW_PHASE,
} LEDMode;

void goToPhase(Phase phase, bool hardReset);
float calculateAmountIntoPhase(Phase phase, Mode mode);
float getPhaseDuration(Phase phase, Mode mode);
float getValue(Mode mode, Phase phase, float t, Phase* shouldChangePhaseTo);
float expFunction(float t, float k);
float inverseExpFunction(float t, float k);
bool isLooping();
bool shouldLoop();
void sampleCV(Mode mode, Phase phase);
void setLed(uint8_t led);

volatile float cvValues[4] = {0.2, 0.2, 0.8, 0.2};
#define CV_ATTACK  (cvValues[0])
#define CV_DECAY   (cvValues[1])
#define CV_SUSTAIN (cvValues[2])
#define CV_RELEASE (cvValues[3])
#define CV_ATTACK_EXP  (cvValues[1] * 2 - 1)
#define CV_RELEASE_EXP (cvValues[3] * 2 - 1)
#define CV_TRAP_ATTACK  (cvValues[0])
#define CV_TRAP_SUSTAIN (cvValues[1])
#define CV_TRAP_RELEASE (cvValues[2])
#define CV_TRAP_DELAY   (cvValues[3])

volatile Mode currentMode = DEFAULT_MODE;
volatile Phase currentPhase = OFF;
volatile LEDMode ledMode = SHOW_MODE;
volatile float currentValue = 0;
volatile float phaseStartTime = 0;
volatile float currentPhaseDuration = INFINITY;
volatile uint32_t currentTime = 0;
volatile uint32_t lastButtonPressTime = 0;
volatile bool gateOpen = false;

#if EOR_TRIGGER_ENABLED
volatile uint32_t lastEORTriggerTime = 0;
volatile bool EORTriggerOn = false;
#endif
#if EOF_TRIGGER_ENABLED
volatile uint32_t lastEOFTriggerTime = 0;
volatile bool EOFTriggerOn = false;
#endif

float debug_data[4];

float update(uint32_t _currentTime) {
    currentTime = _currentTime;

    //turn off EOR and EOF triggers if necessary
    #if EOR_TRIGGER_ENABLED
    if (EORTriggerOn) {
        uint32_t timeTriggerHasBeenOn = currentTime - lastEORTriggerTime;
        if (timeTriggerHasBeenOn >= TRIGGER_TIME_MICROS) {
            EORTriggerOn = true;
            digitalWrite(EOR_TRIGGER_PIN, LOW);
        }
    }
    #endif
    #if EOF_TRIGGER_ENABLED
    if (EOFTriggerOn) {
        uint32_t timeTriggerHasBeenOn = currentTime - lastEOFTriggerTime;
        if (timeTriggerHasBeenOn >= TRIGGER_TIME_MICROS) {
            EOFTriggerOn = true;
            digitalWrite(EOF_TRIGGER_PIN, LOW);
        }
    }
    #endif

    //switch LED mode if needed
    if (currentTime - lastButtonPressTime >= LED_SHOW_MODE_TIME_MICROS) {
        ledMode = SHOW_PHASE;
    }

    //calculate envelope value
    float elapsedTimeInPhase = currentTime - phaseStartTime;
    float t = elapsedTimeInPhase / currentPhaseDuration;
    Phase newPhase = currentPhase;
    currentValue = getValue(currentMode, currentPhase, t, &newPhase);
    if (newPhase != currentPhase) {
        goToPhase(newPhase, false);
    }
    return currentValue;
}

/**
 * Call to notify that the gate has been turned on or off
 */
void gate(bool on) {
    gateOpen = on;
    if (isLooping() && LOOP_WHEN_GATE_OFF) {
        if (on) {
            goToPhase(RELEASE, false);
        } else {
            goToPhase(ATTACK, false);
        }
    } else {
        if (on) {
            goToPhase(ATTACK, false);
        } else {
            goToPhase(RELEASE, false);
        }
    }
 }

/**
 * Call to notify that a ping input has been recieved 
 */
void ping() {
    if (isLooping() && LOOP_HARD_SYNC_ON_PING) {
        goToPhase(ATTACK, true);
    } else {
        goToPhase(ATTACK, false);
    }
}

/**
 * Go to the next operating mode (i.e. the button has just been pressed)
 */
void cycleModes() {
    /* 
    //debounce
    if (currentTime - lastButtonPressTime <= MIN_TIME_BETWEEN_BUTTON_PRESSES_MICROS) {
        return;
    }
    */

    //update
    lastButtonPressTime = currentTime;
    currentMode = (currentMode + 1) % 4;

    //set leds
    ledMode = SHOW_MODE;

    /*
    //sample CV
    sampleCV(currentMode, currentPhase);
    */
    goToPhase(OFF, true);
}

/**
 * For a given mode and phase, get the value that the envelope should be [0-1] based on
 * how far into that phase we are (t in [0-1]).
 * 
 * If it's time to go to the next phase, `shouldChangePhaseTo` will be written to as an
 * OUT parameter.
 */
float getValue(Mode mode, Phase phase, float t, Phase* shouldChangePhaseTo) {
    switch(mode) {
        case ADSR:
        switch(phase) {
            case ATTACK: {
                float x = t;
                if (x >= 1) {
                    *shouldChangePhaseTo = gateOpen ? DECAY : RELEASE;
                    x = 1;
                }
                return x;
            }
            case DECAY: {
                float x = 1 - (1 - CV_SUSTAIN) * t;
                if (x <= CV_SUSTAIN) {
                    *shouldChangePhaseTo = SUSTAIN;
                    x = CV_SUSTAIN;
                }
                return x;
            }
            case SUSTAIN: {
                return CV_SUSTAIN;
            }
            case RELEASE: {
                float x = CV_SUSTAIN - CV_SUSTAIN * t;
                if (x <= 0) {
                    *shouldChangePhaseTo = OFF;
                    x = 0;
                }
                return x;
            }
            case OFF: {
                return 0;
            }
        }
        break;
        case AARR:
        switch(phase) {
            case ATTACK: {
                float x = expFunction(t, CV_ATTACK_EXP);
                if (t >= 1) {
                    *shouldChangePhaseTo = gateOpen ? SUSTAIN : RELEASE;
                    x = 1;
                }
                return x;
            }
            case DECAY: {
                *shouldChangePhaseTo = SUSTAIN;
                return 1;
            }
            case SUSTAIN: {
                return 1;
            }
            case RELEASE: {
                float x = 1 - expFunction(t, CV_RELEASE_EXP);
                if (t >= 1) {
                    *shouldChangePhaseTo = OFF;
                    x = 0;
                }
                return x;
            }
            case OFF: {
                return 0;
            }
        }
        break;
        case AARR_LOOP:
        switch(phase) {
            case ATTACK: {
                float x = expFunction(t, CV_ATTACK_EXP);
                if (t >= 1) {
                    *shouldChangePhaseTo = RELEASE;
                    x = 1;
                }
                return x;
            }
            case DECAY: {
                *shouldChangePhaseTo = RELEASE;
                return 1;
            }
            case SUSTAIN: {
                *shouldChangePhaseTo = RELEASE;
                return 1;
            }
            case RELEASE: {
                float x = 1 - expFunction(t, CV_RELEASE_EXP);
                if (t >= 1) {
                    *shouldChangePhaseTo = shouldLoop() ? ATTACK : OFF;
                    x = 0;
                }
                return x;
            }
            case OFF: {
                if (shouldLoop()) {
                    *shouldChangePhaseTo = ATTACK;
                }
                return 0;
            }
        }
        break;
        case TRAP_LOOP:
        switch(phase) {
            case ATTACK: {
                float x = t;
                if (t >= 1) {
                    *shouldChangePhaseTo = SUSTAIN;
                    x = 1;
                }
                return x;
            }
            case DECAY: {
                *shouldChangePhaseTo = SUSTAIN;
                return 1;
            }
            case SUSTAIN: {
                if (t >= 1) {
                    *shouldChangePhaseTo = RELEASE;
                }
                return 1;
            }
            case RELEASE: {
                float x = 1 - t;
                if (t >= 1) {
                    *shouldChangePhaseTo = OFF;
                    x = 0;
                }
                return x;
            }
            case OFF: {
                if (t >= 1 && shouldLoop()) {
                    *shouldChangePhaseTo = ATTACK;
                }
                return 0;
            }
        }
        break;
    }
}

/**
 * t = time in range [0, 1]
 * k = sharpness of curve, in range [-1, 1]
 * output in range [0, 1]
 */
float expFunction(float t, float k) {
    /*
    t = 2 * (t - 1);
    k = EXP_RATE_SCALE * k;
    if (k == 0) {
        return (t + 2) / 4;
    }

    return ( exp(k * t) - exp(-2 * k) ) / ( exp(2 * k) - exp(-2 * k) );
    */
    if (fabsf(k) < EXP_FUNCTION_ZERO_THRESH) {
        return t;
    }
    return ( exp(2 * EXP_RATE_SCALE * k * t * 2) - 1 ) / ( exp(4 * EXP_RATE_SCALE * k) - 1 );
}

float inverseExpFunction(float t, float k) {
    if (fabsf(k) < EXP_FUNCTION_ZERO_THRESH) {
        return t;
    }
    return log( (exp(4 * EXP_RATE_SCALE * k) - 1) * t + 1 ) / ( 4 * EXP_RATE_SCALE * k );
}

void goToPhase(Phase phase, bool hardReset) {
    #if EOR_TRIGGER_ENABLED
    bool shouldTriggerEOR = false;
    #endif
    #if EOF_TRIGGER_ENABLED
    bool shouldTriggerEOF = false;
    #endif
    Phase oldPhase = currentPhase;
    do {
        sampleCV(currentMode, phase);
        currentPhaseDuration = getPhaseDuration(phase, currentMode);
        currentPhase = phase;
        #if EOR_TRIGGER_ENABLED
        if (phase == ATTACK) shouldTriggerEOR = true;
        #endif
        #if EOF_TRIGGER_ENABLED
        if (phase == RELEASE) shouldTriggerEOF = true;
        #endif
        // try the next phase if this phase has 0 duration
        phase = (phase + 1) % 5;
    } while (currentPhaseDuration == 0);
    if (currentPhaseDuration == INFINITY) {
        phaseStartTime = currentTime;
    } else {
        float amountIntoPhase = hardReset ? 0 : calculateAmountIntoPhase(currentPhase, currentMode);
        phaseStartTime = currentTime - currentPhaseDuration * amountIntoPhase;
    }

    // Handle EOR/EOF triggers
    #if EOR_TRIGGER_ENABLED
    if (shouldTriggerEOR) {
        lastEORTriggerTime = currentTime;
        EORTriggerOn = true;
        digitalWrite(EOR_TRIGGER_PIN, HIGH);
    }
    #endif
    #if EOF_TRIGGER_ENABLED
    if (shouldTriggerEOF) {
        lastEOFTriggerTime = currentTime;
        EOFTriggerOn = true;
        digitalWrite(EOF_TRIGGER_PIN, HIGH);
    }
    #endif
}

#define clamp(x) (x < 0 ? 0 : x > 1 ? 1 : x)
#define readCV(i, pin) (cvValues[i] = clamp( \
            ((float) analogRead(pin) - (float) ANALOG_READ_ZERO_VALUE) \
            / (float) (ANALOG_READ_MAX_VALUE - ANALOG_READ_ZERO_VALUE) ))

void sampleCV(Mode mode, Phase phase) {
    switch(mode) {
        case ADSR:
        switch(phase) {
            case ATTACK:
                readCV(0, CV_PIN_A); 
            break;
            case DECAY:
                readCV(1, CV_PIN_D);
                readCV(2, CV_PIN_S);
            break;
            case SUSTAIN:
                readCV(2, CV_PIN_S);
            break;
            case RELEASE:
                readCV(2, CV_PIN_S);
                readCV(3, CV_PIN_R);
            break;
        } break;
        case AARR:
        case AARR_LOOP:
        switch(phase) {
            case ATTACK:
                readCV(0, CV_PIN_A);
                readCV(1, CV_PIN_D);
            break;
            case RELEASE:
                readCV(2, CV_PIN_S);
                readCV(3, CV_PIN_R);
            break;
        } break;
        case TRAP_LOOP:
        switch(phase) {
            case ATTACK:
                readCV(0, CV_PIN_A);
            break;
            case SUSTAIN:
                readCV(1, CV_PIN_D);
            break;
            case RELEASE:
                readCV(2, CV_PIN_S);
            break;
            case OFF:
                readCV(3, CV_PIN_R);
            break;
        } break;
    }
}

/**
 * If we were to go into phase `phase` right now, how far into it would we alredy
 * be (e.g. if we are in a SUSTAIN, we are already most of the way throuh an
 * ATTACK so if we want to go back to ATTACK we don't start from zero).
 * 
 * returns [0-1]
 */
float calculateAmountIntoPhase(Phase phase, Mode mode) {
    switch(mode) {
        case ADSR:
        switch(phase) {
            case ATTACK: return currentValue;
            case DECAY: return CV_SUSTAIN == 0 ? 1 - currentValue : 1 - (currentValue - CV_SUSTAIN) / (1 - CV_SUSTAIN);
            case SUSTAIN: return 0;
            case RELEASE: return CV_SUSTAIN == 0 ? 1 : 1 - currentValue / CV_SUSTAIN;
            case OFF: return 0;
        }
        break;
        case TRAP_LOOP:
        switch(phase) {
            case ATTACK: return currentValue;
            case DECAY: return 0;
            case SUSTAIN: return 0;
            case RELEASE: return 1 - currentValue;
            case OFF: return 0;
        }
        break;
        case AARR:
        case AARR_LOOP:
        switch(phase) {
            case ATTACK: return inverseExpFunction(currentValue, CV_ATTACK_EXP);
            case DECAY: return 0;
            case SUSTAIN: return 0;
            case RELEASE: return inverseExpFunction(1 - currentValue, CV_RELEASE_EXP);
            case OFF: return 0;
        }
        break;
    }
}


#define lerp(x, min, max) (min + x * (max - min))
/**
 * Get the total duration of a full iteration of the given phase in micros
 */
float getPhaseDuration(Phase phase, Mode mode) {
    switch(mode) {
        case ADSR:
        switch(phase) {
            case ATTACK:  return lerp(CV_ATTACK,  ATTACK_MIN_DURATION_MICROS,  ATTACK_MAX_DURATION_MICROS);
            case DECAY:   return lerp(CV_DECAY,   DECAY_MIN_DURATION_MICROS,   DECAY_MAX_DURATION_MICROS);
            case SUSTAIN: return INFINITY;
            case RELEASE: return lerp(CV_RELEASE, RELEASE_MIN_DURATION_MICROS, RELEASE_MAX_DURATION_MICROS * CV_SUSTAIN);
            case OFF:     return INFINITY;
        }
        break;
        case AARR:
        switch(phase) {
            case ATTACK:  return lerp(CV_ATTACK,  ATTACK_MIN_DURATION_MICROS,  ATTACK_MAX_DURATION_MICROS);
            case DECAY:   return 0;
            case SUSTAIN: return INFINITY;
            case RELEASE: return lerp(CV_SUSTAIN, RELEASE_MIN_DURATION_MICROS, RELEASE_MAX_DURATION_MICROS);
            case OFF:     return INFINITY;
        }
        break;
        case AARR_LOOP:
        switch(phase) {
            case ATTACK:  return lerp(CV_ATTACK,  ATTACK_MIN_DURATION_MICROS,  ATTACK_MAX_DURATION_MICROS);
            case DECAY:   return 0;
            case SUSTAIN: return 0;
            case RELEASE: return lerp(CV_SUSTAIN, RELEASE_MIN_DURATION_MICROS, RELEASE_MAX_DURATION_MICROS);
            case OFF:     return 0;
        }
        break;
        case TRAP_LOOP:
        switch(phase) {
            case ATTACK:  return lerp(CV_TRAP_ATTACK,  ATTACK_MIN_DURATION_MICROS,  ATTACK_MAX_DURATION_MICROS);
            case DECAY:   return 0;
            case SUSTAIN: return lerp(CV_TRAP_SUSTAIN, SUSTAIN_MIN_DURATION_MICROS, SUSTAIN_MAX_DURATION_MICROS);
            case RELEASE: return lerp(CV_TRAP_RELEASE, RELEASE_MIN_DURATION_MICROS, RELEASE_MAX_DURATION_MICROS);
            case OFF:     return lerp(CV_TRAP_DELAY,   DELAY_MIN_DURATION_MICROS,   DELAY_MAX_DURATION_MICROS);
        }
        break;
    }
}

/*
void setLed(uint8_t led) {
    static bool state[4] = {false, false, false, false};
    for (uint8_t i = 0; i < 4; i++) {
        const bool desired = i == led;
        const bool actual = state[i];
        if (desired != actual) {
            state[i] = desired;
            digitalWrite(LED_PINS[i], desired);
        }
    }
}
*/

void setLeds(bool leds[4]) {
    static bool cache[4] = {false, false, false, false};
    for (uint8_t i = 0; i < 4; i++) {
        if (leds[i] != cache[i]) {
            cache[i] = leds[i];
            digitalWrite(LED_PINS[i], leds[i]);
        }
    }
}

const bool LED_LOOKUP[4][5][4] = {
    {
        {1, 0, 0, 0},
        {0, 2, 0, 0},
        {0, 0, 3, 0},
        {0, 0, 0, 4},
        {0, 0, 0, 0},
    },
    {
        {1, 1, 0, 0},
        {0, 0, 0, 0},
        {1, 1, 1, 1},
        {0, 0, 1, 1},
        {0, 0, 0, 0},
    },
    {
        {1, 1, 0, 0},
        {0, 0, 0, 0},
        {0, 0, 0, 0},
        {0, 0, 1, 1},
        {0, 0, 0, 0},
    },
    {
        {1, 0, 0, 0},
        {0, 0, 0, 0},
        {0, 1, 0, 0},
        {0, 0, 1, 0},
        {0, 0, 0, 1},
    },
};

void updateLEDs() {
    if (ledMode == SHOW_MODE) {
        //setLed(currentMode);
        bool leds[] = {false, false, false, false};
        leds[currentMode] = true;
        setLeds(leds);
    } else {
        //setLed(currentPhase);
        for (uint8_t i = 0; i < 4; i++) {
            setLeds(LED_LOOKUP[currentMode][currentPhase]);
        }
    }
    #if LED_MODE_INDICATOR_ENABLED
    //TODO for performance this should be cached but I don't think this
    //is a feature anyone will really use
    digitalWrite(LED_MODE_INDICATOR_PIN, currentMode == SHOW_MODE);
    #endif
}

inline bool isLooping() {
    return currentMode == AARR_LOOP || currentMode == TRAP_LOOP;
}

inline bool shouldLoop() {
    return (!gateOpen &&  LOOP_WHEN_GATE_OFF) ||
           ( gateOpen && !LOOP_WHEN_GATE_OFF);
}
