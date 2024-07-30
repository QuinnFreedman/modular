mod adsr;
mod shared;

use fixed::{types::extra::U16, FixedU16};

use crate::{
    envelope::shared::{read_cv_signed_fixed, step_time},
    exponential_curves::exp_curve,
};
use adsr::adsr;

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

pub fn update(state: &mut EnvelopeState, trigger: TriggerAction, cv: &[u16; 4]) -> (u16, bool) {
    let scale = |input: u32| (input >> 20) as u16;

    let time = &mut state.time;

    let (value, rollover) = match state.mode {
        EnvelopeMode::Adsr(ref mut phase) => adsr(phase, time, state.last_value, trigger, cv),
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
    };

    debug_assert!(value <= MAX_DAC_VALUE);
    state.last_value = value;

    (value, rollover)
}

const MAX_DAC_VALUE: u16 = 4095;

fn acrc_segment(time: &mut u32, raw_cv_len: u16, raw_cv_c: u16, invert: bool) -> (u16, bool) {
    let (t, rollover) = step_time(time, raw_cv_len);
    let (c_fixed, c_negative) = read_cv_signed_fixed(raw_cv_c);
    let t_fixed = FixedU16::<U16>::from_bits((t >> 16) as u16);
    let value = exp_curve(t_fixed, c_fixed, c_negative);
    const DAC_MAX_VALUE: u16 = 0xfff;
    debug_assert!(value <= DAC_MAX_VALUE);

    (if invert { DAC_MAX_VALUE - value } else { value }, rollover)
}
