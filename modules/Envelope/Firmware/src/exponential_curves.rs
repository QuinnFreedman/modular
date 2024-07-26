use avr_progmem::{progmem, wrapper::ProgMem};
use fixed::{types::extra::U16, FixedU16, FixedU32};

progmem! {
    pub static progmem EXP_LUT: [u8; 1024] = *include_bytes!("../exp2lut.bin");
}

fn lut_load_value(i: usize, lut: &ProgMem<[u8; 1024]>) -> FixedU32<U16> {
    debug_assert!(i < lut.len() / 4);
    let bytes: [u8; 4] = lut.load_sub_array::<4>(4 * i);
    FixedU32::<U16>::from_le_bytes(bytes)
}

/**
Returns 2^16x by finding the two nearest entries in the lookup table and
interpolating between them.
*/
fn exp2_lut(x: FixedU16<U16>) -> FixedU32<U16> {
    let idx_low = x.to_bits() >> 8;
    let idx_high = u16::min(255, idx_low + 1);
    let remainder = FixedU32::<U16>::from_bits((x.to_bits() << 8) as u32);
    let v_low = lut_load_value(idx_low as usize, &EXP_LUT);
    let v_high = lut_load_value(idx_high as usize, &EXP_LUT);
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
