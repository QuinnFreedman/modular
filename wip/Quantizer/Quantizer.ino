#include <stdint.h>
#include <SPI.h>
#include <Tlc5940.h>

const uint8_t BUTTON_LADDER_PIN = A0;
const uint8_t ANALOG_INPUT_PIN_A = A6;
const uint8_t ANALOG_INPUT_PIN_B = A7;
const uint8_t DAC_CS_PIN = 8;

const uint16_t LED_BRIGHT = (pow(2, 12) - 1) / 10;
const uint16_t LED_DIM = LED_BRIGHT / 4;
const uint16_t LED_OFF = 0;

const uint16_t ANALOG_READ_MAX_VALUE = 1023;

void setup() {
    Tlc.init();
    Tlc.clear();
    Tlc.update();
    
    SPI.begin();
    SPI.setBitOrder(MSBFIRST);
    SPI.setDataMode(SPI_MODE0);
    
    Serial.begin(9600);
    pinMode(BUTTON_LADDER_PIN, INPUT);
    pinMode(DAC_CS_PIN, OUTPUT);
    pinMode(ANALOG_INPUT_PIN_A, INPUT);
    pinMode(ANALOG_INPUT_PIN_B, INPUT);
}

float readInputVoltage(uint8_t pin) {
    uint16_t rawValue = analogRead(pin);
    return (rawValue / (float) ANALOG_READ_MAX_VALUE) * 20.0 - 10.0;
}

int i = 0;
void loop() {
    uint16_t value = analogRead(BUTTON_LADDER_PIN);
    float fvalue = value / 1023.0;
    //Serial.println(fvalue);

    float voltInA = readInputVoltage(ANALOG_INPUT_PIN_A);
    float voltInB = readInputVoltage(ANALOG_INPUT_PIN_B);
    /*
    Serial.print(voltInA);
    Serial.print(" ");
    Serial.print(voltInB);
    Serial.println();
    */

    i++;
    if (i > 100) { i = 0; }
    MCP4922_write(DAC_CS_PIN, 0, i / 100.0);
    MCP4922_write(DAC_CS_PIN, 1, i / 100.0);
    Serial.println(i / 100.0);
    delay(10);

    /*
    delay(2);
    return;
    for (int i = 0; i <= 12; i++) {
        Tlc.set(i, LED_BRIGHT);
    }
    Tlc.update();
    delay(300);
    for (int i = 0; i <= 12; i++) {
        Tlc.set(i, LED_DIM);
    }
    Tlc.update();
    delay(300);
    for (int i = 0; i <= 12; i++) {
        Tlc.set(i, LED_OFF);
    }
    Tlc.update();
    delay(300);
    */
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
