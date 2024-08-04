use fixed::{types::extra::U16, FixedU16};

use super::{
    shared::{read_cv_signed_fixed, step_time},
    GateState, Input, MAX_DAC_VALUE,
};
use crate::exponential_curves::{exp_curve, exp_curve_inverse};

const CFG_ACRC_HARD_SYNC_ON_TRIGGER: bool = false;

#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub enum AcrcState {
    #[default]
    Wait,
    Attack,
    Hold,
    Release,
}

#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub enum AcrcLoopState {
    #[default]
    Attack,
    Release,
}

pub fn acrc(
    phase: &mut AcrcState,
    time: &mut u32,
    last_value: u16,
    input: &Input,
    cv: &[u16; 4],
    artificial_gate: &mut bool,
) -> (u16, bool) {
    match input.gate {
        GateState::High => compute_acrc_value(phase, time, cv),
        GateState::Rising => {
            let (c_fixed, c_negative) = read_cv_signed_fixed(cv[1]);
            *time = get_acrc_inverse_attack(last_value, c_fixed, c_negative);
            *phase = AcrcState::Attack;
            // We don't have time to compute the inverse AND compute a new sample
            // in one cycle, so repeat the last value for one sample
            (last_value, true)
        }
        GateState::Falling => {
            let (c_fixed, c_negative) = read_cv_signed_fixed(cv[3]);
            *phase = AcrcState::Release;
            *time = get_acrc_inverse_release(last_value, c_fixed, c_negative);
            (last_value, true)
        }
        GateState::Low => {
            if input.trigger {
                *time = if CFG_ACRC_HARD_SYNC_ON_TRIGGER {
                    0
                } else {
                    let (c_fixed, c_negative) = read_cv_signed_fixed(cv[1]);
                    get_acrc_inverse_attack(last_value, c_fixed, c_negative)
                };
                *phase = AcrcState::Attack;
                *artificial_gate = true;
                return (last_value, true);
            }

            let (value, rollover) = compute_acrc_value(phase, time, cv);

            if *artificial_gate {
                if rollover && *phase == AcrcState::Hold {
                    *phase = AcrcState::Release;
                }
            }

            (value, rollover)
        }
    }
}

fn get_acrc_inverse_attack(last_value: u16, c: FixedU16<U16>, c_negative: bool) -> u32 {
    // convert from [0, 4095] to [0, 1] in (0.16)
    let last_value_frac = FixedU16::<U16>::from_bits(last_value << 4);
    let x_frac = exp_curve_inverse(last_value_frac, c, c_negative);
    (x_frac.to_bits() as u32) << 16
}

fn get_acrc_inverse_release(last_value: u16, c: FixedU16<U16>, c_negative: bool) -> u32 {
    let last_value_flipped = MAX_DAC_VALUE - last_value;
    let last_value_frac = FixedU16::<U16>::from_bits(last_value_flipped << 4);
    let x_frac = exp_curve_inverse(last_value_frac, c, c_negative);
    (x_frac.to_bits() as u32) << 16
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

pub fn acrc_loop(
    phase: &mut AcrcLoopState,
    time: &mut u32,
    input: &Input,
    cv: &[u16; 4],
) -> (u16, bool) {
    if input.trigger {
        *time = 0;
        let did_change = *phase == AcrcLoopState::Release;
        *phase = AcrcLoopState::Attack;
        return (0, did_change);
    }

    match phase {
        AcrcLoopState::Attack => {
            let (value, rollover) = acrc_segment(time, cv[0], cv[1], false);
            if rollover {
                *phase = AcrcLoopState::Release;
            }
            (value, rollover)
        }
        AcrcLoopState::Release => {
            let (value, rollover) = acrc_segment(time, cv[2], cv[3], true);
            if rollover && input.gate == GateState::High {
                *time = u32::MAX;
                return (0, false);
            }
            if rollover {
                *phase = AcrcLoopState::Attack;
            }
            (value, rollover)
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
