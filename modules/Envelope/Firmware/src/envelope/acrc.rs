use fixed::{types::extra::U16, FixedU16};

use super::{
    shared::{read_cv_signed_fixed, step_time},
    TriggerAction,
};
use crate::exponential_curves::exp_curve;

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

pub fn acrc(
    phase: &mut AcrcState,
    time: &mut u32,
    last_value: u16,
    trigger: TriggerAction,
    cv: &[u16; 4],
) -> (u16, bool) {
    match phase {
        AcrcState::Wait => (0, false),    // TODO
        AcrcState::Attack => (0, false),  // TODO
        AcrcState::Release => (0, false), // TODO
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
