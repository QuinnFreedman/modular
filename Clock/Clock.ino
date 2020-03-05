#include <stdint.h>
#include <Adafruit_GFX.h>
#include <Adafruit_SSD1306.h>

const int BUTTON_PIN = 2;
const int LED_PIN_1 = 12;
const int LED_PIN_2 = 11;

const int ENCODER_PIN_A = 3;
const int ENCODER_PIN_B = 4;

const int NUM_OUTPUTS = 8;

#define SCREEN_WIDTH 128 // OLED display width, in pixels
#define SCREEN_HEIGHT 64 // OLED display height, in pixels
#define OLED_RESET     4 // Reset pin # (or -1 if sharing Arduino reset pin)
#define SCREEN_I2C_ADDRESS 0x3C // Change to correct address for your screen

typedef enum {NAVIGATE, EDIT_FAST, EDIT_SLOW, SLEEP} MenuMode;

typedef struct {
    int bpm;
    int values[NUM_OUTPUTS];
    int cursor;
    MenuMode mode;
} State;

State state;

Adafruit_SSD1306 display(SCREEN_WIDTH, SCREEN_HEIGHT, &Wire, OLED_RESET);

void setup() {
    pinMode(BUTTON_PIN, INPUT_PULLUP);
    pinMode(LED_PIN_1, OUTPUT);
    pinMode(LED_PIN_2, OUTPUT);
	pinMode(ENCODER_PIN_A, INPUT_PULLUP);
	pinMode(ENCODER_PIN_B, INPUT_PULLUP);
    attachInterrupt(digitalPinToInterrupt(BUTTON_PIN), buttonChangeHandler, CHANGE);
    attachInterrupt(digitalPinToInterrupt(ENCODER_PIN_A), rotaryEncoderHandler, CHANGE);
    Serial.begin(9600);
 
    if(!display.begin(SSD1306_SWITCHCAPVCC, SCREEN_I2C_ADDRESS)) {
        Serial.println(F("SSD1306 allocation failed"));
        // Loop forever
        while (true) {}
    }

    state = {
        .bpm = 128,
        .values = {1, -2, -4, -8, 2, 4, 8, 16},
        .cursor = -1,
        .mode = NAVIGATE
    };
}


void loop() {
    /* display.clearDisplay(); */
    redrawMenu(display, state);
    delay(100);
}

const uint32_t LONG_PRESS_TIME = 700;

void buttonChangeHandler() {
    static uint32_t pressTime = 0;
    bool buttonPressed = digitalRead(BUTTON_PIN) == LOW;
    uint32_t now = millis();
    if (buttonPressed) {
        pressTime = now;
    } else {
        uint32_t holdTime = now - pressTime;
        if (holdTime >= LONG_PRESS_TIME) {
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
    if (currentStateCLK != lastCLK  && currentStateCLK == 1) {
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

void onLongPress() {
    switch (state.mode) {
    case NAVIGATE: 
        state.mode = EDIT_SLOW;
        break;
    default:
        state.mode = NAVIGATE;
        break;
    }
}

void onShortPress() {
    switch (state.mode) {
    case NAVIGATE: 
        state.mode = EDIT_FAST;
        break;
    default:
        state.mode = NAVIGATE;
        break;
    }
}

void onKnobTurned(int direction, int counter) {
    switch (state.mode) {
    case NAVIGATE: {
        state.cursor += direction;
        if (state.cursor < -1) {
            state.cursor = -1;
        } else if (state.cursor >= NUM_OUTPUTS) {
            state.cursor = NUM_OUTPUTS - 1;
        }
    } break;
    case EDIT_SLOW: {
        if (state.cursor == -1) {
            state.bpm += direction;
        } else {
            int value = state.values[state.cursor];
            if (value == 1 && direction == -1) {
                value = -2;
            } else if (value == -2 && direction == 1) {
                value = 1;
            } else {
                value += direction;
            }
            if (value <= 99 && value >= -99) {
                state.values[state.cursor] = value;
            }
        }
    } break;
    case EDIT_FAST: {
        if (state.cursor == -1) {
            state.bpm += direction;
        } else {
            int value = state.values[state.cursor];
            if (value == 1 && direction == -1) {
                value = -2;
            } else if (value == -2 && direction == 1) {
                value = 1;
            } else {
                if (sign(direction) == sign(value)) {
                   value *= 2;
                } else {
                   value /= 2;
                }
            }
            if (value <= 99 && value >= -99) {
                state.values[state.cursor] = value;
            }
        }
    } break;
    case SLEEP: {
        state.mode = NAVIGATE;
    } break;
    }
}

void redrawMenu(Adafruit_SSD1306 display, State state) {
    display.clearDisplay();
    display.cp437(true);

    const int screenWidth = SCREEN_WIDTH;
    const int screenHeight = SCREEN_HEIGHT;

    if (state.cursor == -1) {
        // Draw BPM
        const int bpmNumberFontSize = 4;
        const int bpmLabelFontSize = 2;
        const int charWidth = 6;
        char bpmText[5];
        itoa(state.bpm, bpmText, 10);
        const int bpmStrLen = strlen(bpmText);
        const int bpmNumberWidthPx = bpmStrLen * charWidth * bpmNumberFontSize;

        const int bpmNumberHeight = 8 * bpmNumberFontSize;
        const int bpmLabelHeight  = 8 * bpmLabelFontSize;
        const int totalHeight = bpmLabelHeight + bpmNumberHeight;
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

        const int bpmLabelWidthPx = 3 * charWidth * bpmLabelFontSize;

        const int x2 = (screenWidth - bpmLabelWidthPx) / 2;
        const int y2 = y1 + bpmNumberHeight;

        display.setTextSize(bpmLabelFontSize);
        display.setCursor(x2, y2);
        display.write("bpm");
    } else {
        // Draw menu
        const int fontSize = 3;
        display.setTextSize(fontSize);
        const int numHorizontalBoxes = 2;
        const int numVerticalBoxes = 2;
        const int boxesPerScreen = numVerticalBoxes * numHorizontalBoxes;
        const int boxWidth = screenWidth / numHorizontalBoxes;
        const int boxHeight = screenHeight / numHorizontalBoxes;
        const int startIndex = (state.cursor / boxesPerScreen) * boxesPerScreen;
        const int relativeIndex = state.cursor % boxesPerScreen;

        for (int i = 0; i < boxesPerScreen; i++) {
            const int x = i % numHorizontalBoxes;
            const int y = i / numHorizontalBoxes;
            const int screenX = x * boxWidth;
            const int screenY = y * boxHeight;

            const int index = i + startIndex;
            const bool isActiveBox = index == state.cursor;

            const int borderRadius = 8;
            const int borderWidth = 2;

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
            
            const int value = state.values[index];
            char buffer[4];
            buffer[0] = value < 0 ? '/' : 'x';
            itoa(abs(value), &buffer[1], 10);
            const int textLength = strlen(buffer);

            const int textHeight = 8 * fontSize;
            const int textWidth = 6 * fontSize * textLength - fontSize;

            display.setCursor(
                screenX + (boxWidth - textWidth) / 2,
                screenY + (boxHeight - textHeight) / 2);
            

            display.write(buffer);
        }
    }

    display.display();
}

inline int sign(int x) {
    return x < 0 ? -1 : 1;
}
