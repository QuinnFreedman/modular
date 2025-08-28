use core::f32::consts::PI;

use fixed::types::I1F31;
use rp_pico::hal::rom_data::float_funcs::{fcos, fsin};

struct FilterCoefficients {
    a1: I1F31,
    a2: I1F31,
    b0: I1F31,
    b1: I1F31,
    b2: I1F31,
}

pub struct BiquadBandPass {
    coeffs: FilterCoefficients,
    d1: I1F31,
    d2: I1F31,
}

fn get_coeffs(fs: f32, f0: f32, q: f32) -> FilterCoefficients {
    assert!(fs > 0.0 && f0 > 0.0 && q > 0.0);
    let w0 = 2.0 * PI * f0 / fs;
    let alpha = (fsin(w0)) / (2.0 * q);

    // RBJ coefficients (float)
    let b0 = alpha;
    let b1 = 0.0;
    let b2 = -alpha;
    let a0 = 1.0 + alpha;
    let a1 = -2.0 * fcos(w0);
    let a2 = 1.0 - alpha;

    FilterCoefficients {
        a1: I1F31::saturating_from_num(a1 / a0),
        a2: I1F31::saturating_from_num(a2 / a0),
        b0: I1F31::saturating_from_num(b0 / a0),
        b1: I1F31::saturating_from_num(b1 / a0),
        b2: I1F31::saturating_from_num(b2 / a0),
    }
}

impl BiquadBandPass {
    pub fn step(&mut self, x: I1F31) -> I1F31 {
        let y = self.coeffs.b0 * x + self.d1;
        self.d1 = self.coeffs.b1 * x - self.coeffs.a1 * y + self.d2;
        self.d2 = self.coeffs.b2 * x - self.coeffs.a2 * y;
        y
    }

    pub fn new(sample_hz: f32, filter_hz: f32, filter_q: f32) -> Self {
        Self {
            coeffs: get_coeffs(sample_hz, filter_hz, filter_q),
            d1: I1F31::ZERO,
            d2: I1F31::ZERO,
        }
    }
}
