use std::ops::RangeInclusive;

use rand::{thread_rng, RngCore};

#[derive(Copy, Clone)]
pub struct SinWaveVoice {
    phase: f32,
    pub debug_overflow_flag: bool,
}

impl SinWaveVoice {
    pub fn new() -> Self {
        Self {
            phase: 0.0,
            debug_overflow_flag: false,
        }
    }

    pub fn next_sample(&mut self, freq: f32, sample_rate: u32) -> f32 {
        self.phase += freq / sample_rate as f32;

        if self.phase >= 1.0 {
            self.phase -= 1.0;
            self.debug_overflow_flag = true;
        }

        (self.phase * 2.0 * std::f32::consts::PI).sin()
    }
}

#[derive(Clone, Copy)]
pub struct PerlinSequence1D {
    phase: f32,
    last_grad: f32,
    next_grad: f32,
}

impl PerlinSequence1D {
    pub fn new() -> Self {
        Self {
            phase: 0.0,
            last_grad: 0.0,
            next_grad: Self::random_grad(),
        }
    }

    pub fn new_random() -> Self {
        Self {
            phase: 0.5,
            last_grad: Self::random_grad(),
            next_grad: Self::random_grad(),
        }
    }

    pub fn next_sample(&mut self, frequency: f32, sample_rate: u32) -> f32 {
        debug_assert_ne!(sample_rate, 0);
        self.phase += frequency / sample_rate as f32;

        if self.phase >= 1.0 {
            self.phase -= 1.0;
            self.last_grad = self.next_grad;
            self.next_grad = Self::random_grad();
        }

        let a = self.last_grad * self.phase;
        let b = self.next_grad * (self.phase - 1.0);

        let result = Self::interp(a, b, self.phase);
        debug_assert!(result <= 4.0);
        debug_assert!(result >= -4.0);
        result
    }

    fn interp(a: f32, b: f32, t: f32) -> f32 {
        let x = 10.0 * t.powi(3) - 15.0 * t.powi(4) + 6.0 * t.powi(5);
        a + x * (b - a)
    }

    fn random_grad() -> f32 {
        let h = thread_rng().next_u32() & 15;
        let grad_int = 1 + (h & 7);
        let grad = 8.0 / grad_int as f32;
        if (h & 8) != 0 {
            -grad
        } else {
            grad
        }
    }
}

pub fn lerp(range: impl Into<RangeInclusive<f32>>, t: f32) -> f32 {
    let range = range.into();
    let from: f32 = *range.start();
    let to: f32 = *range.end();
    from + t * (to - from)
}
