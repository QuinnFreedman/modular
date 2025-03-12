use fixed::types::I8F8;

pub struct QuantizerState {
    pub channels_linked: bool,
    pub channel_b_mode: PitchMode,
    pub channels: [QuantizerChannel; 2],
}

impl QuantizerState {
    pub fn step(&mut self, input_semitones_a: I8F8, input_semitones_b: I8F8) -> QuantizationResult {
        QuantizationResult {
            channel_a: self.channels[0].step(input_semitones_a),
            channel_b: self.channels[1].step(input_semitones_b),
        }
    }
}

pub struct QuantizerChannel {
    pub notes: [bool; 12],
    pub sample_mode: SampleMode,
    pub glide_amount: u8,
    pub trigger_delay_amount: u8,
    pub pre_shift: i8,
    pub scale_shift: i8,
    pub post_shift: i8,
    state: HysteresisState,
}

pub enum PitchMode {
    Relative,
    Absolute,
}

pub enum SampleMode {
    TrackAndHold,
    SampleAndHold,
}

struct HysteresisState {
    last_input: I8F8,
    last_output: i8,
}

impl QuantizerChannel {
    pub const fn new() -> Self {
        Self {
            notes: [false; 12],
            sample_mode: SampleMode::TrackAndHold,
            glide_amount: 0,
            trigger_delay_amount: 0,
            pre_shift: 0,
            scale_shift: 0,
            post_shift: 0,
            state: HysteresisState {
                last_input: I8F8::ZERO,
                last_output: 0,
            },
        }
    }

    fn step(&mut self, input_semitones: I8F8) -> u8 {
        // TODO use channel parameters
        self.state.quantize(input_semitones, &self.notes)
    }
}

impl QuantizerState {
    pub const fn new() -> Self {
        Self {
            channels: [const { QuantizerChannel::new() }; 2],
            channels_linked: false,
            channel_b_mode: PitchMode::Absolute,
        }
    }
}

impl HysteresisState {
    fn quantize(&mut self, input_semitones: I8F8, notes: &[bool; 12]) -> u8 {
        if notes.iter().all(|x| !x) {
            return 0;
        }

        assert!(input_semitones >= I8F8::ZERO);

        // TODO save hysteresis bounds to state, short-circuit if within bounds; otherwise update bounds on quantize

        let mut delta = 0u8;
        loop {
            let floor = input_semitones.int();
            let upper_bound = (floor + I8F8::ONE)
                .to_num::<u8>()
                .saturating_add(delta)
                .min(120);
            let lower_bound = floor.to_num::<u8>().saturating_sub(delta);
            let mut bounds = [lower_bound, upper_bound];
            if input_semitones.frac() >= I8F8::ONE / 2 {
                bounds.reverse();
            }

            for bound in bounds {
                if notes[(bound % 12) as usize] {
                    return bound;
                }
            }

            delta += 1;
        }
    }
}

pub struct QuantizationResult {
    pub channel_a: u8,
    pub channel_b: u8,
}
