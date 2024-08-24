use avr_progmem::{progmem, wrapper::ProgMem};
use fixed::{
    types::extra::{U15, U16},
    FixedI16,
};
use fm_lib::rng::ParallelLfsr;

const LUT_SIZE: usize = 256;
const I16_BYTES: usize = i16::BITS as usize / 8;

progmem! {
    pub static progmem ICDF_LUT: [u8; LUT_SIZE * I16_BYTES] = *include_bytes!("../icdf_lut.bin");
}

fn lut_load_i16(i: usize, lut: &ProgMem<[u8; LUT_SIZE * I16_BYTES]>) -> FixedI16<U15> {
    debug_assert!(i < lut.len() / I16_BYTES);
    let bytes: [u8; I16_BYTES] = lut.load_sub_array::<I16_BYTES>(I16_BYTES * i);
    FixedI16::<U15>::from_bits(i16::from_le_bytes(bytes))
}

fn icdf(u: u16) -> FixedI16<U15> {
    debug_assert!(u <= i16::MAX as u16);
    let idx_low = u >> 7;
    let idx_high = u16::min(LUT_SIZE as u16 - 1, idx_low + 1);

    let remainder = FixedI16::<U15>::from_bits(((u << 8) & 0x7FFF) as i16);

    let v_low = lut_load_i16(idx_low as usize, &ICDF_LUT);
    let v_high = lut_load_i16(idx_high as usize, &ICDF_LUT);
    remainder.lerp(v_low, v_high)
}

/**
Returns a random value between -1 and 1 from a distribution given by the
Inverse CDF lookup table. Right now, that is a triangular distribution but
it could be Gaussian in the future
*/
pub fn random_from_distribution(rng: &mut ParallelLfsr) -> FixedI16<U15> {
    icdf(rng.next() & 0x7FFF)
}
