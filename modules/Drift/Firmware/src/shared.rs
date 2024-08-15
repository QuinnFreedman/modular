pub fn get_delta_t(cv: u16) -> u32 {
    // 10 seconds
    const MAX_PHASE_TIME_MICROS: u32 = 10 * 1000 * 1000;
    // ~2.27kHz == .48 ms / period
    const MICROS_PER_STEP: u32 = 480;
    const MAX_STEPS_PER_CYCLE: u16 = (MAX_PHASE_TIME_MICROS / MICROS_PER_STEP) as u16;

    let cv_frac = exp_cv_from_knob(1023 - cv);
    let mut actual_steps_per_cycle =
        (cv_frac.numerator as u32 * MAX_STEPS_PER_CYCLE as u32) / cv_frac.denominator as u32;
    if actual_steps_per_cycle == 0 {
        actual_steps_per_cycle = 1;
    }

    u32::MAX / actual_steps_per_cycle
}

#[derive(Copy, Clone)]
pub struct Fraction<T> {
    pub numerator: T,
    pub denominator: T,
}

/**
Maps an input ADC reading from 0-1023 to a fraction through a piecewise
exponential curve approximation
*/
pub fn exp_cv_from_knob(cv: u16) -> Fraction<u16> {
    // TODO adjust this curve so that the range is 0-1024
    // to be able to use bit shift for division
    let numerator = if cv < 512 {
        cv / 4
    } else if cv < 768 {
        cv - 384
    } else {
        3 * cv - 1920
    };

    Fraction {
        numerator,
        denominator: 1149,
    }
}

pub trait DriftModule {
    /**
    Advance the module one time step and compute the output at that point.
    Returns a value between 0 and 4095.
    */
    fn step(&mut self, cv: &[u16; 4]) -> u16;
}
