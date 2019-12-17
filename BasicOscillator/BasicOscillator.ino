#include <cstdint>
#include <SPI.h>

/*
//const int ledPin =  13;
const int dacChipSelectPin = 9;

uint32_t interval = 1000;

unsigned int ticks = 0;
int ledState = LOW;

uint32_t startTime;

void setup() {
    pinMode(1, OUTPUT);
    pinMode(dacChipSelectPin, OUTPUT);
    digitalWrite(dacChipSelectPin, HIGH);
    for (int i = 0; i < 5; i++) {
        digitalWrite(1, HIGH);
        delay(100);
        digitalWrite(1, LOW);
        delay(100);
    }
    Serial.begin(9600);

    SPI.begin();
    SPI.setBitOrder(MSBFIRST);
    SPI.setDataMode(SPI_MODE0);

    startTime = micros();
}

void loop(){
    digitalWrite(1, HIGH);

    float hertz = .5;
    uint32_t periodMicros = (uint32_t) (1.0 / hertz * 1000000);

    uint32_t currentTime = micros();
    uint32_t elapsed = currentTime - startTime;
    elapsed = elapsed %  periodMicros;

    float output = sin(elapsed * TWO_PI / periodMicros);
    output = (output + 1) / 2;
    writeToDac(output, 0);
}
*/

const uint16_t potentiometer = 14;

const uint16_t chip_select = 4;

uint32_t periodStart;

void setup() {
    pinMode(chip_select, OUTPUT);
    digitalWrite(chip_select, HIGH);

    pinMode(1, OUTPUT);
    for (int i = 0; i < 5; i++) {
        digitalWrite(1, HIGH);
        delay(100);
        digitalWrite(1, LOW);
        delay(100);
    }
    digitalWrite(1, HIGH);

    SPI.begin();

    pinMode(potentiometer, INPUT);

    periodStart = micros();
}

float hertz = 500;

void loop() {
    //float hertz = 500;

    uint16_t potValue = analogRead(potentiometer);
    float newHertz = potValue + 100;
    float hertzDelta = (newHertz - hertz) * 0.1;
    hertz += hertzDelta;
    uint32_t periodMicros = (uint32_t) (1.0 / hertz * 1000000);
    for (int i = 0; i < 50; i++) {
        uint32_t currentTime = micros();
        uint32_t elapsed = currentTime - periodStart;
        while (elapsed >= periodMicros) {
            periodStart = currentTime;
            elapsed -= periodMicros;
        }

        float elapsedFraction = ((float) elapsed) / ((float) periodMicros);

        uint16_t value = (uint16_t) (waveSaw(elapsedFraction) * 4095);

        MCP4922_write(chip_select, 0, value);
    }
}

float waveSin(float x) {
    if (x < .5) {
        return 0;
    } else {
        return 1;
    }
}

float waveSaw(float x) {
    return x;
}

void MCP4922_write(int cs_pin, byte dac, uint16_t value) {
    byte low = value & 0xff;
    byte high = (value >> 8) & 0x0f;
    dac = (dac & 1) << 7;
    digitalWrite(cs_pin, LOW);
    SPI.transfer(dac | 0x30 | high);
    SPI.transfer(low);
    digitalWrite(cs_pin, HIGH);
}
