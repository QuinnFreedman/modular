use fixed::types::{I16F16, I1F15, U0F16, U16F16};
use fm_lib::rng::ParallelLfsr;

use crate::shared::{get_delta_t, DriftModule};

pub struct PerlinModuleState {
    time: u32,
    last_grad: I1F15,
    next_grad: I1F15,
    rng: ParallelLfsr,
}

impl PerlinModuleState {
    pub fn new(random_seed: u16) -> Self {
        let mut rng = ParallelLfsr::new(random_seed);
        let last_grad = random_grad(&mut rng);
        let next_grad = random_grad(&mut rng);

        Self {
            time: 0,
            rng,
            last_grad,
            next_grad,
        }
    }

    fn step_time(&mut self, knob: u16, cv: u16) {
        let dt = get_delta_t(knob, cv, 0);
        let (new_time, rollover) = self.time.overflowing_add(dt);
        self.time = new_time;
        if rollover {
            self.last_grad = self.next_grad;
            self.next_grad = random_grad(&mut self.rng);
        }
    }
}

impl DriftModule for PerlinModuleState {
    fn step(&mut self, cv: &[u16; 4]) -> u16 {
        self.step_time(cv[2], 0 /* TODO read cv */);

        let time_fixed = U0F16::from_bits((self.time >> 16) as u16);

        let value = perlin_segment(time_fixed, self.last_grad, self.next_grad);

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

fn perlin_segment(x: U0F16, last_grad: I1F15, next_grad: I1F15) -> I1F15 {
    const ONE: I1F15 = I1F15::from_bits(0x7FFF);

    let u = fade(x);

    let xf_signed = to_signed(x);
    let a = last_grad * xf_signed;
    let b = next_grad * (xf_signed - ONE);

    to_signed(u).lerp(a, b) * 4
}
