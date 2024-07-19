#[derive(Copy, Clone)]
pub enum EnvelopeState {
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

pub fn ui_show_mode(state: &EnvelopeState) -> u8 {
    match state {
        EnvelopeState::Adsr(_) => 0b1000 as u8,
        EnvelopeState::Acrc(_) => 0b0100,
        EnvelopeState::AcrcLoop(_) => 0b0010,
        EnvelopeState::AhrdLoop(_) => 0b0001,
    }
    .reverse_bits()
}

pub fn ui_show_stage(state: &EnvelopeState) -> u8 {
    match state {
        EnvelopeState::Adsr(phase) => match phase {
            AdsrState::Wait => 0b0000 as u8,
            AdsrState::Attack => 0b1000,
            AdsrState::Decay => 0b0100,
            AdsrState::Sustain => 0b0010,
            AdsrState::Release => 0b0001,
        },
        EnvelopeState::Acrc(phase) => match phase {
            AcrcState::Wait => 0b0000,
            AcrcState::Attack => 0b1100,
            AcrcState::Release => 0b0011,
        },
        EnvelopeState::AcrcLoop(phase) => match phase {
            AcrcLoopState::Attack => 0b1100,
            AcrcLoopState::Release => 0b0011,
        },
        EnvelopeState::AhrdLoop(phase) => match phase {
            AhrdState::Attack => 0b1000,
            AhrdState::Hold => 0b0100,
            AhrdState::Release => 0b0010,
            AhrdState::Delay => 0b0001,
        },
    }
    .reverse_bits()
}

struct Fraction<T> {
    numerator: T,
    denominator: T,
}

/**
Transforms a raw cv value into a usable fraction of the maximum.
- Inverts value to compensate for the inverting amplifier in hardware
- Shifts values slightly to account for the fact that the input voltage
    is limited to a slightly smaller range than the DAC can read
- Applies a simple piecewise exponential curve to make the knobs more usable
*/
fn read_cv(cv: u16) -> Fraction<u16> {
    // ADC reads up to 1023, but voltage doesn't go all the way to 5v
    const MAX_ADC_VALUE: u16 = 977;
    let x = MAX_ADC_VALUE.saturating_sub(cv);

    // let exp_cv = if x < 512 { x / 2 } else { x * 3 / 2 - 512 };
    let exp_cv = if x < 512 {
        x / 4
    } else if x < 768 {
        x - 384
    } else {
        3 * x - 1920
    };

    Fraction {
        // CV is inverted in hardware; correct for that here
        numerator: exp_cv,
        // the piecewise function isn't perfect, the range is a little larger
        // than the domain. Round to 1024 for performance
        denominator: 1024, // MAX_ADC_VALUE,
    }
}

fn get_delta_t(cv: u16) -> u32 {
    const MAX_PHASE_TIME_MS10: u32 = 10 * 1000 * 10;
    const MS10_PER_STEP: u32 = 2;
    const MAX_STEPS_PER_CYCLE: u32 = MAX_PHASE_TIME_MS10 / MS10_PER_STEP;
    let cv_fraction = read_cv(cv);
    let actual_steps_per_cycle = u32::max(
        cv_fraction.numerator as u32 * MAX_STEPS_PER_CYCLE / cv_fraction.denominator as u32,
        1,
    );
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
    let scale = |input: u32| (input >> 20) as u16;

    match state {
        EnvelopeState::Adsr(ref mut phase) => match phase {
            AdsrState::Wait => (0, false),
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
            AcrcState::Wait => (0, false),
            AcrcState::Attack => (0, false),
            AcrcState::Release => (0, false),
        },
        EnvelopeState::AcrcLoop(ref mut phase) => match phase {
            AcrcLoopState::Attack => (0, false),
            AcrcLoopState::Release => (0, false),
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
