/*
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
*/

use fixed::{
    traits::ToFixed as _,
    types::{I1F31, I2F30},
};
use rp_pico::hal::rom_data::float_funcs::{fcos, fsin};

pub struct BiquadDF2 {
    // feedforward coefficients
    b0: I2F30,
    b1: I2F30,
    b2: I2F30,
    // feedback coefficients (a0 normalized to 1)
    a1: I2F30,
    a2: I2F30,
    // state (previous inputs and outputs)
    s1: I2F30,
    s2: I2F30,
    sample_rate: i32,
}

impl BiquadDF2 {
    /// Create a new band-pass biquad.
    ///
    /// f0: center frequency in Hz (must be > 0 and < sample_rate/2)
    /// q: quality factor (must be > 0). Typical values: 0.1 .. 20
    /// sample_rate: sampling rate in Hz (must be > 0)
    ///
    /// This implements the "constant skirt gain, peak gain = Q" band-pass form
    /// (RBJ cookbook "band-pass (constant skirt gain)").
    pub fn new_bandpass(f0: f32, q: f32, sample_rate: i32) -> Self {
        assert!(f0 > 0.0, "f0 must be > 0");
        assert!(sample_rate > 0, "sample_rate must be > 0");
        assert!(q > 0.0, "Q must be > 0");
        let mut s = Self {
            b0: I2F30::ZERO,
            b1: I2F30::ZERO,
            b2: I2F30::ZERO,
            a1: I2F30::ZERO,
            a2: I2F30::ZERO,
            s1: I2F30::ZERO,
            s2: I2F30::ZERO,
            sample_rate,
        };
        s.set_bandpass_params(f0, q);
        s
    }

    /// Reset internal state (clears history).
    pub fn reset(&mut self) {
        self.s1 = I2F30::ZERO;
        self.s2 = I2F30::ZERO;
    }

    /// Update filter parameters (center frequency and Q).
    /// This recomputes coefficients but does not touch the internal state.
    pub fn set_bandpass_params(&mut self, f0: f32, q: f32) {
        assert!(f0 > 0.0, "f0 must be > 0");
        assert!(self.sample_rate > 0, "sample_rate must be > 0");
        assert!(q > 0.0, "Q must be > 0");
        let fs = self.sample_rate as f32;
        // prevent Nyquist / DC
        let nyquist = fs / 2.0;
        assert!(
            f0 < nyquist,
            "f0 must be less than Nyquist (sample_rate/2). f0: {}, nyquist: {}",
            f0,
            nyquist
        );

        // RBJ cookbook:
        // w0 = 2*pi*f0 / fs
        // alpha = sin(w0)/(2*Q)
        // For bandpass (constant skirt gain):
        // b0 =   sin(w0)/2 =   alpha
        // b1 =   0
        // b2 =  -sin(w0)/2 =  -alpha
        // a0 =   1 + alpha
        // a1 =  -2*cos(w0)
        // a2 =   1 - alpha
        //
        // Then normalize by a0.

        let w0 = 2.0 * core::f32::consts::PI * f0 / fs;
        let sin_w0 = fsin(w0);
        let cos_w0 = fcos(w0);
        let alpha = sin_w0 / (2.0 * q);

        let b0 = alpha;
        let b1 = 0.0;
        let b2 = -alpha;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos_w0;
        let a2 = 1.0 - alpha;

        // normalize so a0 = 1
        self.b0 = I2F30::saturating_from_num(b0 / a0);
        self.b1 = I2F30::saturating_from_num(b1 / a0);
        self.b2 = I2F30::saturating_from_num(b2 / a0);
        self.a1 = I2F30::saturating_from_num(a1 / a0);
        self.a2 = I2F30::saturating_from_num(a2 / a0);
    }

    /// Process a single sample (mono). Returns the filtered output.
    pub fn process_sample(&mut self, x: I1F31) -> I1F31 {
        let x: I2F30 = x.saturating_to_fixed();
        let y = self.b0 * x + self.s1;
        self.s1 = self.b1 * x + self.s2 - self.a1 * y;
        self.s2 = self.b2 * x - self.a2 * y;

        y.saturating_to_fixed()
    }

    /// Set sample rate (and update stored sample_rate). Does not change coefficients.
    /// Call `set_bandpass_params` after this if you want coefficients updated for new sample rate.
    pub fn set_sample_rate(&mut self, sr: i32) {
        assert!(sr > 0, "sample_rate must be > 0");
        self.sample_rate = sr;
    }
}
