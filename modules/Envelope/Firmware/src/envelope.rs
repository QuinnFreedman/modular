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

#[derive(PartialEq, Eq)]
enum CvType {
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
fn read_cv<const CURVE: CvType>(cv: u16) -> Fraction<u16> {
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
fn read_cv_signed_fixed(cv: u16) -> (FixedU16<U16>, bool) {
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

fn get_adsr_inverse_attack(current_phase: &AdsrState, current_time: &u32) -> u32 {
    return 0;
    // match current_phase {
    //     AdsrState::Wait => todo!(),
    //     AdsrState::Attack => todo!(),
    //     AdsrState::Decay => todo!(),
    //     AdsrState::Sustain => todo!(),
    //     AdsrState::Release => todo!(),
    // }
}

pub fn update(state: &mut EnvelopeState, trigger: TriggerAction, cv: &[u16; 4]) -> (u16, bool) {
    let scale = |input: u32| (input >> 20) as u16;

    let time = &mut state.time;

    match state.mode {
        EnvelopeMode::Adsr(ref mut phase) => match trigger {
            TriggerAction::None => handle_adsr_update(time, cv, phase),
            TriggerAction::GateRise => {
                *time = get_adsr_inverse_attack(phase, time);
                *phase = AdsrState::Attack;
                let (value, _) = handle_adsr_update(time, cv, phase);
                (value, true)
            }
            TriggerAction::GateFall => {
                *phase = AdsrState::Release;
                *time = 0; // TODO calculate time from current value
                let (value, _) = handle_adsr_update(time, cv, phase);
                (value, true)
            }
            TriggerAction::Trigger => {
                // TODO handle ping trigger
                let (value, _) = handle_adsr_update(time, cv, phase);
                (value, true)
            }
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

const MAX_DAC_VALUE: u16 = 4095;

fn handle_adsr_update(time: &mut u32, cv: &[u16; 4], phase: &mut AdsrState) -> (u16, bool) {
    let scale = |input: u32| (input >> 20) as u16;
    let get_sustain = || {
        let cv_frac = read_cv::<{ CvType::Linear }>(cv[2]);
        let scaled = ((cv_frac.numerator as u32 * (MAX_DAC_VALUE + 1) as u32)
            / cv_frac.denominator as u32) as u16;
        u16::min(scaled, MAX_DAC_VALUE)
    };

    match phase {
        AdsrState::Wait => (0, false),
        AdsrState::Attack => {
            let (t, rollover) = step_time(time, cv[0]);
            if rollover {
                *phase = AdsrState::Decay;
                // TODO skip decay if sustain is maxed or decay is 0
            }
            (scale(t), rollover)
        }
        AdsrState::Decay => {
            let (t, rollover) = step_time(time, cv[1]);
            if rollover {
                *phase = AdsrState::Sustain;
            }
            let sustain = get_sustain();
            let scaled = lerp((t >> 16) as u16, sustain, MAX_DAC_VALUE);
            (sustain + (MAX_DAC_VALUE - scaled), rollover)
        }
        AdsrState::Sustain => (get_sustain(), false),
        AdsrState::Release => {
            let (t, rollover) = step_time(time, cv[3]);
            if rollover {
                *phase = AdsrState::Wait;
            }
            let sustain = get_sustain();
            let scaled = lerp((t >> 16) as u16, 0, sustain);
            (sustain.saturating_sub(scaled), rollover)
        }
    }
}

fn lerp(x: u16, min: u16, max: u16) -> u16 {
    debug_assert!(min <= max);
    let range = max - min;
    ((x as u32 * range as u32) >> 16) as u16 + min
}

fn acrc_segment(time: &mut u32, raw_cv_len: u16, raw_cv_c: u16, invert: bool) -> (u16, bool) {
    let (t, rollover) = step_time(time, raw_cv_len);
    let (c_fixed, c_negative) = read_cv_signed_fixed(raw_cv_c);
    let t_fixed = FixedU16::<U16>::from_bits((t >> 16) as u16);
    let value = exp_curve(t_fixed, c_fixed, c_negative);
    const DAC_MAX_VALUE: u16 = 0xfff;
    debug_assert!(value <= DAC_MAX_VALUE);

    (if invert { DAC_MAX_VALUE - value } else { value }, rollover)
}
