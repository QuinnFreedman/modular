#ifndef lfo_hpp_INCLUDED
#define lfo_hpp_INCLUDED

#include <stdint.h>

const uint32_t MICROSECONDS_IN_SECOND = 1000000;

template <typename f>
using waveform_t = f(*)(f);

template <typename float_t>
class LFO {
    public:
        LFO(waveform_t<float_t> waveform);
        float_t update(uint32_t timeMicros);
        void setHertz(float_t hertz);
        void setWaveform(waveform_t<float_t> waveform);
    private:
        uint32_t lastTime;
        uint32_t lastHertzUpdateTime;
        float_t lastHertzUpdatePeroidFraction;
        uint32_t period;
        waveform_t<float_t> waveform;
        inline float_t getFractionOfCurrentPeriod();
};

#define round(x) ((int32_t)(x + 0.5f))

template <typename float_t>
LFO<float_t>::LFO(waveform_t<float_t> waveform) {
    this->lastTime = 0;
    this->lastHertzUpdateTime = 0;
    this->lastHertzUpdatePeroidFraction = 0;
    this->period = 0;
    this->waveform = waveform;
}

template <typename float_t>
void LFO<float_t>::setWaveform(waveform_t<float_t> waveform) {
    this->waveform = waveform;
}

template <typename float_t>
void LFO<float_t>::setHertz(float_t hertz) {
    this->lastHertzUpdatePeroidFraction = getFractionOfCurrentPeriod();
    this->period = MICROSECONDS_IN_SECOND / hertz;
    this->lastHertzUpdateTime = this->lastTime;
}

template <typename float_t>
inline float_t LFO<float_t>::getFractionOfCurrentPeriod() {
    if (this->period == 0) {
        return 0;
    }
    const uint32_t timeMicros = this->lastTime;
    const uint32_t timeSinceLastHertzUpdate = timeMicros - this->lastHertzUpdateTime;
    float_t periodTime = ((float_t) (timeSinceLastHertzUpdate % this->period) / (float_t) this->period) + this->lastHertzUpdatePeroidFraction;
    if (periodTime > 1) {
        periodTime -= 1; 
    }
    return periodTime;
}

template <typename float_t>
float_t LFO<float_t>::update(uint32_t timeMicros) {
    this->lastTime = timeMicros;
    float_t timeInPeriod = getFractionOfCurrentPeriod();

    return this->waveform(timeInPeriod);
}

namespace Waveforms {
    
template <typename float_t>
float_t saw(float_t t) {
    return t;
}

template <typename float_t>
float_t inverseSaw(float_t t) {
    return 1-t;
}

template <typename float_t>
float_t tri(float_t t) {
    return t < 0.5
        ? 2 * t
        : 1 - 2 * (t - 0.5);
}

#ifndef PI
#define PI 3.14159265359
#endif

template <typename float_t>
float_t sin(float_t x) {
    return (1 - cos(x * 2 * PI)) / 2;
}

template <typename float_t>
float_t square(float_t t) {
    return (t >  0.25) && (t < 0.75);
}

template <typename float_t>
float_t bounce(float_t x) {
    const float_t k = 1;
    const float_t decayInverseZero = exp(k) - 1;
    const float_t scaledX = x * decayInverseZero;
    const float_t decay = 1 - log(scaledX + 1) / k;
    return abs(cos(2 * PI * exp(scaledX))) * decay;
}

}

#endif // lfo_h_INCLUDED

