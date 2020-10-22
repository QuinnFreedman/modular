#include "envelope.h"
#include <Arduino.h>

/*
 * Config
 */

#define MILLION 1000000
const uint32_t ADSR_ATTACK_MAX_DURATION_MICROS = 5 * MILLION;
const uint32_t ADSR_DECAY_MAX_DURATION_MICROS = 5 * MILLION;
const uint32_t ADSR_RELEASE_MAX_DURATION_MICROS = 5 * MILLION;
 

typedef enum Phase {ATTACK, RELEASE, SUSTAIN, DECAY, OFF} Phase;
typedef enum Mode {ADSR, AARR} Mode;

void goToPhase(Phase _phase);
float calculateAmountIntoPhase(Phase phase, Mode mode);
float getPhaseDuration(Phase phase, Mode mode);
float getValue(Mode mode, Phase phase, float t, Phase* shouldChangePhaseTo);

float cvValues[4] = {0.2, 0.05, 0.3, 0.2};
#define CV_ATTACK  (cvValues[0])
#define CV_DECAY   (cvValues[1])
#define CV_SUSTAIN (cvValues[2])
#define CV_RELEASE (cvValues[3])

Mode currentMode = ADSR;
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
        goToPhase(newPhase);
    }
    return currentValue;
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
                    //x = 0;
                }
                return x;
            }
            case OFF: {
                return 0;
            }
        }
        break;
    }
}

void gate(bool on) {
    gateOpen = on;
    if (on) {
        goToPhase(ATTACK);
    } else {
        goToPhase(RELEASE);
    }
}

void ping() {
    goToPhase(ATTACK);
}


void goToPhase(Phase phase) {
    // read pots for next phase
    //float x = analogRead(A2);

    float amountIntoPhase = calculateAmountIntoPhase(phase, currentMode);
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
    }
}
