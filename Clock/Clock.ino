#include <stdint.h>
#include <Adafruit_GFX.h>
#include <Adafruit_SSD1306.h>

/*
 * Pins (for Arduino Nano)
 * Change if you want a different layout or a different board.
 * Note: this code is expecting ENCODER_BUTTON_PIN and ENCODER_PIN_A
 *       to be external interrupt pins and CLOCK_INPUT and 
 *       PAUSE_BUTTON to be in PCINT1_vect for ISR. Search the Arduino
 *       documentation if you're not sure about your board.
 */
const uint8_t LED_PIN_1 = 12;
const uint8_t LED_PIN_2 = 11;
const uint8_t ENCODER_BUTTON_PIN = 2;
const uint8_t ENCODER_PIN_A = 3;
const uint8_t ENCODER_PIN_B = 4;
const uint8_t CLOCK_INPUT_PIN = A0;
const uint8_t PAUSE_BUTTON_PIN = A1;

const uint8_t NUM_OUTPUTS = 8;
const PROGMEM uint8_t OUTPUT_PINS[NUM_OUTPUTS] = {5, 6, 7, 8, 9, 10, 11, 12};

// Number of seconds before the screen goes to sleep
const uint32_t SLEEP_TIMEOUT_MICROS = 8 * 1000000;

// How mong you have to hold down the rotary button to count as a long press
const uint32_t LONG_PRESS_TIME_MICROS = 500000;

// Min & max input clock deltas (micros) for running in slave mode.
const uint32_t CLOCK_INPUT_TIME_MAX = 60000000 / 35;
const uint32_t CLOCK_INPUT_TIME_MIN = 200000;

/*
 * OLED screen settings. See Adafruit_SSD1306 for more info.
 */
#define SCREEN_WIDTH 128 // OLED display width, in pixels
#define SCREEN_HEIGHT 64 // OLED display height, in pixels
#define OLED_RESET     4 // Reset pin # (or -1 if sharing Arduino reset pin)
#define SCREEN_I2C_ADDRESS 0x3C // Change to correct address for your screen

/*
 * Screen saver mode. After SLEEP_TIMEOUT_MICROS microseconds since th last
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

typedef enum : uint8_t {NAVIGATE, EDIT_FAST, SUBMENU, SUBMENU_EDIT, SLEEP} MenuMode;
typedef enum : uint8_t {MULTIPLY, DIVIDE} ClockMode;

typedef struct {
    uint8_t multiplier;
    ClockMode mode;
    int8_t offset;
    int8_t pulseWidth;
} Clock;

typedef struct {
    uint8_t bpm;
    Clock clocks[NUM_OUTPUTS];
    int8_t cursor;
    int8_t submenuCursor;
    MenuMode mode;
    bool newInput;
} State;

State state;

Adafruit_SSD1306 display(SCREEN_WIDTH, SCREEN_HEIGHT, &Wire, OLED_RESET);

uint32_t lastRecordedTime;

uint8_t outputCache[NUM_OUTPUTS] = {LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW};

bool paused = false;

const PROGMEM int8_t initClockValues[NUM_OUTPUTS] = {1, -2, -4, -8, 2, 4, 8, 16};

/*
 * Initialize variables, setup pins
 */
void setup() {
    pinMode(LED_PIN_1, OUTPUT);
    pinMode(LED_PIN_2, OUTPUT);
    pinMode(ENCODER_BUTTON_PIN, INPUT_PULLUP);
	pinMode(ENCODER_PIN_A, INPUT_PULLUP);
	pinMode(ENCODER_PIN_B, INPUT_PULLUP);
    pinMode(CLOCK_INPUT_PIN, INPUT);
    pinMode(PAUSE_BUTTON_PIN, INPUT_PULLUP);
    attachInterrupt(digitalPinToInterrupt(ENCODER_BUTTON_PIN), rotaryButtonChangeHandler, CHANGE);
    attachInterrupt(digitalPinToInterrupt(ENCODER_PIN_A), rotaryEncoderHandler, CHANGE);
    enableInterrupt(CLOCK_INPUT_PIN);
    enableInterrupt(PAUSE_BUTTON_PIN);

    Serial.begin(9600);
 
    if(!display.begin(SSD1306_SWITCHCAPVCC, SCREEN_I2C_ADDRESS)) {
        Serial.println(F("SSD1306 allocation failed"));
        // Loop forever
        while (true) {}
    }

    state.bpm = 70;
    state.cursor = -1;
    state.submenuCursor = 0;
    state.mode = NAVIGATE;
    state.newInput = true;

    for (uint8_t i = 0; i < NUM_OUTPUTS; i++) {
        pinMode(pgm_read_byte(&OUTPUT_PINS[i]), OUTPUT);
        const int8_t val = pgm_read_byte(&(initClockValues[i]));
        Clock* clock = &state.clocks[i];

        clock->mode = val < 0 ? DIVIDE : MULTIPLY;
        clock->multiplier = abs(val);
        clock->offset = 0;
        clock->pulseWidth = 50;
    }
    
    lastRecordedTime = micros();
}


void loop() {
    static uint32_t beatStart = micros();
    static uint32_t lastInputTime = micros();

    /*  
     * Keep track of time
     */
    static uint32_t numBeats = 0;
    const uint32_t beatMicros = 60000000 / state.bpm;
    const uint32_t currentTime = micros();
    lastRecordedTime = currentTime;
    if (paused) return;
    uint32_t elapsed = currentTime - beatStart;
    while (elapsed >= beatMicros) {
        elapsed -= beatMicros;
        beatStart = currentTime - elapsed;
        numBeats++;
    }
    float elapsedFraction = ((float) elapsed) / ((float) beatMicros);
    
    /*
     * Check if there has been new knob/button input
     */
    const bool newInput = state.newInput;
    if (newInput) {
        lastInputTime = currentTime;
        state.newInput = false;
    }

    /*
     * Send signals to output pins
     */
    for (int i = 0; i < NUM_OUTPUTS; i++) {
        float elapsed;
        const Clock clock = state.clocks[i];
        if (clock.mode == MULTIPLY) {
            float _elapsedFraction = elapsedFraction - (float) clock.offset / (float) 200 * clock.multiplier;
            float _numBeats = numBeats;
            if (_elapsedFraction > 1) {
                _numBeats++;
                _elapsedFraction -= 1;
            } else if (_elapsedFraction < 0) {
                _numBeats--;
                _elapsedFraction += 1;
            } 
            elapsed = getElapsed(clock.multiplier, _numBeats, _elapsedFraction);
        } else {
            const float fraction = 1.0 / (float) clock.multiplier;
            float _elapsedFraction = 1 + elapsedFraction - (float) clock.offset / (float) 200 * fraction;
            float remainder = _elapsedFraction;
            while (remainder >= fraction) {
                remainder -= fraction;
            }
            elapsed = remainder / fraction;
        }
        const uint8_t output = elapsed < (float) clock.pulseWidth / (float) 100 ? HIGH : LOW;
        if (output != outputCache[i]) {
            outputCache[i] = output;
            digitalWrite(pgm_read_byte(&OUTPUT_PINS[i]), output);
        }
    }

    /*
     * draw screen
     */
    #if SCREEN_SAVER != SS_NONE
    bool forceRedrawScreen = false;
    if (state.mode != SLEEP && currentTime - lastInputTime > SLEEP_TIMEOUT_MICROS) {
        #if SCREEN_SAVER == SS_BPM
        state.mode = NAVIGATE;
        state.newInput = true;
        #else
        state.mode = SLEEP;
        forceRedrawScreen = true;
        #endif
        state.cursor = -1;
    }
    #endif

    if (state.mode == SLEEP) {
        #if SCREEN_SAVER == SS_BARS
        float elapsed = getElapsed(4, numBeats, elapsedFraction);
        #elif SCREEN_SAVER == SS_PULSE
        float elapsed = getElapsed(1, numBeats, elapsedFraction);
        #endif
        drawScreenSaver(display, elapsed, forceRedrawScreen);
    } else if (newInput) {
        if (state.mode == NAVIGATE || state.mode == EDIT_FAST) {
            drawMainMenu(display, state);
        } else if (state.mode == SUBMENU || state.mode == SUBMENU_EDIT) {
            drawSubMenu(display, state);
        }
    }
    delay(1);
}

/*
 * given elapsedFraction -- the amount of the the current beat that has passed --
 * gives the amount of a larger multi-beat section that has passed.
 */
inline float getElapsed(uint8_t periodLenBeats, uint32_t numBeatsElapsed,
                       float elapsedInCurrentBeat) {
    const uint32_t beatsInCurrentPeriod = numBeatsElapsed % periodLenBeats;
    const float fractionElapsedPrevious = (float) beatsInCurrentPeriod / (float) periodLenBeats;
    const float fractionElapsed = fractionElapsedPrevious + 
            (elapsedInCurrentBeat / (float) periodLenBeats);
    return fractionElapsed;
}

/*
 * BEGIN HANDLERS --
 * simple functions attached to interrupts to handle button input.
 * Most of them call anoither function to handle actual application logic.
 */

void rotaryButtonChangeHandler() {
    static uint32_t pressTime = 0;
    bool buttonPressed = digitalRead(ENCODER_BUTTON_PIN) == LOW;
    uint32_t now = lastRecordedTime;
    if (buttonPressed) {
        pressTime = now;
    } else {
        uint32_t holdTime = now - pressTime;
        if (holdTime >= LONG_PRESS_TIME_MICROS) {
            onLongPress();
        } else {
            onShortPress();
        }
    }
}

void rotaryEncoderHandler() {
    static int lastCLK = digitalRead(ENCODER_PIN_A);
    static int counter = 0;
    
    int a = digitalRead(ENCODER_PIN_A);
    int b = digitalRead(ENCODER_PIN_B);

    int currentStateCLK = a;
    if (currentStateCLK != lastCLK && currentStateCLK == 1) {
        int direction;
        if (b != currentStateCLK) {
            direction = -1;
        } else {
            direction = 1;
        }
        counter += direction;

        onKnobTurned(direction, counter);
    }

    lastCLK = currentStateCLK;
}

// builtin Arduino handler for pin change interrupt for pins A0 to A5
ISR(PCINT1_vect) {
    clockInputChangeHandler();
    pauseButtonChagneHandler();
}

inline void clockInputChangeHandler() {
    static int lastValue = digitalRead(CLOCK_INPUT_PIN);
    const int currentValue = digitalRead(CLOCK_INPUT_PIN);
    if (lastValue == LOW && currentValue == HIGH) {
        onClockInput();
    }
    lastValue = currentValue;
}

inline void pauseButtonChagneHandler() {
    static int lastValue = digitalRead(PAUSE_BUTTON_PIN);
    const int currentValue = digitalRead(PAUSE_BUTTON_PIN);
    if (lastValue == LOW && currentValue == HIGH) {
        onPuasePressed();
    }
    lastValue = currentValue;
}

/*
 * END HANDLERS
 */

inline void onLongPress() {
    state.newInput = true;
    switch (state.mode) {
    case NAVIGATE: 
        if (state.cursor == -1) {
            state.mode = EDIT_FAST;
        } else {
            state.mode = SUBMENU;
            state.submenuCursor = 0;
        }
        break;
    default:
        state.mode = NAVIGATE;
        break;
    }
}

inline void onShortPress() {
    state.newInput = true;
    switch (state.mode) {
    case NAVIGATE: 
        state.mode = EDIT_FAST;
        break;
    case SUBMENU: 
        state.mode = SUBMENU_EDIT;
        break;
    case SUBMENU_EDIT: 
        state.mode = SUBMENU;
        break;
    default:
        state.mode = NAVIGATE;
        break;
    }
}

inline void onKnobTurned(int direction, int counter) {
    state.newInput = true;
    switch (state.mode) {
    case NAVIGATE: {
        state.cursor += direction;
        if (state.cursor < -1) {
            state.cursor = -1;
        } else if (state.cursor >= NUM_OUTPUTS) {
            state.cursor = NUM_OUTPUTS - 1;
        }
    } break;
    case SUBMENU: {
        state.submenuCursor += direction;
        if (state.submenuCursor < 0) {
            state.submenuCursor = 0;
        } else if (state.submenuCursor > 2) {
            state.submenuCursor = 2;
        }
    } break;
    case EDIT_FAST: {
        if (state.cursor == -1) {
            state.bpm += direction;
        } else {
            Clock* clock = &state.clocks[state.cursor];
            if (clock->mode == MULTIPLY && clock->multiplier == 1 && direction == -1) {
                clock->mode = DIVIDE;
                clock->multiplier = 2;
            } else if (clock->mode == DIVIDE && clock->multiplier == 2 && direction == 1) {
                clock->mode = MULTIPLY;
                clock->multiplier = 1;
            } else {
                int newValue = clock->multiplier;
                if (clock->mode == MULTIPLY) {
                    if (direction > 0) {
                        newValue *= 2;
                    } else {
                        newValue /= 2;
                    }
                } else {
                    if (direction > 0) {
                        newValue /= 2;
                    } else {
                        newValue *= 2;
                    }
                }
                if (newValue <= 99) {
                    clock->multiplier = newValue;
                }
            }
        }
    } break;
    case SUBMENU_EDIT: {
        Clock* clock = &state.clocks[state.cursor];
        switch(state.submenuCursor) {
        case 0: {
            if (clock->mode == MULTIPLY && clock->multiplier == 1 && direction == -1) {
                clock->mode = DIVIDE;
                clock->multiplier = 2;
            } else if (clock->mode == DIVIDE && clock->multiplier == 2 && direction == 1) {
                clock->mode = MULTIPLY;
                clock->multiplier = 1;
            } else {
                if (clock->mode == MULTIPLY) {
                    clock->multiplier += direction;
                } else {
                    clock->multiplier -= direction;
                }
                if (clock->multiplier > 99) {
                    clock->multiplier = 99;
                }
            }
        } break;
        case 1: {
            addMaxMin(&clock->pulseWidth, direction, 0, 100);
        } break;
        case 2: {
            addMaxMin(&clock->offset, direction, -50, 50);
        } break;
        }
    } break;
    case SLEEP: {
        state.mode = NAVIGATE;
    } break;
    }
}

/*
 * Add a delta value to an int pointer without exceeding max/min values
 */
inline void addMaxMin(int8_t* value, int8_t delta, int8_t min, int8_t max) {
    *value += delta;
    if (*value < min) {
        *value = min;
    } else if (*value > max) {
        *value = max;
    }
}

void onClockInput() {
    static uint32_t lastPressedTime = 0;

    const uint32_t currentTime = lastRecordedTime;
    const uint32_t deltaTime = currentTime - lastPressedTime;

    // if the delta is very rappid (prob an accidental double-press) ignore it
    if (deltaTime < CLOCK_INPUT_TIME_MIN) {
        return;
    }

    lastPressedTime = currentTime;

    if (deltaTime > CLOCK_INPUT_TIME_MAX) {
        return;
    }
    
    state.bpm = 60000000 / deltaTime;
}

inline void onPuasePressed() {
    paused = !paused;
}

void drawMainMenu(Adafruit_SSD1306 display, const State state) {
    display.clearDisplay();
    display.cp437(true);

    const uint8_t screenWidth = SCREEN_WIDTH;
    const uint8_t screenHeight = SCREEN_HEIGHT;

    if (state.cursor == -1) {
        // Draw BPM
        const uint8_t bpmNumberFontSize = 4;
        const uint8_t bpmLabelFontSize = 2;
        const uint8_t charWidth = 6;
        char bpmText[4];
        itoa(state.bpm, bpmText, 10);
        const uint8_t bpmStrLen = strlen(bpmText);
        const uint8_t bpmNumberWidthPx = bpmStrLen * charWidth * bpmNumberFontSize;

        const uint8_t bpmNumberHeight = 8 * bpmNumberFontSize;
        const uint8_t bpmLabelHeight  = 8 * bpmLabelFontSize;
        const uint8_t totalHeight = bpmLabelHeight + bpmNumberHeight;
        const int y1 = (screenHeight - totalHeight) / 2;
        const int x1 = (screenWidth - bpmNumberWidthPx) / 2;

        if (state.mode) {
            display.setTextColor(SSD1306_BLACK, SSD1306_WHITE);
        } else {
            display.setTextColor(SSD1306_WHITE);
        }
        display.setTextSize(bpmNumberFontSize);
        display.setCursor(x1, y1);
        display.write(bpmText);

        display.setTextColor(SSD1306_WHITE);

        const uint8_t bpmLabelWidthPx = 3 * charWidth * bpmLabelFontSize;

        const int x2 = (screenWidth - bpmLabelWidthPx) / 2;
        const int y2 = y1 + bpmNumberHeight;

        display.setTextSize(bpmLabelFontSize);
        display.setCursor(x2, y2);
        display.write("bpm");
    } else {
        // Draw menu
        const uint8_t fontSize = 3;
        display.setTextSize(fontSize);
        const uint8_t numHorizontalBoxes = 2;
        const uint8_t numVerticalBoxes = 2;
        const uint8_t boxesPerScreen = numVerticalBoxes * numHorizontalBoxes;
        const uint8_t boxWidth = screenWidth / numHorizontalBoxes;
        const uint8_t boxHeight = screenHeight / numHorizontalBoxes;
        const uint8_t startIndex = (state.cursor / boxesPerScreen) * boxesPerScreen;
        const uint8_t relativeIndex = state.cursor % boxesPerScreen;

        for (uint8_t i = 0; i < boxesPerScreen; i++) {
            const uint8_t x = i % numHorizontalBoxes;
            const uint8_t y = i / numHorizontalBoxes;
            const uint8_t screenX = x * boxWidth;
            const uint8_t screenY = y * boxHeight;

            const uint8_t index = i + startIndex;
            const bool isActiveBox = index == state.cursor;

            const uint8_t borderRadius = 8;
            const uint8_t borderWidth = 2;

            if (isActiveBox && state.mode) {
                display.setTextColor(SSD1306_BLACK);
                display.fillRoundRect(screenX, screenY, boxWidth, boxHeight, 
                    borderRadius, SSD1306_WHITE);
            } else if (isActiveBox) {
                display.setTextColor(SSD1306_WHITE);
                display.fillRoundRect(screenX, screenY, boxWidth, boxHeight, 
                    borderRadius, SSD1306_WHITE);
                display.fillRoundRect(screenX + borderWidth, screenY + borderWidth,
                    boxWidth - 2 * borderWidth, boxHeight - 2 * borderWidth, 
                    borderRadius - borderWidth, SSD1306_BLACK);
            } else {
                display.setTextColor(SSD1306_WHITE);
            }
            
            const Clock clock = state.clocks[index];
            char buffer[4];
            buffer[0] = clock.mode == DIVIDE ? '/' : 'x';
            itoa(clock.multiplier, &buffer[1], 10);
            const uint8_t textLength = strlen(buffer);

            const uint8_t textHeight = 8 * fontSize;
            const uint8_t textWidth = 6 * fontSize * textLength - fontSize;

            display.setCursor(
                screenX + (boxWidth - textWidth) / 2,
                screenY + (boxHeight - textHeight) / 2);
            

            display.write(buffer);
        }
    }

    display.display();
}


void drawSubMenu(Adafruit_SSD1306 display, const State state) {
    display.clearDisplay();
    display.cp437(true);

    const uint8_t screenWidth = SCREEN_WIDTH;
    const uint8_t screenHeight = SCREEN_HEIGHT;

    Clock* clock = &state.clocks[state.cursor];

    const uint8_t fontSize = 3;
    display.setTextSize(fontSize);

    const int8_t textHeight = fontSize * 8 * 3;
    const int8_t offsetY = state.submenuCursor == 2 
        ? (int) screenHeight - (int) textHeight
        : 0;
    
    {
        display.setTextColor(SSD1306_WHITE);

        char label[5];
        label[0] = '[';
        itoa(state.cursor + 1, label + 1, 10);
        label[2] = ']';
        label[3] = state.submenuCursor == 0 ? '>' : ' ';
        label[4] = '\0';
        
        display.setCursor(0, offsetY);
        display.write(label);

        char buffer[4];
        buffer[0] = clock->mode == DIVIDE ? '/' : 'x';
        itoa(clock->multiplier, &buffer[1], 10);

        if (state.mode == SUBMENU_EDIT && state.submenuCursor == 0) {
            display.setTextColor(SSD1306_BLACK, SSD1306_WHITE);
        }
        display.write(buffer);
    }

    {
        display.setTextColor(SSD1306_WHITE);

        char label[5] = "PW:";
        label[3] = state.submenuCursor == 1 ? '>' : ' ';
        label[4] = '\0';

        display.setCursor(0, offsetY + 8 * fontSize);
        display.write(label);
        
        char buffer[4];
        itoa(clock->pulseWidth, buffer, 10);
        
        if (state.mode == SUBMENU_EDIT && state.submenuCursor == 1) {
            display.setTextColor(SSD1306_BLACK, SSD1306_WHITE);
        }
        display.write(buffer);
    }

    {
        display.setTextColor(SSD1306_WHITE);

        char label[5] = "PS:";
        label[3] = state.submenuCursor == 2 ? '>' : ' ';
        label[4] = '\0';

        display.setCursor(0, offsetY + 2 * 8 * fontSize);
        display.write(label);
        
        char buffer[4];
        itoa(clock->offset, buffer, 10);

        if (state.mode == SUBMENU_EDIT && state.submenuCursor == 2) {
            display.setTextColor(SSD1306_BLACK, SSD1306_WHITE);
        }
        display.write(buffer);
    }


    display.display();
}


inline void drawScreenSaver(Adafruit_SSD1306 display, float elapsedFraction, bool forceRedraw) {
#if SCREEN_SAVER == SS_BLACK
    if (forceRedraw) {
        display.clearDisplay();
        display.display();
    }
#elif SCREEN_SAVER == SS_PULSE
    static uint8_t lastHeight = 0;
    static uint8_t lastWidth = 0;
    const float fraction = 2 * (elapsedFraction < 0.5 ? elapsedFraction : 1 - elapsedFraction);
    const uint8_t width = fraction * SCREEN_WIDTH;
    const uint8_t height = fraction * SCREEN_HEIGHT;
    
    if (!forceRedraw && width == lastWidth && height == lastHeight) {
        return;
    }

    lastWidth = width;
    lastHeight = height;

    display.clearDisplay();
    const int x = (SCREEN_WIDTH - width) / 2;
    const int y = (SCREEN_HEIGHT - height) / 2;
    display.drawRect(x, y, width, height, SSD1306_WHITE);
    display.display();
#elif SCREEN_SAVER == SS_BARS
    static uint8_t lastBars = 0;
    const uint8_t numBars = 4;
    uint8_t bars = numBars * elapsedFraction;
    if (!forceRedraw && bars == lastBars) {
        return;
    }
    lastBars = bars;
    
    display.clearDisplay();
    display.fillRect(SCREEN_WIDTH / numBars * bars, 0, SCREEN_WIDTH / numBars, SCREEN_HEIGHT, SSD1306_WHITE);
    // Horizontal:
    // display.fillRect(0, SCREEN_HEIGHT / numBars * bars, SCREEN_WIDTH, SCREEN_HEIGHT / numBars, SSD1306_WHITE);
    display.display();
#endif
}

/*
 * Helper function for Arduino ICR macro
 */
inline void enableInterrupt(byte pin) {
    *digitalPinToPCMSK(pin) |= bit (digitalPinToPCMSKbit(pin));  // enable pin
    PCIFR  |= bit (digitalPinToPCICRbit(pin)); // clear any outstanding interrupt
    PCICR  |= bit (digitalPinToPCICRbit(pin)); // enable interrupt for the group
}
