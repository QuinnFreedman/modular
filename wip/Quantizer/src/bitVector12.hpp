#include <stdint.h>
#include <Arduino.h>

class BitVector12 {
    public:
    BitVector12(const uint16_t value) : data(value) {};
    BitVector12(const BitVector12 &other) : data(other.data) {};
    bool operator [] (const uint8_t i) const { return bitRead(data, i); }
    void set(const uint8_t i, const bool b) { bitWrite(data, i, b); }
    void toggle(const uint8_t i) { bitWrite(data, i, !bitRead(data, i)); }
    bool operator == (const BitVector12 other) const {
        return (data & MASK) == (other.data & MASK);
    }
    void shiftUp() {
        bool x = bitRead(data, 0);
        data = (data >> 1);
        bitWrite(data, 11, x);
    }
    void shiftDown() {
        bool x = bitRead(data, 11);
        data = (data << 1);
        bitWrite(data, 0, x);
    }
    private:
    uint16_t data;
    static const uint16_t MASK = 0x0FFF;
};