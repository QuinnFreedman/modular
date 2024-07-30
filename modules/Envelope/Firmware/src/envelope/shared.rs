use fixed::{types::extra::U16, FixedU16};

#[derive(Copy, Clone)]
pub struct Fraction<T> {
    pub numerator: T,
    pub denominator: T,
}

#[derive(PartialEq, Eq)]
pub enum CvType {
    Linear,
    Exponential,
}

impl core::marker::ConstParamTy for CvType {}

/**
Transforms a raw cv value into a usable fraction of the maximum.
- Inverts value to compensate for the inverting amplifier in hardware
- Shifts values slightly to account for the fact that the input voltage
    is limited to a slightly smaller range than the DAC can read
- Applies a simple piecewise exponential curve to make the knobs more usable
*/
pub fn read_cv<const CURVE: CvType>(cv: u16) -> Fraction<u16> {
    // ADC reads up to 1023, but voltage doesn't go all the way to 5v
    const MAX_ADC_VALUE: u16 = 977;
    // CV is inverted in hardware; correct for that here
    let x = MAX_ADC_VALUE.saturating_sub(cv);

    let numerator = match CURVE {
        CvType::Linear => x,
        CvType::Exponential => {
            if x < 512 {
                x / 4
            } else if x < 768 {
                x - 384
            } else {
                3 * x - 1920
            }
        }
    };

    let denominator = match CURVE {
        CvType::Linear => MAX_ADC_VALUE,
        // the piecewise function isn't perfect, the range is a little larger
        // than the domain. It actually goes to 1011. Round to 1024 for performance
        CvType::Exponential => 1024,
    };

    Fraction {
        numerator: u16::min(numerator, denominator),
        denominator,
    }
}

/**
Transforms a raw cv value into a fixed point number between 0 and 1.
- Inverts value to compensate for the inverting amplifier in hardware
- Shifts values slightly to account for the fact that the input voltage
    is limited to a slightly smaller range than the DAC can read
*/
pub fn read_cv_signed_fixed(cv: u16) -> (FixedU16<U16>, bool) {
    // ADC reads up to 1023, but voltage doesn't go all the way to 5v
    const MAX_ADC_VALUE: u16 = 977;
    const MIDPOINT: u16 = MAX_ADC_VALUE / 2;
    let x = MAX_ADC_VALUE.saturating_sub(cv);

    if x > MIDPOINT {
        (FixedU16::<U16>::from_bits(x - MIDPOINT << 7), false)
    } else {
        (FixedU16::<U16>::from_bits(MIDPOINT - x << 7), true)
    }
}

fn get_delta_t(cv: u16) -> u32 {
    // 10 seconds
    const MAX_PHASE_TIME_MICROS: u32 = 10 * 1000 * 1000;
    // ~2.27kHz == .48 ms / period
    const MICROS_PER_STEP: u32 = 480;
    const MAX_STEPS_PER_CYCLE: u16 = (MAX_PHASE_TIME_MICROS / MICROS_PER_STEP) as u16;
    let cv_fraction = read_cv::<{ CvType::Exponential }>(cv);
    let mut actual_steps_per_cycle = (cv_fraction.numerator as u32 * MAX_STEPS_PER_CYCLE as u32)
        / cv_fraction.denominator as u32;
    if actual_steps_per_cycle == 0 {
        actual_steps_per_cycle = 1;
    }

    u32::MAX / actual_steps_per_cycle
}

pub fn step_time(t: &mut u32, cv: u16) -> (u32, bool) {
    let dt = get_delta_t(cv);
    *t = t.saturating_add(dt);
    let rollover = *t == u32::MAX;
    let before_rollover = *t;
    if rollover {
        *t = 0;
    }
    (before_rollover, rollover)
}

pub fn lerp(x: u16, min: u16, max: u16) -> u16 {
    debug_assert!(min <= max);
    let range = max - min;
    ((x as u32 * range as u32) >> 16) as u16 + min
}
