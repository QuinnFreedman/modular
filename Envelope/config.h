#ifndef __config_h__
#define __config_h__

#define MILLION 1000000
const uint32_t ATTACK_MAX_DURATION_MICROS = 5 * MILLION;
const uint32_t ATTACK_MIN_DURATION_MICROS = 0;
const uint32_t DECAY_MAX_DURATION_MICROS = 5 * MILLION;
const uint32_t DECAY_MIN_DURATION_MICROS = 0;
//sustain time only used in TRAP mode
const uint32_t SUSTAIN_MAX_DURATION_MICROS = 5 * MILLION;
const uint32_t SUSTAIN_MIN_DURATION_MICROS = 0;
const uint32_t RELEASE_MAX_DURATION_MICROS = 5 * MILLION;
const uint32_t RELEASE_MIN_DURATION_MICROS = 0;
const uint32_t DELAY_MAX_DURATION_MICROS = 5 * MILLION;
const uint32_t DELAY_MIN_DURATION_MICROS = 0;

const float EXP_RATE_SCALE = 4;
const float EXP_FUNCTION_ZERO_THRESH = 0.0001;

const bool LOOP_HARD_SYNC_ON_PING = true;
const bool LOOP_WHEN_GATE_OFF = true;

const uint32_t LED_SHOW_MODE_TIME_MICROS = 2 * MILLION;

const uint16_t ANALOG_READ_MAX_VALUE = 1024;
const uint16_t ANALOG_READ_ZERO_VALUE = 15;

#define DEFAULT_MODE 0

#define GATE_PASSTHROUGH_ENABLED false
#define LED_MODE_INDICATOR_ENABLED false
#define EOR_TRIGGER_ENABLED false
#define EOF_TRIGGER_ENABLED false

#if EOR_TRIGGER_ENABLED || EOF_TRIGGER_ENABLED
const uint16_t TRIGGER_TIME_MICROS = 50000; // 5ms
#endif

//PINS
const uint16_t GATE_IN_PIN = 3;
const uint16_t RETRIG_IN_PIN = 2;
const uint16_t LED_PINS[4] = {5, 6, 7, 8};
const uint16_t DAC_CS_PIN = 9;
#define CV_PIN_A A2
#define CV_PIN_D A0
#define CV_PIN_S A1
#define CV_PIN_R A3
#define AUX_PIN_1 A4 
#define AUX_PIN_2 A5 

#define BUTTON_PIN 4

#if GATE_PASSTHROUGH_ENABLED
#define GATE_OUT_PIN AUX_PIN_1
#endif

#if LED_MODE_INDICATOR_ENABLED
#define LED_MODE_INDICATOR_PIN AUX_PIN_2
#endif

#if EOR_TRIGGER_ENABLED
#define EOR_TRIGGER_PIN AUX_PIN_1
#endif

#if EOF_TRIGGER_ENABLED
#define EOF_TRIGGER_PIN AUX_PIN_2
#endif

#endif // __config_h__

