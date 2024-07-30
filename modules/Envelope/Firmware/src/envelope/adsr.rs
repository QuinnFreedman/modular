use super::{
    shared::{lerp, read_cv, step_time, CvType},
    AdsrState, TriggerAction, MAX_DAC_VALUE,
};

pub fn adsr(
    phase: &mut AdsrState,
    time: &mut u32,
    last_value: u16,
    trigger: TriggerAction,
    cv: &[u16; 4],
) -> (u16, bool) {
    match trigger {
        TriggerAction::None => handle_adsr_update(time, cv, phase),
        TriggerAction::GateRise => {
            *time = get_adsr_inverse_attack(last_value);
            *phase = AdsrState::Attack;
            let (value, _) = handle_adsr_update(time, cv, phase);
            (value, true)
        }
        TriggerAction::GateFall => {
            *phase = AdsrState::Release;
            let sustain = get_sustain(cv[2]);
            *time = get_adsr_inverse_release(last_value, sustain);
            let (value, _) = handle_adsr_update(time, cv, phase);
            (value, true)
        }
        TriggerAction::Trigger => {
            // TODO handle ping trigger
            let (value, _) = handle_adsr_update(time, cv, phase);
            (value, true)
        }
    }
}

fn get_adsr_inverse_attack(current_value: u16) -> u32 {
    (current_value as u32) << 20
}

fn get_adsr_inverse_release(current_value: u16, sustain_level: u16) -> u32 {
    // ((current_value as u32 * 4096u32) / (4095u16.saturating_sub(sustain_level) as u32)) << 8
    // (((current_value.saturating_sub(sustain_level)) as u32 * 4095u32) / sustain_level as u32) << 8
    0
}

fn get_sustain(cv: u16) -> u16 {
    let cv_frac = read_cv::<{ CvType::Linear }>(cv);
    let scaled = ((cv_frac.numerator as u32 * (MAX_DAC_VALUE + 1) as u32)
        / cv_frac.denominator as u32) as u16;
    u16::min(scaled, MAX_DAC_VALUE)
}

fn handle_adsr_update(time: &mut u32, cv: &[u16; 4], phase: &mut AdsrState) -> (u16, bool) {
    let scale = |input: u32| (input >> 20) as u16;

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
            let sustain = get_sustain(cv[2]);
            let scaled = lerp((t >> 16) as u16, sustain, MAX_DAC_VALUE);
            (sustain + (MAX_DAC_VALUE - scaled), rollover)
        }
        AdsrState::Sustain => (get_sustain(cv[2]), false),
        AdsrState::Release => {
            let (t, rollover) = step_time(time, cv[3]);
            if rollover {
                *phase = AdsrState::Wait;
            }
            let sustain = get_sustain(cv[2]);
            let scaled = lerp((t >> 16) as u16, 0, sustain);
            (sustain.saturating_sub(scaled), rollover)
        }
    }
}
