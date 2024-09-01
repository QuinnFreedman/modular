use fixed::types::{I16F16, I1F15, U0F16, U16F16};
use fm_lib::rng::ParallelLfsr;

use crate::shared::{get_delta_t, DriftModule};

pub struct PerlinModuleState {
    time: u32,
    rng: ParallelLfsr,
    pub hash_table: [u8; 256],
}

impl PerlinModuleState {
    pub fn new(random_seed: u16) -> Self {
        let mut rng = ParallelLfsr::new(random_seed);
        let hash_table = generate_permutation_table(&mut rng);
        Self {
            time: 0,
            rng,
            hash_table,
        }
    }
}

impl DriftModule for PerlinModuleState {
    fn step(&mut self, cv: &[u16; 4]) -> u16 {
        // let last_integer_time = self.time >> 16;
        let dt = u32::max(1, get_delta_t(cv[2], 0, 0) >> 16);
        self.time = self.time.saturating_add(dt) & 0xFFFFFF;
        // let integer_time = self.time >> 16;
        let time_fixed = I16F16::from_bits(self.time as i32);

        let value = perlin1d(time_fixed, &self.hash_table);

        let scaled = (value.to_bits() / 16) + (1 << 11);
        // TODO clamp might not be necessary
        scaled.clamp(0, 4095) as u16
    }
}

fn fade(t: U0F16) -> U0F16 {
    const SIX: U0F16 = U0F16::from_bits(6 << 12);
    const FIFTEEN: U0F16 = U0F16::from_bits(15 << 12);
    const TEN: U0F16 = U0F16::from_bits(10 << 12);
    let t_3 = t * t * t;
    let t_4 = t_3 * t;
    let t_5 = t_4 * t;
    let result = (TEN * t_3).saturating_add(SIX * t_5) - FIFTEEN * t_4;
    let result_bits = u16::min(result.to_bits(), 4095);
    U0F16::from_bits(result_bits << 4)
}

fn grad(hash: u8, x: I1F15) -> I1F15 {
    let h = hash & 15;
    let grad_int = 1 + (h & 7);
    let grad = I1F15::from_bits(((grad_int as u16) << 11) as i16);
    if (h & 8) != 0 {
        -grad * x
    } else {
        grad * x
    }
}

fn to_signed(x: U0F16) -> I1F15 {
    I1F15::from_bits((x.to_bits() >> 1) as i16)
}

fn perlin1d(x: I16F16, permutation: &[u8]) -> I1F15 {
    let xi = x.int().to_num::<i32>() & 255;
    let xf = U0F16::from_bits(x.frac().to_bits() as u16);

    let u = fade(xf);

    let a = permutation[xi as usize];
    let b = permutation[(xi + 1) as usize & 255];

    let xf_signed = to_signed(xf);
    const ONE: I1F15 = I1F15::from_bits(0x7FFF);
    let result = to_signed(u).lerp(grad(a, xf_signed), grad(b, xf_signed - ONE));
    I1F15::from_bits(result.to_bits() * 4)
}

fn generate_permutation_table(rng: &mut ParallelLfsr) -> [u8; 256] {
    let mut table = [0u8; 256];
    for i in 0..=255 {
        table[i as usize] = i;
    }
    shuffle(&mut table, rng);
    table
}

fn shuffle<T>(list: &mut [T], rng: &mut ParallelLfsr)
where
    T: Copy,
{
    for i in (1..list.len()).rev() {
        let j = rng.next() as usize & i;
        let temp = list[i];
        list[i] = list[j];
        list[j] = temp;
    }
}
