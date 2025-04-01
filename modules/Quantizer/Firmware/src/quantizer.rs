use fixed::{
    traits::FromFixed,
    types::{I8F24, I8F8},
};

pub struct QuantizerState {
    pub channels_linked: bool,
    pub channel_b_mode: PitchMode,
    pub channels: [QuantizerChannel; 2],
}

impl QuantizerState {
    pub fn step(
        &mut self,
        input_semitones_a: I8F8,
        input_semitones_b: I8F8,
        trig_a: bool,
        trig_b: bool,
    ) -> QuantizationResult {
        let input_b = match self.channel_b_mode {
            PitchMode::Relative => input_semitones_a
                .saturating_add(input_semitones_b)
                .min(I8F8::from_num(120)),
            PitchMode::Absolute => input_semitones_b,
        };

        QuantizationResult {
            channel_a: self.channels[0].step(input_semitones_a, trig_a),
            channel_b: self.channels[1].step(input_b, trig_b),
        }
    }
}

pub struct QuantizerChannel {
    pub config: ChannelConfig,
    ephemeral: ChannelState,
}

pub struct ChannelConfig {
    pub notes: [bool; 12],
    pub sample_mode: SampleMode,
    pub glide_amount: u8,
    pub trigger_delay_amount: u8,
    pub pre_shift: i8,
    pub scale_shift: i8,
    pub post_shift: i8,
}

struct ChannelState {
    last_output: Option<InternalChannelOutput>,
    last_trigger_input: bool,
    hysteresis_state: HysteresisState,
}

pub enum PitchMode {
    Relative,
    Absolute,
}

#[derive(PartialEq, Eq)]
pub enum SampleMode {
    TrackAndHold,
    SampleAndHold,
}

struct HysteresisState {
    last_output: i8,
}

impl QuantizerChannel {
    pub const fn new() -> Self {
        Self {
            config: ChannelConfig {
                notes: [false; 12],
                sample_mode: SampleMode::TrackAndHold,
                glide_amount: 0,
                trigger_delay_amount: 0,
                pre_shift: 0,
                scale_shift: 0,
                post_shift: 0,
            },
            ephemeral: ChannelState {
                last_output: None,
                last_trigger_input: false,
                hysteresis_state: HysteresisState { last_output: 0 },
            },
        }
    }

    fn step(&mut self, input_semitones: I8F8, sample_trigger: bool) -> ChannelOutput {
        // NOTE: right now, does not update when current note is de-selected from scale
        // Maybe it should?
        let should_update = self.ephemeral.last_output.is_none()
            || match self.config.sample_mode {
                SampleMode::TrackAndHold => sample_trigger,
                SampleMode::SampleAndHold => !self.ephemeral.last_trigger_input && sample_trigger,
            };
        self.ephemeral.last_trigger_input = sample_trigger;

        let (nominal_semitones, glide_target) = if should_update {
            let result = self._calculate_quantization_with_transposition(input_semitones);
            (
                result.nominal_semitones,
                I8F24::from_fixed(result.actual_semitones),
            )
        } else {
            let last_output = self.ephemeral.last_output.as_ref().unwrap();
            (last_output.nominal_semitones, last_output.glide_target)
        };

        let last_actual_output = self
            .ephemeral
            .last_output
            .as_ref()
            .map(|x| x.glide_current)
            .unwrap_or(I8F24::ZERO);
        let actual_output = self._calculate_glide(last_actual_output, glide_target);
        self.ephemeral.last_output = Some(InternalChannelOutput {
            nominal_semitones,
            glide_target,
            glide_current: actual_output,
        });

        // TODO handle trigger outputs

        ChannelOutput {
            nominal_semitones,
            actual_semitones: I8F8::from_fixed(actual_output),
        }
    }

    fn _calculate_glide(&self, current: I8F24, target: I8F24) -> I8F24 {
        debug_assert!(target >= 0);
        debug_assert!(target <= 120);
        debug_assert!(current >= 0);
        debug_assert!(current <= 120);
        if current == target {
            return current;
        }

        let alpha = I8F24::ONE >> self.config.glide_amount;

        let mut delta = alpha * (target - current);
        if delta == 0 {
            delta = if target > current {
                I8F24::from_bits(1)
            } else {
                -I8F24::from_bits(1)
            }
        }
        let result = current + delta;

        if current < target {
            assert!(result <= target);
        } else {
            assert!(result >= target);
        }
        debug_assert!(result >= 0);
        result
    }

    fn _calculate_quantization_with_transposition(
        &mut self,
        input_semitones: I8F8,
    ) -> ChannelOutput {
        let pre_shifted = (input_semitones + I8F8::from_num(self.config.pre_shift))
            .clamp(I8F8::ZERO, I8F8::from_num(120));
        let quantized = self
            .ephemeral
            .hysteresis_state
            .quantize(pre_shifted, &self.config.notes);
        let scale_shifted = step_in_scale(&self.config.notes, quantized, self.config.scale_shift);
        let post_shifted = (scale_shifted + self.config.post_shift).clamp(0, 120);
        ChannelOutput {
            nominal_semitones: scale_shifted,
            actual_semitones: I8F8::from_num(post_shifted),
        }
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
    fn quantize(&mut self, input_semitones: I8F8, notes: &[bool; 12]) -> i8 {
        if notes.iter().all(|x| !x) {
            return 0;
        }

        debug_assert!(input_semitones >= 0);

        if let Some((upper_thresh, lower_thresh)) = self.calculate_hysteresis_thresholds(notes) {
            if input_semitones <= upper_thresh && input_semitones >= lower_thresh {
                // uwriteln!(&mut serial, " <").unwrap_infallible();
                return self.last_output;
            }
        }

        let floor = input_semitones.int();
        let should_round_up = input_semitones.frac() >= I8F8::ONE / 2;
        let mut upper_bound = (floor + I8F8::ONE).to_num::<i8>();
        let mut lower_bound = floor.to_num::<i8>();
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
            lower_bound = (lower_bound - 1).max(0);
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

        let hysteresis_amount = I8F8::from_num(0.4);

        let delta_up = (next_note_up - decimal_note) / 2 + hysteresis_amount;
        let delta_down = (decimal_note - next_note_down) / 2 + hysteresis_amount;

        let upper_hyst_thresh = decimal_note + delta_up;
        let lower_hyst_thresh = decimal_note - delta_down;

        Some((upper_hyst_thresh, lower_hyst_thresh))
    }
}

fn step_in_scale(notes: &[bool; 12], starting_note: i8, num_steps: i8) -> i8 {
    let direction = if num_steps < 0 {
        Direction::Negative
    } else {
        Direction::Positive
    };

    let mut note = starting_note;
    for _ in 0..num_steps {
        note = get_next_selected_note(notes, note, direction);
    }

    note
}

fn get_next_selected_note(notes: &[bool; 12], starting_note: i8, direction: Direction) -> i8 {
    let mut note = starting_note;

    loop {
        match direction {
            Direction::Positive => {
                note += 1;
                if note >= 120 {
                    return 120;
                }
            }
            Direction::Negative => {
                note -= 1;
                if note <= 0 {
                    return 0;
                }
            }
        }

        if notes[(note % 12) as usize] {
            return note;
        }
    }
}

#[derive(Clone, Copy)]
enum Direction {
    Positive,
    Negative,
}

pub struct QuantizationResult {
    pub channel_a: ChannelOutput,
    pub channel_b: ChannelOutput,
}

impl QuantizationResult {
    pub const fn zero() -> Self {
        Self {
            channel_a: ChannelOutput {
                nominal_semitones: 0,
                actual_semitones: I8F8::ZERO,
            },
            channel_b: ChannelOutput {
                nominal_semitones: 0,
                actual_semitones: I8F8::ZERO,
            },
        }
    }
}

#[derive(Clone)]
pub struct ChannelOutput {
    pub nominal_semitones: i8,
    pub actual_semitones: I8F8,
}

struct InternalQuantizationResult {
    channel_a: InternalChannelOutput,
    channel_b: InternalChannelOutput,
}

struct InternalChannelOutput {
    nominal_semitones: i8,
    glide_target: I8F24,
    glide_current: I8F24,
}
