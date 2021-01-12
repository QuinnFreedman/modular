#ifndef config_h_INCLUDED
#define config_h_INCLUDED

#include <stdint.h>

// Number of microseconds in second
constexpr uint32_t SECOND = 1000000;

/*
 * Configuration
 * Change the following values to change how the module works.
 */
 
// Number of (micro)seconds before the screen goes to sleep
const uint32_t SLEEP_TIMEOUT_MICROS = 8 * SECOND;

// How long you have to hold down the rotary button to count as a long press
const uint32_t LONG_PRESS_TIME_MICROS = .5 * SECOND;

// Symbol used for clock speeds that are SLOWER than base
// Use 246 for the ASCII division symbol
const char CLOCK_DIVISION_SYMBOL = '/';
// Symbol used for clock speeds that are FASTER than base
const char CLOCK_MULTIPLE_SYMBOL = 'x';
// Character used for the menu cursor when edititng channel properties
const char SUBMENU_CURSOR_SYMBOL = '>';

// number of output channels
#define NUM_OUTPUTS 8
// The initial values of each clock channel. Negative = division
const PROGMEM int8_t DEFAULT_CLOCK_VALUES[NUM_OUTPUTS] = {1, -2, -4, -8, 2, 4, 8, 16};

// The initial BPM value
const uint8_t DEFAULT_BPM = 70;

#define SWING_ENABLED         true
#define PHASE_SHIFT_ENABLED   true
#define PAUSE_BUTTON_ENABLED  false
#define TAP_TEMPO_ENABLED     false
#define CACHE_OUTPUTS_ENABLED true
#define PORTS_ENABLED         true

#if TAP_TEMPO_ENABLED
// Min & max input clock deltas (micros) for running in follow mode.
const uint32_t CLOCK_INPUT_TIME_MAX = 60 * SECOND / 35;  // 35bpm
const uint32_t CLOCK_INPUT_TIME_MIN = 60 * SECOND / 255; // 255bpm
#endif

#if SWING_ENABLED 
// What is the maximum value that can be set for the swing of each clock
const int8_t MAX_SWING = 75;
// What is the maximum value that can be set for the swing of each clock
// (i.e. can swing be negative?)
const int8_t MIN_SWING = 0;
// Swing is divided by this number to get a fraction from -1 to 1.
const float SWING_SCALE = 100;
#endif

#if PAUSE_BUTTON_ENABLED
// If true, the LED pin will be set HIGH when paused.
// Otherwise, will be set HIGH when not paused
const bool GLOW_ON_PAUSED = true;
#endif

/*
 * Screen saver mode. After SLEEP_TIMEOUT_MICROS microseconds since the last
 * user input, the screen will go to one of these modes.
 *
 * Set SCREEN_SAVER to be equal to whatever mode you want to use.
 *
 * Note - drawing to the screen takes time (I2C is really slow)
 *        so setting a fast-moving screen saver may cause fast
 *        subdivisions to be a little out of sync. SS_NONE, SS_BLACK,
 *        and SS_BPM all don't require any redraws after the first one
 *        so they should be fine.
 */
#define SS_NONE  0 // No screensaver -- stay on the current menu
#define SS_BLACK 1 // Turn the screen off
#define SS_BPM   2 // Return to the BPM-select screen
#define SS_PULSE 3 // Show a pulsing 1-beat animation
#define SS_BARS  4 // Show a 4-beat scroll animation
#define SCREEN_SAVER SS_BARS 

/*
 * Pins (for Arduino Nano)
 * Change if you want a different layout or a different board.
 * Note: this code is expecting ENCODER_PIN_A to be an external interrupt
 *       pin, ENCODER_BUTTON_PIN and PAUSE_BUTTON_PIN to be in PCINT2_vect,
 *       and CLOCK_INPUT to be in PCINT0_vect. Search the Arduino
 *       documentation if you're not sure about your board.
 */
const uint8_t ENCODER_PIN_A = 3;
const uint8_t ENCODER_PIN_B = 4;
const uint8_t ENCODER_BUTTON_PIN = 5;
const uint8_t CLOCK_INPUT_PIN = 8;
const uint8_t PAUSED_LED_PIN = 7;
const uint8_t PAUSE_BUTTON_PIN = 6;

constexpr PROGMEM uint8_t OUTPUT_PINS[NUM_OUTPUTS] = {A3, 9, A2, 10, A1, 11,  A0, 12};


/*
 * OLED screen settings. See Adafruit_SSD1306 for more info.
 */
#define SCREEN_WIDTH 128 // OLED display width, in pixels
#define SCREEN_HEIGHT 64 // OLED display height, in pixels
#define OLED_RESET     4 // Reset pin # (or -1 if sharing Arduino reset pin)
#define SCREEN_I2C_ADDRESS 0x3C // Change to correct address for your screen

#endif // config_h_INCLUDED

