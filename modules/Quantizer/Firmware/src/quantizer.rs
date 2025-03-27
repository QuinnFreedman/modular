use arduino_hal::prelude::*;
use fixed::{
    traits::ToFixed,
    types::{I16F16, I8F8, U8F8},
};
use ufmt::{uwrite, uwriteln};

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
        QuantizationResult {
            channel_a: self.channels[0].step(input_semitones_a, trig_a),
            channel_b: self.channels[1].step(input_semitones_b, trig_b),
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
    last_output: Option<ChannelOutput>,
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
        let should_update = self.ephemeral.last_output.is_none()
            || match self.config.sample_mode {
                SampleMode::TrackAndHold => sample_trigger,
                SampleMode::SampleAndHold => !self.ephemeral.last_trigger_input && sample_trigger,
            };
        self.ephemeral.last_trigger_input = sample_trigger;

        if should_update {
            self.ephemeral.last_output = Some(self._calculate_output(input_semitones))
        }

        // TODO set trigger output here correctly once I add that
        // (won't be the same as last output)
        self.ephemeral.last_output.clone().unwrap()
    }

    fn _calculate_output(&mut self, input_semitones: I8F8) -> ChannelOutput {
        // TODO use all channel parameters
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

        // let dp = unsafe { arduino_hal::Peripherals::steal() };
        // let pins = arduino_hal::pins!(dp);
        // let mut serial = arduino_hal::default_serial!(dp, pins, 57600);
        // uwrite!(
        //     &mut serial,
        //     "{}",
        //     (input_semitones.to_fixed::<I16F16>() * 1000).to_num::<u16>()
        // )
        // .unwrap_infallible();

        if let Some((upper_thresh, lower_thresh)) = self.calculate_hysteresis_thresholds(notes) {
            if input_semitones <= upper_thresh && input_semitones >= lower_thresh {
                // uwriteln!(&mut serial, " <").unwrap_infallible();
                return self.last_output;
            }
        }

        // uwriteln!(&mut serial, "").unwrap_infallible();

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
