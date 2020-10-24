#ifndef __envelope_h__
#define __envelope_h__

#include <stdint.h>
#include <stdbool.h>

float update(uint32_t currentTime);
void gate(bool on);
void ping();
void cycleModes();

#endif

