#ifndef sequencer_hpp_INCLUDED
#define sequencer_hpp_INCLUDED

#include <stdint.h>

enum Mode {
    MODE_4x4,
    MODE_2x8,
    MODE_1x16,
    MODE_OFF,
};

enum OutputMode {
    OUTPUT_PROBABILITY,
    OUTPUT_ANALOG,
    OUTPUT_QUANTIZED,
};

class Sequencer {
    public:
        Sequencer(uint32_t seed);
        void advance(bool channels[4]);
        void jump(uint8_t column);
        const bool * getLedState() { return this->leds; }
        void setValue(uint8_t i, float value) { this->values[i] = value; }
        void setMode(Mode mode);
        void setOutputMode(OutputMode outputMode) { this->outputMode = outputMode; };
        void getOutput(float*);
    private:
        Mode mode;
        OutputMode outputMode;
        bool leds[16];
        float values[16];
        uint8_t cursors[4];
        void updateLEDs();
        uint32_t randomSeed;
        uint32_t getNextRandom();
        updateRandom();
};

void Sequencer::updateRandom() {
    for (uint8_t i = 0; i < 4; i++) {
        this->randomSeed = this->getNextRandom();
    }
}

uint32_t Sequencer::getNextRandom() {
    constexpr uint32_t m = 1U << 24;
    constexpr uint32_t a = 0x43FD43FD;
    constexpr uint32_t c = 0xC39EC3;
    return (a * this->randomSeed + c) % m;
}

uint8_t getNumChannels(Mode mode) {
    switch(mode) {
        case MODE_4x4: {
            return 4;
        } break;
        case MODE_2x8: {
            return 2;
        } break;
        case MODE_1x16: {
            return 1;
        } break;
    }
}

uint8_t getChannelLength(Mode mode) {
    return 16 / getNumChannels(mode); //TODO temp hack
}

Sequencer::Sequencer(uint32_t seed) {
    randomSeed = seed;
    for (uint8_t i = 0; i < 4; i++) {
        cursors[i] = 0;
    }
    for (uint8_t i = 0; i < 16; i++) {
        leds[i] = 0;
        values[i] = 0;
    }

    this->setMode(MODE_4x4);
    outputMode = OUTPUT_ANALOG;
}

void Sequencer::setMode(Mode mode) {
    this->mode = mode;
    switch(this->mode) {
        case MODE_4x4: {
            cursors[0] = 0; 
            cursors[1] = 4; 
            cursors[2] = 8; 
            cursors[3] = 12; 
        } break;
        case MODE_2x8: {
            cursors[0] = 0; 
            cursors[1] = 0; 
            cursors[2] = 8; 
            cursors[3] = 8; 
        } break;
        case MODE_1x16: {
            cursors[0] = 0; 
            cursors[1] = 0; 
            cursors[2] = 0; 
            cursors[3] = 0;
        } break;
    }
    this->updateLEDs();
}

void Sequencer::updateLEDs() {
    for (uint8_t i = 0; i < 16; i++) {
        leds[i] = 0;
    }
    
    if (this->mode == MODE_OFF) return;
    
    for (uint8_t i = 0; i < 4; i++) {
        uint8_t cursor = this->cursors[i];
        this->leds[cursor] = 1;
    }
}

void Sequencer::jump(uint8_t step) {
    switch(this->mode) {
        case MODE_4x4: 
        case MODE_2x8: 
        case MODE_1x16: {
            uint8_t numChannels = getNumChannels(this->mode);
            uint8_t channelSteps = getChannelLength(this->mode);
            for (uint8_t i = 0; i < numChannels; i++) {
                this->cursors[i] = step * (channelSteps / 4) + i * channelSteps;
            }
        } break;
    }
    this->updateLEDs();
    this->updateRandom();
}

#define offsetMod(x, modulus, offset) (((x - offset) % modulus) + offset)

void Sequencer::advance(bool channelsToAdvance[4]) {
    switch(this->mode) {
        case MODE_4x4: {
            if (channelsToAdvance[0]) cursors[0] = offsetMod(cursors[0] + 1, 4, 0);
            else if (channelsToAdvance[1]) cursors[1] = offsetMod(cursors[1] + 1, 4, 4);
            else if (channelsToAdvance[2]) cursors[2] = offsetMod(cursors[2] + 1, 4, 8);
            else if (channelsToAdvance[3]) cursors[3] = offsetMod(cursors[3] + 1, 4, 12);
        } break;
        case MODE_2x8: {
            if (channelsToAdvance[0] || channelsToAdvance[1]) {
                cursors[0] = offsetMod(cursors[0] + 1, 8, 0);
                cursors[1] = cursors[0];
            }
            else if (channelsToAdvance[2] || channelsToAdvance[3]) {
                cursors[2] = offsetMod(cursors[2] + 1, 8, 8);
                cursors[3] = cursors[2];
            }
        } break;
        case MODE_1x16: {
            cursors[0] = offsetMod(cursors[0] + 1, 16, 0);
            cursors[1] = cursors[0];
            cursors[2] = cursors[0];
            cursors[3] = cursors[0];
        } break;
    }
    this->updateLEDs();
    this->updateRandom();
}

void Sequencer::getOutput(float* output) {
    if (this->outputMode == OUTPUT_PROBABILITY) {
        for (uint8_t i = 0; i < 4; i++) {
            float value = this->values[this->cursors[i]];
            float random = getNextRandom() / (1 << 24);
            output[i] = value > random;
        }
    } else {
        for (uint8_t i = 0; i < 4; i++) {
            output[i] = this->values[this->cursors[i]];
        }
    }
}

#endif // sequencer_hpp_INCLUDED

