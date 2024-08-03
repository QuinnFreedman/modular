use super::{shared::step_time, Input};

#[derive(Copy, Clone, Default)]
pub enum AhrdState {
    #[default]
    Attack,
    Hold,
    Release,
    Delay,
}

pub fn ahrd(phase: &mut AhrdState, time: &mut u32, input: &Input, cv: &[u16; 4]) -> (u16, bool) {
    let scale = |x: u32| (x >> 20) as u16;

    match phase {
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
    }
}
