#include "envelope.h"
#include <Arduino.h>

/*
 * Config
 */

#define MILLION 1000000
const uint32_t ADSR_ATTACK_MAX_DURATION_MICROS = 5 * MILLION;
const uint32_t ADSR_DECAY_MAX_DURATION_MICROS = 5 * MILLION;
//sustain time only used in TRAP mode
const uint32_t ADSR_SUSTAIN_MAX_DURATION_MICROS = 5 * MILLION;
const uint32_t ADSR_RELEASE_MAX_DURATION_MICROS = 5 * MILLION;

const float EXP_RATE_SCALE = 4;
const float EXP_FUNCTION_ZERO_THRESH = 0.0001;

const bool LOOP_HARD_SYNC_ON_PING = true;
const bool LOOP_WHEN_GATE_OFF = true;

/*
 * End config
 */

typedef enum Phase {ATTACK, RELEASE, SUSTAIN, DECAY, OFF} Phase;
typedef enum Mode {ADSR, AARR, AARR_LOOP} Mode;

void goToPhase(Phase phase, bool hardReset);
float calculateAmountIntoPhase(Phase phase, Mode mode);
float getPhaseDuration(Phase phase, Mode mode);
float getValue(Mode mode, Phase phase, float t, Phase* shouldChangePhaseTo);
float expFunction(float t, float k);
float inverseExpFunction(float t, float k);
bool isLooping();
bool shouldLoop();

float cvValues[4] = {0.2, 0.5, 0.2, 0.0};
#define CV_ATTACK  (cvValues[0])
#define CV_DECAY   (cvValues[1])
#define CV_SUSTAIN (cvValues[2])
#define CV_RELEASE (cvValues[3])
#define CV_ATTACK_EXP  (cvValues[1] * 2 - 1)
#define CV_RELEASE_EXP (cvValues[3] * 2 - 1)

Mode currentMode = AARR_LOOP;
Phase currentPhase = OFF;
float currentValue = 0;
float phaseStartTime = 0;
float currentPhaseDuration = 0;
uint32_t currentTime = 0;
bool gateOpen = false;

float update(uint32_t _currentTime) {
    currentTime = _currentTime;
    float elapsedTimeInPhase = currentTime - phaseStartTime;
    float t = elapsedTimeInPhase / (float) currentPhaseDuration;
    Phase newPhase = currentPhase;
    currentValue = getValue(currentMode, currentPhase, t, &newPhase);
    if (newPhase != currentPhase) {
        goToPhase(newPhase, false);
    }
    return currentValue;
}

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

void ping() {
    if (isLooping() && LOOP_HARD_SYNC_ON_PING) {
        goToPhase(ATTACK, true);
    } else {
        goToPhase(ATTACK, false);
    }
}

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
    if (abs(k) < EXP_FUNCTION_ZERO_THRESH) {
        return t;
    }
    return ( exp(2 * EXP_RATE_SCALE * k * t) - 1 ) / ( exp(4 * EXP_RATE_SCALE * k) - 1 );
}

float inverseExpFunction(float t, float k) {
    if (abs(k) < EXP_FUNCTION_ZERO_THRESH) {
        return t;
    }
    return log( (exp(4 * EXP_RATE_SCALE * k - 1) - 1) * t + 1 ) / ( 2 * EXP_RATE_SCALE * k );
}

void goToPhase(Phase phase, bool hardReset) {
    // read pots for next phase
    //float x = analogRead(A2);

    float amountIntoPhase = hardReset ? 0 : calculateAmountIntoPhase(phase, currentMode);
    currentPhaseDuration = getPhaseDuration(phase, currentMode);
    phaseStartTime = currentTime - currentPhaseDuration * amountIntoPhase;
    currentPhase = phase;
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
            case DECAY: return 1 - (currentValue - CV_SUSTAIN) / (1 - CV_SUSTAIN);
            case SUSTAIN: return 0;
            case RELEASE: return 1 - currentValue / CV_SUSTAIN;
            case OFF: return 0;
        }
        break;
        case AARR:
        case AARR_LOOP:
        switch(phase) {
            case ATTACK: return inverseExpFunction(currentValue, CV_ATTACK_EXP);
            case DECAY: 0;
            case SUSTAIN: return 0;
            case RELEASE: return inverseExpFunction(1 - currentValue, CV_RELEASE_EXP);
            case OFF: return 0;
        }
        break;
    }
}

/**
 * Get the total duration of a full iteration of the given phase in micros
 */
float getPhaseDuration(Phase phase, Mode mode) {
    switch(mode) {
        case ADSR:
        switch(phase) {
            case ATTACK: return CV_ATTACK * ADSR_ATTACK_MAX_DURATION_MICROS;
            case DECAY: return CV_DECAY * ADSR_DECAY_MAX_DURATION_MICROS;
            case SUSTAIN: return 1;
            case RELEASE: return CV_RELEASE * ADSR_RELEASE_MAX_DURATION_MICROS;
            case OFF: return 1;
        }
        break;
        case AARR:
        switch(phase) {
            case ATTACK: return CV_ATTACK * ADSR_ATTACK_MAX_DURATION_MICROS;
            case DECAY: return 0;
            case SUSTAIN: return 1;
            case RELEASE: return CV_SUSTAIN * ADSR_RELEASE_MAX_DURATION_MICROS;
            case OFF: return 1;
        }
        break;
        case AARR_LOOP:
        switch(phase) {
            case ATTACK: return CV_ATTACK * ADSR_ATTACK_MAX_DURATION_MICROS;
            case DECAY: return 0;
            case SUSTAIN: return 0;
            case RELEASE: return CV_SUSTAIN * ADSR_RELEASE_MAX_DURATION_MICROS;
            case OFF: return 0;
        }
        break;
    }
}

bool isLooping() {
    return currentMode == AARR_LOOP;
}

inline bool shouldLoop() {
    return (!gateOpen &&  LOOP_WHEN_GATE_OFF) ||
           ( gateOpen && !LOOP_WHEN_GATE_OFF);
}
