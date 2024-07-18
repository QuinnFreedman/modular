#[derive(Copy, Clone)]
pub enum EnvelopeState {
    Adsr(AdsrState),
    Acrc(AcrcState),
    AcrcLoop(AcrcState),
    AhrdLoop(AhrdState),
}

#[derive(Copy, Clone, Default)]
pub enum AdsrState {
    #[default]
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
    Attack,
    Release,
}

pub fn ui_show_mode(state: &EnvelopeState) -> [bool; 4] {
    // TODO use bit flags instead of bools
    match state {
        EnvelopeState::Adsr(_) => [true, false, false, false],
        EnvelopeState::Acrc(_) => [false, true, false, false],
        EnvelopeState::AcrcLoop(_) => [false, false, true, false],
        EnvelopeState::AhrdLoop(_) => [false, false, false, true],
    }
}

pub fn ui_show_stage(state: &EnvelopeState) -> [bool; 4] {
    // TODO use bit flags instead of bools
    match state {
        EnvelopeState::Adsr(phase) => match phase {
            AdsrState::Attack => [true, false, false, false],
            AdsrState::Decay => [false, true, false, false],
            AdsrState::Sustain => [false, false, true, false],
            AdsrState::Release => [false, false, false, true],
        },
        EnvelopeState::Acrc(phase) | EnvelopeState::AcrcLoop(phase) => match phase {
            AcrcState::Attack => [true, true, false, false],
            AcrcState::Release => [false, false, true, true],
        },
        EnvelopeState::AhrdLoop(phase) => match phase {
            AhrdState::Attack => [true, false, false, false],
            AhrdState::Hold => [false, true, false, false],
            AhrdState::Release => [false, false, true, false],
            AhrdState::Delay => [false, false, false, true],
        },
    }
}

fn get_delta_t(cv: u16) -> u32 {
    const MAX_PHASE_TIME_MS10: u32 = 10 * 1000 * 10;
    const MS10_PER_STEP: u32 = 2;
    const MAX_STEPS_PER_CYCLE: u32 = MAX_PHASE_TIME_MS10 / MS10_PER_STEP;
    const MAX_CV_VALUE: u32 = 0x3ff;
    // CV is inverted in hardware; correct for that here
    let inv_cv = MAX_CV_VALUE as u16 - cv;
    let actual_steps_per_cycle = u32::max(inv_cv as u32 * MAX_STEPS_PER_CYCLE / MAX_CV_VALUE, 1);
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

pub fn update(state: &mut EnvelopeState, time: &mut u32, cv: [u16; 4]) -> (u16, bool) {
    // .2ms / cycle
    // const MAX_PHASE_TIME_MS10: u32 = 10 * 1000 * 10;
    // const MS10_PER_STEP: u32 = 2;
    // const MAX_STEPS_PER_CYCLE: u32 = MAX_PHASE_TIME_MS10 / MS10_PER_STEP;
    // const MIN_DELTA_T: u32 = u32::MAX / MAX_STEPS_PER_CYCLE;
    // const MAX_DELTA_T: u32 = u32::MAX;
    // const MAX_CV_VALUE: u32 = 0xfff;
    // const delta_t: u32 = 0xfffu16 as u32 * MAX_DELTA_T / MAX_CV_VALUE;

    let scale = |input: u32| (input >> 20) as u16;

    match state {
        EnvelopeState::Adsr(ref mut phase) => match phase {
            AdsrState::Attack => {
                // let rollover = step_time(t, cv[0]);
                // if rollover {
                //     *phase = AdsrState::Decay;
                // }
                // (0, rollover)
                (0, false)
            }
            AdsrState::Decay => {
                // let (t, rollover) = step_time(time, cv[1]);
                // if rollover {
                //     *phase = AdsrState::Sustain;
                // }
                // (scale(u32::MAX), false)
                (0, false)
            }
            AdsrState::Sustain => (0, false), // TODO
            AdsrState::Release => (0, false), // TODO
        },
        EnvelopeState::Acrc(ref mut phase) => match phase {
            AcrcState::Attack => (0, false),
            AcrcState::Release => (0, false),
        },
        EnvelopeState::AcrcLoop(ref mut phase) => match phase {
            AcrcState::Attack => (0, false),
            AcrcState::Release => (0, false),
        },
        EnvelopeState::AhrdLoop(ref mut phase) => match phase {
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
