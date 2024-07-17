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

pub fn update(state: &mut EnvelopeState, t: &mut u32, cv: [u16; 4]) -> (u16, bool) {
    // .2ms / cycle
    // const MAX_PHASE_TIME_MS10: u32 = 10 * 1000 * 10;
    // const MS10_PER_STEP: u32 = 2;
    // const MAX_STEPS_PER_CYCLE: u32 = MAX_PHASE_TIME_MS10 / MS10_PER_STEP;
    // const MIN_DELTA_T: u32 = u32::MAX / MAX_STEPS_PER_CYCLE;
    // const MAX_DELTA_T: u32 = u32::MAX;
    // const MAX_CV_VALUE: u32 = 0xfff;
    // const delta_t: u32 = 0xfffu16 as u32 * MAX_DELTA_T / MAX_CV_VALUE;

    const MAX_PHASE_TIME_MS10: u32 = 10 * 1000 * 10;
    const MS10_PER_STEP: u32 = 2;
    const MAX_STEPS_PER_CYCLE: u32 = MAX_PHASE_TIME_MS10 / MS10_PER_STEP;
    const MAX_CV_VALUE: u32 = 0x3ff;
    const ACTUAL_STEPS_PER_CYCLE: u32 = 50u16 as u32 * MAX_STEPS_PER_CYCLE / MAX_CV_VALUE;
    const ACTUAL_DELTA_T: u32 = u32::MAX / ACTUAL_STEPS_PER_CYCLE;

    // TODO maybe catch overflow
    *t = t.saturating_add(ACTUAL_DELTA_T);
    let rollover = *t == u32::MAX;

    let scale = |input: u32| (input >> 20) as u16;

    let result = match state {
        EnvelopeState::Adsr(ref mut phase) => match phase {
            AdsrState::Attack => {
                // if rollover {
                //     *phase = AdsrState::Decay;
                // }
                0
            }
            AdsrState::Decay => {
                if rollover {
                    *phase = AdsrState::Sustain;
                }
                scale(u32::MAX)
            }
            AdsrState::Sustain => 0, // TODO
            AdsrState::Release => 0, // TODO
        },
        EnvelopeState::Acrc(ref mut phase) => match phase {
            AcrcState::Attack => 0,
            AcrcState::Release => 0,
        },
        EnvelopeState::AcrcLoop(ref mut phase) => match phase {
            AcrcState::Attack => 0,
            AcrcState::Release => 0,
        },
        EnvelopeState::AhrdLoop(ref mut phase) => match phase {
            AhrdState::Attack => {
                if rollover {
                    *phase = AhrdState::Hold;
                }
                scale(*t)
            }
            AhrdState::Hold => {
                if rollover {
                    *phase = AhrdState::Release;
                }
                scale(u32::MAX)
            }
            AhrdState::Release => {
                if rollover {
                    *phase = AhrdState::Delay
                }
                scale(u32::MAX - *t)
            }
            AhrdState::Delay => {
                if rollover {
                    *phase = AhrdState::Attack;
                }
                scale(0)
            }
        },
    };

    if rollover {
        *t = 0;
    }
    (result, rollover)
}
