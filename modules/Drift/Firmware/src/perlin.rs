use fixed::types::{I1F15, U0F16};
use fm_lib::rng::ParallelLfsr;

use crate::shared::{get_delta_t, DriftModule};

struct PerlinOctave {
    time: u32,
    last_grad: I1F15,
    next_grad: I1F15,
}

impl PerlinOctave {
    fn step(&mut self, rng: &mut ParallelLfsr, delta_time: u32) -> I1F15 {
        let (new_time, rollover) = self.time.overflowing_add(delta_time);
        self.time = new_time;
        if rollover {
            self.last_grad = self.next_grad;
            self.next_grad = random_grad(rng);
        }

        let time_fixed = U0F16::from_bits((self.time >> 16) as u16);

        perlin_segment(time_fixed, self.last_grad, self.next_grad)
    }
}

pub struct PerlinModuleState {
    base: PerlinOctave,
    octave: PerlinOctave,
    rng: ParallelLfsr,
}

impl PerlinModuleState {
    pub fn new(random_seed: u16) -> Self {
        let mut rng = ParallelLfsr::new(random_seed);
        let grad1 = random_grad(&mut rng);
        let grad2 = random_grad(&mut rng);
        let grad3 = random_grad(&mut rng);
        let grad4 = random_grad(&mut rng);

        Self {
            base: PerlinOctave {
                time: 0,
                last_grad: grad1,
                next_grad: grad2,
            },
            octave: PerlinOctave {
                time: 0,
                last_grad: grad3,
                next_grad: grad4,
            },
            rng,
        }
    }
}

impl DriftModule for PerlinModuleState {
    fn step(&mut self, cv: &[u16; 4]) -> u16 {
        let dt = get_delta_t(cv[2], cv[0], 0);

        let base_value = self.base.step(&mut self.rng, dt);
        let octave_value = self.octave.step(&mut self.rng, dt * 4);

        debug_assert!(base_value <= 0.25);
        debug_assert!(base_value >= -0.25);
        debug_assert!(octave_value <= 0.25);
        debug_assert!(octave_value >= -0.25);

        let blend = I1F15::from_bits((u16::min(1023, cv[3] + cv[1]) << 5) as i16);
        const ONE: I1F15 = I1F15::from_bits(0x7FFF);
        let value = base_value * 3 + base_value * (ONE - blend) + octave_value * blend;

        let scaled = (value.to_bits() / 16) + (1 << 11);
        // TODO clamp might not be necessary
        scaled.clamp(0, 4095) as u16
    }
}

/**
The smooth interpolation function used by perlin noise.

Evaluates to 10t^3 - 15t^4 + 6t^5
*/
fn fade(t: U0F16) -> U0F16 {
    // TODO: convert this function to signed I1F15. This would save a bit on converting
    // back and forth between signed and unsigned and allowing negative numbers would also
    // let me take advantage of expanding the polynomial for one fewer multiplication. But,
    // every time I try to make this work in signed I get some weird artifacts. Maybe the
    // extra bit of precision matters for so much repeated multiplication.
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

fn random_grad(rng: &mut ParallelLfsr) -> I1F15 {
    let h = rng.next() & 15;
    let grad_int = 1 + (h & 7);
    let grad = I1F15::from_bits(((grad_int as u16) << 11) as i16);
    if (h & 8) != 0 {
        -grad
    } else {
        grad
    }
}

fn to_signed(x: U0F16) -> I1F15 {
    I1F15::from_bits((x.to_bits() >> 1) as i16)
}

/**
Computes one "segment" of a 1d perlin noise curve between two given 1d gradient "vectors".
Instead of interpolating between random values, perlin noise effectively interpolates
between different gradients/slopes for a much smoother and more organic looking curve.
Returns a fixed point between -.25 and +.25
*/
fn perlin_segment(x: U0F16, last_grad: I1F15, next_grad: I1F15) -> I1F15 {
    const ONE: I1F15 = I1F15::from_bits(0x7FFF);

    let u = fade(x);

    let xf_signed = to_signed(x);
    let a = last_grad * xf_signed;
    let b = next_grad * (xf_signed - ONE);

    to_signed(u).lerp(a, b)
}
