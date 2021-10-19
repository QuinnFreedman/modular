#ifndef __config_h__
#define __config_h__

// whether to illuminate the currently played notes for both channels or only the top one.
// note that a channel will still be displayed even when it has no input plugged in
#define SHOW_BOTH_CHANNELS false

// How bright the LEDs should be when the note is currently being played (from 0 to 4095)
const uint16_t LED_BRIGHT = (pow(2, 12) - 1) / 10;
// How bright the LEDs should be when the note is selected but not active (from 0 to 4095)
const uint16_t LED_DIM = LED_BRIGHT / 10;
// How bright the LEDs should be when the note is not selected (from 0 to 4095)
const uint16_t LED_OFF = 0;

// How long to keep the trigger output HIGH when the note changes (in milliseconds)
const uint32_t TRIGGER_TIME_MS = 100;

// How long you have to hold a button down for it to count as a "long" press
const uint32_t LONG_PRESS_TIME_MILLIS = 500;
// The hysteresis threshold for changing notes, as a fraction of the distance between two notes
const float HYSTERESIS_THRESHOLD = 0.7;

// PINS

const uint8_t BUTTON_LADDER_PIN = A0;
const uint8_t ANALOG_INPUT_PIN_A = A7;
const uint8_t ANALOG_INPUT_PIN_B = A6;
const uint8_t MENU_BUTTON_PIN = 2;
const uint8_t DAC_CS_PIN = 8;

const uint8_t TRIG_PIN_A = 4;
const uint8_t TRIG_PIN_B = 5;

// The maximum value of the analogRead() function for this Arduino
const uint16_t ANALOG_READ_MAX_VALUE = 1023;

#endif
