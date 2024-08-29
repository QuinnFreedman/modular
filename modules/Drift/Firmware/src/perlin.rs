use fixed::types::{I16F16, U16F16};
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

        const HALF: u16 = 1 << 11;
        debug_assert!(value <= 8);
        debug_assert!(value >= -8);
        if value > 0 {
            let unsigned = u32::min(value.to_bits() as u32, (8 << 16) - 1);
            HALF + (unsigned >> 8) as u16
        } else {
            let unsigned = u32::min(value.abs().to_bits() as u32, (8 << 16) - 1);
            HALF - (unsigned >> 8) as u16
        }
    }
}

fn fade(t: I16F16) -> I16F16 {
    const SIX: I16F16 = I16F16::from_bits(6 << 16);
    const FIFTEEN: I16F16 = I16F16::from_bits(15 << 16);
    const TEN: I16F16 = I16F16::from_bits(10 << 16);
    t * t * t * (t * (t * SIX - FIFTEEN) + TEN)
}

fn grad(hash: u8, x: I16F16) -> I16F16 {
    let h = hash & 15;
    let grad = I16F16::from_num(1 + (h & 7));
    if (h & 8) != 0 {
        -grad * x
    } else {
        grad * x
    }
}

fn perlin1d(x: I16F16, permutation: &[u8]) -> I16F16 {
    let xi = x.int().to_num::<i32>() & 255;
    let xf = x.frac();

    let u = fade(xf);

    let a = permutation[xi as usize] as usize;
    let b = permutation[(xi + 1) as usize & 255] as usize;

    // TODO why is u not in [0,1]?
    u.saturating_lerp(
        grad(permutation[a], xf),
        grad(permutation[b], xf - I16F16::from_num(1)),
    )
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
