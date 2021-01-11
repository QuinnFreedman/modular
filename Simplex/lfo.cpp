#include "lfo.hpp"

#include <stdint.h>

#define round(x) ((int32_t)(x + 0.5f))

LFO::LFO() {
    this->lastTime = 0;
    this->lastHertzUpdateTime = 0;
    this->lastHertzUpdatePeroidTime = 0;
    this->period = 0;
}

void LFO::setHertz(float hertz) {
    this->period = (uint32_t) round(((float) MICROSECONDS_IN_SECOND) / hertz);
    this->lastHertzUpdateTime = this->lastTime;
    this->lastHertzUpdatePeroidTime = this->period;
}

float LFO::update(uint32_t timeMicros) {
    uint32_t timeSinceLastHertzUpdate = timeMicros - this->lastHertzUpdateTime;
    uint32_t timeInCurrentPeriod = (timeSinceLastHertzUpdate + this->lastHertzUpdatePeroidTime) % this->period;

    float time = (float) timeInCurrentPeriod / (float) this->period;

    this->lastTime = timeMicros;
    return time;
}
