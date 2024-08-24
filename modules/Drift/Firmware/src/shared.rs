use avr_progmem::{progmem, wrapper::ProgMem};
use fixed::{types::extra::U16, FixedU16, FixedU32};

const LUT_SIZE: usize = 256;
const U32_BYTES: usize = u32::BITS as usize / 8;
progmem! {
    pub static progmem EXP_LUT: [u8; LUT_SIZE * U32_BYTES] = *include_bytes!("../exp2lut.bin");
}

fn lut_load_fixed32(i: usize, lut: &ProgMem<[u8; LUT_SIZE * U32_BYTES]>) -> FixedU32<U16> {
    debug_assert!(i < lut.len() / U32_BYTES);
    let bytes: [u8; U32_BYTES] = lut.load_sub_array::<U32_BYTES>(U32_BYTES * i);
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
    let v_low = lut_load_fixed32(idx_low as usize, &EXP_LUT);
    let v_high = lut_load_fixed32(idx_high as usize, &EXP_LUT);
    remainder.lerp(v_low, v_high)
}

fn decihertz_from_cv_vpo(raw_adc_value: u16) -> FixedU32<U16> {
    /*
    volts = (adc_value / ADC_MAX_VALUE) * 5
    octaves = volts
        > scale octaves to (0.16) * 16
        octaves_scaled = (adc_value / 1024 * 5) / 16 * 2^16
        octaves_scaled = adc_value * 20
    hz = base * 2^octaves
    base = 1/10          ; 1/10th hz at 0 volts
    */
    exp2_lut(FixedU16::<U16>::from_bits(raw_adc_value * 20))
}

fn divided_by(num: u32, denom: FixedU32<U16>) -> u32 {
    // It seems like there should be a more efficient way to do this
    // without the 64 bit division
    let num64: u64 = (num as u64) << 16;
    (num64 / denom.to_bits() as u64) as u32
}

/**
Gets the time constant for the frequency given by the knob and cv inputs.
- `knob` is the raw ADC reading of the knob position [0,1023] scaled as if it spanned 12v
- `cv` is the raw ADC reading of the CV input [0,1023], spanning [0,5] volts
- `offset` is a signed delta (-2^15,2^15), scaled to represent +/- 2.5 volts

All inputs are summed, clamped to the 0-12v range, and tract 1v/oct.
The result is a unit-less number to increment the time counter each sample so that
the 32-bit counter will roll over at the given frequency
*/
pub fn get_delta_t(knob: u16, cv: u16, offset: i16) -> u32 {
    const MICROS_PER_SECOND: u32 = 1_000_000;
    // knob scaled as if it spanned 12v
    let knob_12v = (knob * 12) / 5;
    const MAX_KNOB_VALUE: u16 = (1023 * 12) / 5;
    let mut sum = knob_12v + cv;
    sum = sum.saturating_add_signed(offset / 64);
    sum = u16::min(sum, MAX_KNOB_VALUE);

    let decihertz = decihertz_from_cv_vpo(sum);

    // min freq = 1/40 hz. Max freq = 100Hz
    const MIN_HERTZ_RECIP: u32 = 40;
    let micros_per_cycle: u32 = divided_by(MICROS_PER_SECOND * MIN_HERTZ_RECIP, decihertz);

    // ~2.27kHz sample rate == .48 ms / sample
    const MICROS_PER_SAMPLE: u32 = 480;

    let samples_pre_cycle = micros_per_cycle / MICROS_PER_SAMPLE;

    u32::MAX / samples_pre_cycle
}

#[derive(Copy, Clone)]
pub struct Fraction<T> {
    pub numerator: T,
    pub denominator: T,
}

pub trait DriftModule {
    /**
    Advance the module one time step and compute the output at that point.
    Returns a value between 0 and 4095.
    */
    fn step(&mut self, cv: &[u16; 4]) -> u16;
}
