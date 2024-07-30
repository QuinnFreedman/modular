use fixed::{types::extra::U16, FixedU16};

use super::{
    shared::{read_cv_signed_fixed, step_time},
    TriggerAction, MAX_DAC_VALUE,
};
use crate::exponential_curves::exp_curve;

#[derive(Copy, Clone, Default)]
pub enum AcrcState {
    #[default]
    Wait,
    Attack,
    Hold,
    Release,
}

#[derive(Copy, Clone, Default)]
pub enum AcrcLoopState {
    #[default]
    Attack,
    Release,
}

pub fn acrc(
    phase: &mut AcrcState,
    time: &mut u32,
    last_value: u16,
    trigger: TriggerAction,
    cv: &[u16; 4],
) -> (u16, bool) {
    match trigger {
        TriggerAction::None => compute_acrc_value(phase, time, cv),
        TriggerAction::GateRise => {
            *time = get_acrc_inverse_attack(last_value);
            *phase = AcrcState::Attack;
            let (value, _) = compute_acrc_value(phase, time, cv);
            (value, true)
        }
        TriggerAction::GateFall => {
            *phase = AcrcState::Release;
            *time = get_acrc_inverse_release(last_value);
            let (value, _) = compute_acrc_value(phase, time, cv);
            (value, true)
        }
        TriggerAction::Trigger => {
            // TODO handle ping trigger
            let (value, _) = compute_acrc_value(phase, time, cv);
            (value, true)
        }
    }
}

fn get_acrc_inverse_attack(last_value: u16) -> u32 {
    // TODO calculate inverse
    0
}

fn get_acrc_inverse_release(last_value: u16) -> u32 {
    // TODO calculate inverse
    0
}

fn compute_acrc_value(phase: &mut AcrcState, time: &mut u32, cv: &[u16; 4]) -> (u16, bool) {
    match phase {
        AcrcState::Wait => (0, false),
        AcrcState::Attack => {
            let (t, rollover) = acrc_segment(time, cv[0], cv[1], false);
            if rollover {
                *phase = AcrcState::Hold;
            }
            (t, rollover)
        }
        AcrcState::Hold => (MAX_DAC_VALUE, false),
        AcrcState::Release => {
            let (t, rollover) = acrc_segment(time, cv[2], cv[3], true);
            if rollover {
                *phase = AcrcState::Wait;
            }
            (t, rollover)
        }
    }
}

pub fn acrc_loop(phase: &mut AcrcLoopState, time: &mut u32, cv: &[u16; 4]) -> (u16, bool) {
    match phase {
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
    }
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
