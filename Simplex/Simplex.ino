#include <stdint.h>
#include <SPI.h>

extern "C" {
    #include "perlin.h"
}

const uint16_t CHIP_SELECT_PIN = 8;
const uint16_t SPEED_POT_PIN = A0;

const double MIN_SPEED = 0.001;
const double MAX_SPEED = 0;
const uint16_t ANALOG_READ_MAX_VALUE = 1023;

void setup() {
    pinMode(CHIP_SELECT_PIN, OUTPUT);
    pinMode(SPEED_POT_PIN, INPUT);
    digitalWrite(CHIP_SELECT_PIN, HIGH);

    SPI.begin();
    SPI.setBitOrder(MSBFIRST);
    SPI.setDataMode(SPI_MODE0);

    Serial.begin(9600);
    
    for (int i = 0; i < 5; i++) {
        MCP4922_write(CHIP_SELECT_PIN, 0, 1);
        delay(300);
        MCP4922_write(CHIP_SELECT_PIN, 0, 0);
        delay(300);
    }
}

double speed = 0;

void loop() {
    static double index = 0;
    static uint32_t lastTime = micros();
    uint32_t now = micros();
    uint32_t dt = now - lastTime;
    lastTime = now;
    index += dt * speed;
    
    //double noise = noise1d(index );
    double noise = noise2d(index / 100.0, index / 100.0);
    Serial.println(noise * 3);
    MCP4922_write(CHIP_SELECT_PIN, 0, (noise + 1) / 2);
    //MCP4922_write(CHIP_SELECT_PIN, 1, i / 100.0);

    speed = analogReadRange(SPEED_POT_PIN, MIN_SPEED, MAX_SPEED, 0.000001);
}

double analogReadRange(const uint8_t pin, const double min, const double max, const double exp) {
    double x = analogRead(pin) / (double) ANALOG_READ_MAX_VALUE;
    if (exp != 0) {
        // x = (pow(2, x * exp) - 1) / (pow(2, exp) - 1);
        x = x * x * x * x;
    }
    return (1 - x) * min + x * max;
}

/*
 * Writes a given value to a MCP4922 DAC chip to be output as
 * a voltage.
 *
 * cs_pin - which Arduino pin to use as the CHIP SELECT pin
 *     (should be connected to the CS pin of the DAC)
 * dac - 0 or 1 - Which of the MCP4922's internal DAC channels
 *     to output to (see MCP4922 datasheet for pinout diagram)
 * value - {0..1} - The value to output as a fraction of the
 *     DAC's max/reference voltage. Converted to a 12-bit int.
 */
void MCP4922_write(int cs_pin, byte dac, float value) {
    uint16_t value12 = (uint16_t) (value * 4095);
    byte low = value12 & 0xff;
    byte high = (value12 >> 8) & 0x0f;
    dac = (dac & 1) << 7;
    digitalWrite(cs_pin, LOW);
    SPI.transfer(dac | 0x30 | high);
    SPI.transfer(low);
    digitalWrite(cs_pin, HIGH);
}
