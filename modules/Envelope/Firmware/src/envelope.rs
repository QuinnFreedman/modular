use libm::exp2f;

#[derive(Copy, Clone)]
pub enum EnvelopeState {
    Adsr(AdsrState),
    Acrc(AcrcState),
    AcrcLoop(AcrcLoopState),
    AhrdLoop(AhrdState),
}

#[derive(Copy, Clone, Default)]
pub enum AdsrState {
    #[default]
    Wait,
    Attack,
    Decay,
    Sustain,
    Release,
}

#[derive(Copy, Clone, Default)]
pub enum AhrdState {
    #[default]
    Attack,
    Hold,
    Release,
    Delay,
}

#[derive(Copy, Clone, Default)]
pub enum AcrcState {
    #[default]
    Wait,
    Attack,
    Release,
}

#[derive(Copy, Clone, Default)]
pub enum AcrcLoopState {
    #[default]
    Attack,
    Release,
}

pub fn ui_show_mode(state: &EnvelopeState) -> u8 {
    match state {
        EnvelopeState::Adsr(_) => 0b1000 as u8,
        EnvelopeState::Acrc(_) => 0b0100,
        EnvelopeState::AcrcLoop(_) => 0b0010,
        EnvelopeState::AhrdLoop(_) => 0b0001,
    }
    .reverse_bits()
}

pub fn ui_show_stage(state: &EnvelopeState) -> u8 {
    match state {
        EnvelopeState::Adsr(phase) => match phase {
            AdsrState::Wait => 0b0000 as u8,
            AdsrState::Attack => 0b1000,
            AdsrState::Decay => 0b0100,
            AdsrState::Sustain => 0b0010,
            AdsrState::Release => 0b0001,
        },
        EnvelopeState::Acrc(phase) => match phase {
            AcrcState::Wait => 0b0000,
            AcrcState::Attack => 0b1100,
            AcrcState::Release => 0b0011,
        },
        EnvelopeState::AcrcLoop(phase) => match phase {
            AcrcLoopState::Attack => 0b1100,
            AcrcLoopState::Release => 0b0011,
        },
        EnvelopeState::AhrdLoop(phase) => match phase {
            AhrdState::Attack => 0b1000,
            AhrdState::Hold => 0b0100,
            AhrdState::Release => 0b0010,
            AhrdState::Delay => 0b0001,
        },
    }
    .reverse_bits()
}

#[derive(Copy, Clone)]
struct Fraction<T> {
    numerator: T,
    denominator: T,
}

/**
Transforms a raw cv value into a usable fraction of the maximum.
- Inverts value to compensate for the inverting amplifier in hardware
- Shifts values slightly to account for the fact that the input voltage
    is limited to a slightly smaller range than the DAC can read
- Applies a simple piecewise exponential curve to make the knobs more usable
*/
fn read_cv(cv: u16) -> Fraction<u16> {
    // ADC reads up to 1023, but voltage doesn't go all the way to 5v
    const MAX_ADC_VALUE: u16 = 977;
    let x = MAX_ADC_VALUE.saturating_sub(cv);

    // let exp_cv = if x < 512 { x / 2 } else { x * 3 / 2 - 512 };
    let exp_cv = if x < 512 {
        x / 4
    } else if x < 768 {
        x - 384
    } else {
        3 * x - 1920
    };

    Fraction {
        // CV is inverted in hardware; correct for that here
        numerator: exp_cv,
        // the piecewise function isn't perfect, the range is a little larger
        // than the domain. Round to 1024 for performance
        denominator: 1024, // MAX_ADC_VALUE,
    }
}

/**
Transforms a raw cv value into a usable fraction of the maximum.
- Inverts value to compensate for the inverting amplifier in hardware
- Shifts values slightly to account for the fact that the input voltage
    is limited to a slightly smaller range than the DAC can read
- only has precision of 1/32, because that is all that is needed for
    exponential curve control cv
*/
fn read_cv_linear(cv: u16) -> Fraction<i16> {
    // ADC reads up to 1023, but voltage doesn't go all the way to 5v
    const MAX_ADC_VALUE: u16 = 977;
    let x = MAX_ADC_VALUE.saturating_sub(cv);

    // let scaled = interpolate(x, 0, MAX_ADC_VALUE, 0, 64);

    let signed = x as i16 - 512;

    Fraction {
        // CV is inverted in hardware; correct for that here
        numerator: signed,
        denominator: 512, // MAX_ADC_VALUE / 2,
    }
}

fn get_delta_t(cv: u16) -> u32 {
    const MAX_PHASE_TIME_MS10: u32 = 10 * 1000 * 10;
    // 2.5kHz == .4 ms / period
    const MS10_PER_STEP: u32 = 4;
    const MAX_STEPS_PER_CYCLE: u32 = MAX_PHASE_TIME_MS10 / MS10_PER_STEP;
    let cv_fraction = read_cv(cv);
    let actual_steps_per_cycle = u32::max(
        cv_fraction.numerator as u32 * MAX_STEPS_PER_CYCLE / cv_fraction.denominator as u32,
        1,
    );
    u32::MAX / actual_steps_per_cycle
}

fn step_time(t: &mut u32, cv: u16) -> (u32, bool) {
    let dt = get_delta_t(cv);
    *t = t.saturating_add(dt);
    let rollover = *t == u32::MAX;
    let before_rollover = *t;
    if rollover {
        *t = 0;
    }
    (before_rollover, rollover)
}

pub fn update(state: &mut EnvelopeState, time: &mut u32, cv: &[u16; 4]) -> (u16, bool) {
    let scale = |input: u32| (input >> 20) as u16;

    match state {
        EnvelopeState::Adsr(ref mut phase) => match phase {
            AdsrState::Wait => (0, false),
            AdsrState::Attack => {
                // let rollover = step_time(t, cv[0]);
                // if rollover {
                //     *phase = AdsrState::Decay;
                // }
                // (0, rollover)
                (0, false) // TODO
            }
            AdsrState::Decay => {
                // let (t, rollover) = step_time(time, cv[1]);
                // if rollover {
                //     *phase = AdsrState::Sustain;
                // }
                // (scale(u32::MAX), false)
                (0, false) // TODO
            }
            AdsrState::Sustain => (0, false), // TODO
            AdsrState::Release => (0, false), // TODO
        },
        EnvelopeState::Acrc(ref mut phase) => match phase {
            AcrcState::Wait => (0, false),    // TODO
            AcrcState::Attack => (0, false),  // TODO
            AcrcState::Release => (0, false), // TODO
        },
        EnvelopeState::AcrcLoop(ref mut phase) => match phase {
            AcrcLoopState::Attack => {
                let (t, rollover) = acrc_segment_fast(time, cv[0], cv[1], false);
                if rollover {
                    *phase = AcrcLoopState::Release;
                }
                (t, rollover)
            }
            AcrcLoopState::Release => {
                let (t, rollover) = acrc_segment_fast(time, cv[2], cv[3], true);
                if rollover {
                    *phase = AcrcLoopState::Attack;
                }
                (t, rollover)
            }
        },
        EnvelopeState::AhrdLoop(ref mut phase) => match phase {
            AhrdState::Attack => {
                let (t, rollover) = step_time(time, cv[0]);
                if rollover {
                    *phase = AhrdState::Hold;
                }
                (scale(t), rollover)
            }
            AhrdState::Hold => {
                let (_, rollover) = step_time(time, cv[1]);
                if rollover {
                    *phase = AhrdState::Release;
                }
                (scale(u32::MAX), rollover)
            }
            AhrdState::Release => {
                let (t, rollover) = step_time(time, cv[2]);
                if rollover {
                    *phase = AhrdState::Delay
                }
                (scale(u32::MAX - t), rollover)
            }
            AhrdState::Delay => {
                let (_, rollover) = step_time(time, cv[3]);
                if rollover {
                    *phase = AhrdState::Attack;
                }
                (scale(0), rollover)
            }
        },
    }
}

/*
fn acrc_segment(time: &mut u32, raw_cv_len: u16, raw_cv_c: u16, invert: bool) -> (u16, bool) {
    let (t, rollover) = step_time(time, raw_cv_len);
    let t_frac = t as f32 / u32::MAX as f32;
    let cv_c_frac = read_cv(raw_cv_c);
    let c = (cv_c_frac.numerator as f32 * 2.0) / cv_c_frac.denominator as f32 - 1.0;
    let value = exp_curve(t_frac, c);
    debug_assert!(value >= 0.0);
    debug_assert!(value <= 1.1);
    const DAC_MAX_VALUE: u16 = 0xfff;
    let value_scaled = (value * DAC_MAX_VALUE as f32) as u16;
    (
        if invert {
            DAC_MAX_VALUE - value_scaled
        } else {
            value_scaled
        },
        rollover,
    )
}

/**
x is in range [0, 1]
c is in range [-1, 1], where 0 is linear
*/
fn exp_curve(x: f32, c: f32) -> f32 {
    // let base = powf(2.0, 20.0 * c);

    // if base == 1.0 {
    //     return x;
    // }

    // (powf(base, x) - 1.0) / (base - 1.0)
    let scaled_c = 20.0 * c;
    let num = exp2f(scaled_c * x) - 1.0;
    let den = exp2f(scaled_c) - 1.0;

    if den == 0.0 {
        return x;
    }

    num / den
}
*/

fn acrc_segment_fast(time: &mut u32, raw_cv_len: u16, raw_cv_c: u16, invert: bool) -> (u16, bool) {
    let (t, rollover) = step_time(time, raw_cv_len);
    let cv_c_frac = read_cv_linear(raw_cv_c);
    let negative_cv = cv_c_frac.numerator < 0;
    let c = if negative_cv {
        Fraction {
            numerator: -cv_c_frac.numerator as u16,
            denominator: cv_c_frac.denominator as u16,
        }
    } else {
        Fraction {
            numerator: cv_c_frac.numerator as u16,
            denominator: cv_c_frac.denominator as u16,
        }
    };
    let mut value = exp_curve_fast(t, &c, negative_cv);
    // TODO debug
    // let mut value = exp_curve_fast(
    //     t,
    //     &Fraction {
    //         numerator: 30, // TODO aliases at 100
    //         denominator: 512,
    //     },
    //     false,
    // );
    const DAC_MAX_VALUE: u16 = 0xfff;
    debug_assert!(value <= DAC_MAX_VALUE + 1);
    value &= DAC_MAX_VALUE;

    (if invert { DAC_MAX_VALUE - value } else { value }, rollover)
}

/**
Computes x * y / z
Tries to multiply first for maximum precision, but if that would cause an
integer overflow, divide first instead
*/
fn multiply_then_divide(x: u32, y: u32, z: u32) -> u32 {
    x.checked_mul(y)
        .map(|product| product / z)
        .unwrap_or_else(|| {
            let (max, other) = if x > y { (x, y) } else { (y, x) };
            (max / z) * other
        })
}

fn interpolate(t: u32, min_t: u32, max_t: u32, a: u32, b: u32) -> u32 {
    // a + ((b - a) as u64 * (t - min_t) as u64 / (max_t - min_t) as u64) as u32
    // debug_assert_ne!(a, b);
    // debug_assert_ne!(min_t, max_t);
    // a + multiply_then_divide(b - a, t - min_t, max_t - min_t)
    a + ((b - a) as u64 * (t - min_t) as u64 / (max_t - min_t) as u64) as u32
}

// fn exp2_fast(f: Fraction<u32>) -> u32 {
//     debug_assert!(f.numerator / f.denominator < 32);

//     if f.numerator == 0 {
//         return 1;
//     }

//     let precision = 1024;

//     // let n = (f.numerator as u64 * precision as u64 / f.denominator as u64) as u32;
//     let n = multiply_then_divide(f.numerator, precision, f.denominator);

//     // Determine the nearest lower and upper integer exponents
//     let lower_exponent = n / precision;
//     let upper_exponent = lower_exponent + 1;

//     // Compute 2^lower_exponent and 2^upper_exponent
//     let f_lower = 1 << lower_exponent;
//     let f_upper = 1 << upper_exponent;

//     // Interpolate between 2^lower_exponent and 2^upper_exponent
//     interpolate(
//         n,
//         lower_exponent * precision,
//         upper_exponent * precision,
//         f_lower,
//         f_upper,
//     )
// }

fn interpolate_frac(t: u32, min_t: u32, max_t: u32, a: u32, b: u32) -> Fraction<u32> {
    let span = max_t - min_t;
    Fraction {
        numerator: (a * span) + (b - a) * (t - min_t),
        denominator: span,
    }
}

fn exp2_fast(f: Fraction<u32>) -> Fraction<u32> {
    debug_assert!(f.numerator / f.denominator < 32);
    if f.numerator == 0 {
        return Fraction {
            numerator: 1,
            denominator: 1,
        };
    }
    let precision = 1024;
    let n = f.numerator * precision / f.denominator;
    let lower_exponent = n / precision;
    let upper_exponent = lower_exponent + 1;
    let f_lower = 1 << lower_exponent;
    let f_upper = 1 << upper_exponent;
    interpolate_frac(
        n,
        lower_exponent * precision,
        upper_exponent * precision,
        f_lower,
        f_upper,
    )
}

fn exp_curve_fast(x: u32, c: &Fraction<u16>, negative_c: bool) -> u16 {
    const CURVE_SCALE: u16 = 20;

    let x_scaled = Fraction {
        numerator: x >> 16,
        denominator: 1 << 16,
    };

    let c_scaled = Fraction {
        numerator: (c.numerator * CURVE_SCALE) as u32,
        denominator: c.denominator as u32,
    };

    let exp1 = Fraction {
        numerator: (c_scaled.numerator * x_scaled.numerator) >> 8,
        denominator: (c_scaled.denominator * x_scaled.denominator) >> 8,
    };

    let Fraction {
        numerator: a,
        denominator: b,
    } = exp2_fast(exp1);

    let Fraction {
        numerator: c,
        denominator: d,
    } = exp2_fast(c_scaled);

    let result = if negative_c {
        // simplify (a/b - 1)/(c/d - 1)
        // flip fractions for negative powers of 2
        Fraction {
            numerator: c as u64 * (a - b) as u64,
            denominator: a as u64 * (c - d) as u64,
        }
    } else {
        // simplify (a/b - 1)/(c/d - 1)
        Fraction {
            numerator: d as u64 * (a - b) as u64,
            denominator: b as u64 * (c - d) as u64,
        }
    };

    // Use linear function when denominator approaches 0 to avoid /0 error
    if result.denominator == 0 {
        return (x >> 20) as u16;
    }

    const SCALE_FACTOR: u32 = 4096;

    let scaled =
        ((result.numerator as u64 * SCALE_FACTOR as u64) / result.denominator as u64) as u32;
    debug_assert!(scaled <= SCALE_FACTOR);
    scaled as u16
}

/*
/**
computes (2^kcx - 1) / (2^kc - 1)
where:
- x is expressed as a fraction of u32::MAX
- c is a fraction [0, 1] with any denominator small enough to prevent overflow
- output is scaled between 0 and 4096

*/
fn exp_curve_fast(x: u32, c: &Fraction<u16>, negative_c: bool) -> u16 {
    const CURVE_SCALE: u16 = 20;

    let x_scaled = Fraction {
        numerator: x >> 16 as u16,
        denominator: 0xFFFF,
    };

    let c_scaled = Fraction {
        numerator: (c.numerator * CURVE_SCALE) as u32,
        denominator: c.denominator as u32,
    };

    let exp1 = Fraction {
        numerator: c_scaled.numerator * x_scaled.numerator as u32,
        denominator: c_scaled.denominator * x_scaled.denominator as u32,
    };

    // let result = if negative_c {
    //     let a = exp2_fast(exp1);
    //     let b = exp2_fast(c_scaled);
    //     Fraction {
    //         numerator: b * (a - 1),
    //         denominator: a * (b - 1),
    //     }
    // } else {
    //     Fraction {
    //         numerator: exp2_fast(exp1) - 1,
    //         denominator: exp2_fast(c_scaled) - 1,
    //     }
    // };
    let result = Fraction {
        numerator: exp2_fast(exp1) - 1,
        denominator: exp2_fast(c_scaled) - 1,
    };

    // Use linear function when denominator approaches 0 to avoid /0 error
    if result.denominator == 0 {
        return (x >> 20) as u16;
    }

    const SCALE_FACTOR: u32 = 4096;

    let scaled = result.numerator * SCALE_FACTOR / result.denominator;
    debug_assert!(scaled <= SCALE_FACTOR);
    scaled as u16
}
*/
