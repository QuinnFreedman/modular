#ifndef lfo_hpp_INCLUDED
#define lfo_hpp_INCLUDED

#include <stdint.h>

const uint32_t MICROSECONDS_IN_SECOND = 1000000;

class LFO {
    public:
        LFO();
        float update(uint32_t timeMicros);
        void setHertz(float hertz);
    private:
        uint32_t lastTime;
        uint32_t lastHertzUpdateTime;
        uint32_t lastHertzUpdatePeroidTime;
        uint32_t period;
};

#endif // lfo_h_INCLUDED

