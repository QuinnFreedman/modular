use super::{
    shared::{lerp, read_cv, step_time, CvType},
    GateState, Input, MAX_DAC_VALUE,
};

#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub enum AdsrState {
    #[default]
    Wait,
    Attack,
    Decay,
    Sustain,
    Release,
}

pub fn adsr(
    phase: &mut AdsrState,
    time: &mut u32,
    last_value: u16,
    input: &Input,
    cv: &[u16; 4],
) -> (u16, bool) {
    if input.trigger && (*phase == AdsrState::Decay || *phase == AdsrState::Sustain) {
        *time = get_adsr_inverse_attack(last_value);
        *phase = AdsrState::Attack;
        let (value, _) = compute_adsr_value(phase, time, cv);
        return (value, true);
    }

    match input.gate {
        GateState::High | GateState::Low => compute_adsr_value(phase, time, cv),
        GateState::Rising => {
            *time = get_adsr_inverse_attack(last_value);
            *phase = AdsrState::Attack;
            let (value, _) = compute_adsr_value(phase, time, cv);
            (value, true)
        }
        GateState::Falling => {
            *phase = AdsrState::Release;
            *time = get_adsr_inverse_release(last_value);
            let (value, _) = compute_adsr_value(phase, time, cv);
            (value, true)
        }
    }
}

fn get_adsr_inverse_attack(current_value: u16) -> u32 {
    (current_value as u32) << 20
}

fn get_adsr_inverse_release(current_value: u16) -> u32 {
    ((MAX_DAC_VALUE - current_value) as u32) << 20
}

fn compute_adsr_value(phase: &mut AdsrState, time: &mut u32, cv: &[u16; 4]) -> (u16, bool) {
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
            (MAX_DAC_VALUE.saturating_sub(scale(t)), rollover)
        }
    }
}
