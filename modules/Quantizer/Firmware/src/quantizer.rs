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
    last_output: u8,
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
            state: HysteresisState { last_output: 0 },
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

        debug_assert!(input_semitones >= I8F8::ZERO);

        if let Some((upper_thresh, lower_thresh)) = self.calculate_hysteresis_thresholds(notes) {
            if input_semitones <= upper_thresh && input_semitones >= lower_thresh {
                return self.last_output;
            }
        }

        let floor = input_semitones.int();
        let should_round_up = input_semitones.frac() >= I8F8::ONE / 2;
        let mut upper_bound = (floor + I8F8::ONE).to_num::<u8>();
        let mut lower_bound = floor.to_num::<u8>();
        loop {
            let mut bounds = [lower_bound, upper_bound];
            if should_round_up {
                bounds.reverse();
            }

            for bound in bounds {
                if notes[(bound % 12) as usize] {
                    self.last_output = bound;
                    return bound;
                }
            }

            upper_bound = (upper_bound + 1).min(120);
            lower_bound = lower_bound.saturating_sub(1);
        }
    }

    fn calculate_hysteresis_thresholds(&self, notes: &[bool; 12]) -> Option<(I8F8, I8F8)> {
        if !notes[(self.last_output % 12) as usize] {
            return None;
        }

        let next_note_up = I8F8::from_num(get_next_selected_note(
            notes,
            self.last_output,
            Direction::Positive,
        ));
        let next_note_down = I8F8::from_num(get_next_selected_note(
            notes,
            self.last_output,
            Direction::Negative,
        ));

        let decimal_note = I8F8::from_num(self.last_output);

        let hysteresis_amount = I8F8::from_num(0.2);

        let upper_hyst_thresh = decimal_note / 2 + next_note_up / 2 + hysteresis_amount;
        let lower_hyst_thresh = decimal_note / 2 + next_note_down / 2 - hysteresis_amount;

        Some((upper_hyst_thresh, lower_hyst_thresh))
    }
}

fn get_next_selected_note(notes: &[bool; 12], starting_note: u8, direction: Direction) -> u8 {
    let mut note = starting_note;

    loop {
        match direction {
            Direction::Positive => {
                note += 1;
                if note == 120 {
                    return 120;
                }
            }
            Direction::Negative => match note.checked_sub(1) {
                Some(new_note) => {
                    note = new_note;
                }
                None => return 0,
            },
        }

        if notes[(note % 12) as usize] {
            return note;
        }
    }
}

enum Direction {
    Positive,
    Negative,
}

pub struct QuantizationResult {
    pub channel_a: u8,
    pub channel_b: u8,
}
