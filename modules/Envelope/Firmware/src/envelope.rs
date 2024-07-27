use fixed::{types::extra::U16, FixedU16};

use crate::exponential_curves::exp_curve;

#[derive(Copy, Clone)]
pub enum TriggerAction {
    None,
    GateRise,
    GateFall,
    Trigger,
}

pub struct EnvelopeState {
    pub mode: EnvelopeMode,
    pub time: u32,
    pub last_value: u16,
}

#[derive(Copy, Clone)]
pub enum EnvelopeMode {
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

pub fn ui_show_mode(state: &EnvelopeMode) -> u8 {
    match state {
        EnvelopeMode::Adsr(_) => 0b1000 as u8,
        EnvelopeMode::Acrc(_) => 0b0100,
        EnvelopeMode::AcrcLoop(_) => 0b0010,
        EnvelopeMode::AhrdLoop(_) => 0b0001,
    }
    .reverse_bits()
}

pub fn ui_show_stage(state: &EnvelopeMode) -> u8 {
    match state {
        EnvelopeMode::Adsr(phase) => match phase {
            AdsrState::Wait => 0b0000 as u8,
            AdsrState::Attack => 0b1000,
            AdsrState::Decay => 0b0100,
            AdsrState::Sustain => 0b0010,
            AdsrState::Release => 0b0001,
        },
        EnvelopeMode::Acrc(phase) => match phase {
            AcrcState::Wait => 0b0000,
            AcrcState::Attack => 0b1100,
            AcrcState::Release => 0b0011,
        },
        EnvelopeMode::AcrcLoop(phase) => match phase {
            AcrcLoopState::Attack => 0b1100,
            AcrcLoopState::Release => 0b0011,
        },
        EnvelopeMode::AhrdLoop(phase) => match phase {
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
Transforms a raw cv value into a fixed point number between 0 and 1.
- Inverts value to compensate for the inverting amplifier in hardware
- Shifts values slightly to account for the fact that the input voltage
    is limited to a slightly smaller range than the DAC can read
*/
fn read_cv_linear(cv: u16) -> (FixedU16<U16>, bool) {
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
    let cv_fraction = read_cv(cv);
    let mut actual_steps_per_cycle = (cv_fraction.numerator as u32 * MAX_STEPS_PER_CYCLE as u32)
        / cv_fraction.denominator as u32;
    if actual_steps_per_cycle == 0 {
        actual_steps_per_cycle = 1;
    }

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

pub fn update(state: &mut EnvelopeState, trigger: TriggerAction, cv: &[u16; 4]) -> (u16, bool) {
    let scale = |input: u32| (input >> 20) as u16;

    let time = &mut state.time;

    match state.mode {
        EnvelopeMode::Adsr(ref mut phase) => match phase {
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
        EnvelopeMode::Acrc(ref mut phase) => match phase {
            AcrcState::Wait => (0, false),    // TODO
            AcrcState::Attack => (0, false),  // TODO
            AcrcState::Release => (0, false), // TODO
        },
        EnvelopeMode::AcrcLoop(ref mut phase) => match phase {
            AcrcLoopState::Attack => {
                let (t, rollover) = acrc_segment(time, cv[0], cv[1], false);
                if rollover {
                    *phase = AcrcLoopState::Release;
                }
                (t, rollover)
            }
            AcrcLoopState::Release => {
                let (t, rollover) = acrc_segment(time, cv[2], cv[3], true);
                if rollover {
                    *phase = AcrcLoopState::Attack;
                }
                (t, rollover)
            }
        },
        EnvelopeMode::AhrdLoop(ref mut phase) => match phase {
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

fn acrc_segment(time: &mut u32, raw_cv_len: u16, raw_cv_c: u16, invert: bool) -> (u16, bool) {
    let (t, rollover) = step_time(time, raw_cv_len);
    let (c_fixed, c_negative) = read_cv_linear(raw_cv_c);
    let t_fixed = FixedU16::<U16>::from_bits((t >> 16) as u16);
    let value = exp_curve(t_fixed, c_fixed, c_negative);
    const DAC_MAX_VALUE: u16 = 0xfff;
    debug_assert!(value <= DAC_MAX_VALUE);

    (if invert { DAC_MAX_VALUE - value } else { value }, rollover)
}
