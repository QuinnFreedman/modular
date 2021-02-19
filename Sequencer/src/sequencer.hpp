#ifndef sequencer_hpp_INCLUDED
#define sequencer_hpp_INCLUDED

#include <stdint.h>

enum Mode {
    MODE_4x4,
    MODE_2x8,
    MODE_1x16,
};

enum OutputMode {
    OUTPUT_PROBABILITY,
    OUTPUT_ANALOG,
    OUTPUT_QUANTIZED,
};

class Sequencer {
    public:
        Sequencer();
        void advance(bool channels[4]);
        void jump(uint8_t column);
        const float * getOutputState() { return this->outputValues; }
        const bool * getLedState() { return this->leds; }
        void setValue(uint8_t i, float value) { this->values[i] = value; }
        void setMode(Mode mode);
        void setOutputMode(OutputMode outputMode) { this->outputMode = outputMode; }
    private:
        float outputValues[4];
        bool leds[16];
        float values[16];
        Mode mode;
        OutputMode outputMode;
        uint8_t cursors[4];
        void update(bool[4]);
        void setOutput(uint8_t channels, float value);
        void updateLEDs();
};

uint8_t getNumChannels(Mode mode) {
    switch(mode) {
        case MODE_4x4: {
            return 4;
        } break;
    }
}

uint8_t getChannelLength(Mode mode) {
    return 16 / getNumChannels(mode); //TODO temp hack
}

Sequencer::Sequencer() {
    for (uint8_t i = 0; i < 4; i++) {
        outputValues[i] = 0;
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
    for (uint8_t i = 0; i < 4; i++) {
        cursors[i] = 0;
    }
    bool allChannels[4] = {true, true, true, true};
    this->update(allChannels);
}

void Sequencer::updateLEDs() {
    for (uint8_t i = 0; i < 16; i++) {
        leds[i] = 0;
    }
    uint8_t numChannels = getNumChannels(this->mode);
    uint8_t channelSteps = getChannelLength(this->mode);
    for (uint8_t i = 0; i < numChannels; i++) {
        uint8_t cursor = this->cursors[i];
        uint8_t ledIndex = cursor + (channelSteps * i);
        this->leds[ledIndex] = true;
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
                this->cursors[i] = step * (channelSteps / 4);
            }
        } break;
    }
    bool allChannels[4] = {true, true, true, true};
    this->update(allChannels);
}

void Sequencer::advance(bool channels[4]) {
    switch(this->mode) {
        case MODE_4x4: 
        case MODE_2x8: 
        case MODE_1x16: {
            uint8_t numChannels = getNumChannels(this->mode);
            uint8_t channelSteps = getChannelLength(this->mode);
            for (uint8_t i = 0; i < numChannels; i++) {
                if (channels[i]) {
                    cursors[i] = (cursors[i] + 1) % channelSteps;
                }
            }
        } break;
    }
    this->update(channels);
}

void Sequencer::update(bool channels[4]) {
    switch(this->mode) {
        case MODE_4x4: {
            for (uint8_t i = 0; i < 4; i++) {
                if (channels[i]) {
                    uint8_t channelMask = 1 << i;
                    uint8_t index = (i * 4) + channels[i];
                    setOutput(channelMask, values[index]);
                }
            }
        } break;
    }
    this->updateLEDs();
}

void Sequencer::setOutput(uint8_t channels, float value) {
    switch(this->outputMode) {
        case OUTPUT_ANALOG: {
            for (uint8_t i = 0; i < 4; i++) {
                if (bitRead(channels, i)) {
                    outputValues[i] = value;
                }
            }
        } break;
        case OUTPUT_QUANTIZED: {
            //TODO
        } break;
        case OUTPUT_PROBABILITY: {
            //TODO
        } break;
    }
}

#endif // sequencer_hpp_INCLUDED

