use avr_progmem::{progmem, wrapper::ProgMem};
use fixed::{types::extra::U16, FixedU16, FixedU32};

progmem! {
    pub static progmem EXP_LUT: [u8; LUT_SIZE * U32_BYTES] = *include_bytes!("../exp2lut.bin");
    pub static progmem LOG_LUT: [u8; LUT_SIZE * U16_BYTES] = *include_bytes!("../log2lut.bin");
}

const LUT_SIZE: usize = 256;
const U32_BYTES: usize = u32::BITS as usize / 8;
const U16_BYTES: usize = u16::BITS as usize / 8;

fn lut_load_fixed32(i: usize, lut: &ProgMem<[u8; LUT_SIZE * U32_BYTES]>) -> FixedU32<U16> {
    debug_assert!(i < lut.len() / U32_BYTES);
    let bytes: [u8; U32_BYTES] = lut.load_sub_array::<U32_BYTES>(U32_BYTES * i);
    FixedU32::<U16>::from_le_bytes(bytes)
}

fn lut_load_u16(i: usize, lut: &ProgMem<[u8; LUT_SIZE * U16_BYTES]>) -> u16 {
    debug_assert!(i < lut.len() / U16_BYTES);
    let bytes: [u8; U16_BYTES] = lut.load_sub_array::<U16_BYTES>(U16_BYTES * i);
    u16::from_le_bytes(bytes)
}

/**
Returns 2^16x by finding the two nearest entries in the lookup table and
interpolating between them.
*/
fn exp2_lut(x: FixedU16<U16>) -> FixedU32<U16> {
    let idx_low = x.to_bits() >> 8;
    let idx_high = u16::min(255, idx_low + 1);
    let remainder = FixedU32::<U16>::from_bits((x.to_bits() << 8) as u32);
    let v_low = lut_load_fixed32(idx_low as usize, &EXP_LUT);
    let v_high = lut_load_fixed32(idx_high as usize, &EXP_LUT);
    remainder.lerp(v_low, v_high)
}

/**
Computes the equation (2^(16xc) - 1) / (2^16c - 1)
- x and c are both positive fractions (0 <= x < 1)
- c_negative indicates whether c should be interpreted as a negative number,
which will cause the curve to bend the other direction
- returns a number between 0 and 4095 (0xFFF) inclusive
*/
pub fn exp_curve(x: FixedU16<U16>, c: FixedU16<U16>, c_negative: bool) -> u16 {
    let a = exp2_lut(x * c);
    let b = exp2_lut(c);
    const ONE: FixedU32<U16> = FixedU32::<U16>::from_bits(1u32 << 16);
    const SCALE_FACTOR: u16 = 4096;
    debug_assert!(a >= ONE);
    debug_assert!(b >= ONE);
    let (numerator, denominator) = if c_negative {
        // ((a - ONE).saturating_mul(b), (b - ONE).saturating_mul(a))
        (ONE - a.recip(), ONE - b.recip())
    } else {
        (a - ONE, b - ONE)
    };
    if denominator.is_zero() {
        const SCALE_FACTOR_FIXED: FixedU32<U16> =
            FixedU32::<U16>::from_bits((SCALE_FACTOR as u32) << 16);
        let result = (Into::<FixedU32<U16>>::into(x) * SCALE_FACTOR_FIXED).to_num::<u16>();
        debug_assert!(result < SCALE_FACTOR);
        return result;
    }
    debug_assert!(numerator <= denominator);
    if numerator == denominator {
        return SCALE_FACTOR - 1;
    }
    let result = ((numerator / denominator) * SCALE_FACTOR as u32).to_num::<u16>();
    debug_assert!(result < SCALE_FACTOR);
    return result;
}

/**
Efficiently computes the ABSOLUTE VALUE of the base 2 log of any value between
0 and 2^16, exclusive.

First computes the largest power of 2 strictly less than log2(x) by counting
leading zeros in the binary representation of x (which gives the integer part of
the result); then looks up the fractional part in a table of values of log2(n) for
n in [0.5, 1] and adds the two parts together.
*/
pub fn fixed_point_log2(mut x: FixedU32<U16>) -> FixedU32<U16> {
    debug_assert!(x != 0);

    if x < 1 {
        // It seems like there is a slight loss of precision here (and an extra computation).
        // It might be better to use a separate lookup table and interpolate here, in
        // the same way as computing 2^x (or re-purpose the existing [0.5, 1] table).
        // But I don't want to deal with that.
        // For now, I'm taking advantage of the identity that log2(x) = -log2(1/x)
        // to ensure that x is always >= 1, so the output of the function is always
        // positive
        x = x.recip();
        debug_assert!(x >= 1);
    }

    // Count the number of leading zeros to calculate floor(log2(x))
    let lz = x.leading_zeros() as u32;
    let integer_part = 15 - lz;

    // Shift x to the left to normalize it (i.e. make the MSB 1)
    let normalized = x.to_bits() << lz;

    // Get the most significant 8 bits of the normalized number to use as LUT index
    let fractional_part_index = ((normalized >> 23) & 0xFF) as usize;
    debug_assert!(fractional_part_index <= 255);

    // Lookup the fractional part from the table
    let fractional_part = lut_load_u16(fractional_part_index, &LOG_LUT) as u32;

    // Combine the integer part and the fractional part
    FixedU32::<U16>::from_bits((integer_part << 16) | fractional_part)
}

/**
Calculates the inverse of the exp_curve function s.t. exp_curve_inverse(exp_curve(x, c) / 4096, c) ~= x

The formula is log2(x * (2^c - 1) + 1) / c
*/
pub fn exp_curve_inverse(x: FixedU16<U16>, c: FixedU16<U16>, c_negative: bool) -> FixedU16<U16> {
    if c == 0 {
        return FixedU16::<U16>::from_bits(x.to_bits() >> 4);
    }

    const ONE: FixedU32<U16> = FixedU32::<U16>::from_bits(1u32 << 16);

    // if c is negative, the output will always work out to be positive
    // because log2(a) will be < 0, and so will the denominator
    // To keep things simple, we just ignore the +/- sign.
    // To calculate A for negative c, use identity 2^(-x) = 1/(2^x)
    // and rearrange terms to avoid subtraction underflow
    let a = if c_negative {
        let coefficient = ONE - exp2_lut(c).recip();
        ONE - (Into::<FixedU32<U16>>::into(x) * coefficient)
    } else {
        let coefficient = exp2_lut(c) - ONE;
        (Into::<FixedU32<U16>>::into(x) * coefficient) + ONE
    };

    let numerator = fixed_point_log2(a);
    let denominator = FixedU32::<U16>::from(c) * 16;

    debug_assert!(numerator <= denominator);
    debug_assert!(denominator != 0);

    FixedU16::<U16>::from_bits((numerator / denominator).to_bits() as u16)
}
