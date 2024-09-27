//! This module abstracts the core functionality of the RNG module in a
//! more-or-less hardware-independent way.

use core::cell::Cell;

use fm_lib::{
    bit_ops::BitOps,
    number_utils::{step_in_powers_of_2, ModulusSubtraction},
    rng::ParallelLfsr,
};

pub enum SizeAdjustment {
    PowersOfTwo(i8),
    ExactDelta(i8),
}

pub enum DisplayMode {
    ShowBuffer,
    ShowBufferLengthSince(u32),
}

const BUFFER_LEN_DISPLAY_TIME_MS: u32 = 3000;
const TRIG_LED_TIME_MS: u32 = 60;
const TRIG_TIME_MS: u32 = 10;
const SCALE_MAX: u16 = 0xfff;

/**
Module input parameters
Analog values are 12-bit (0-4095)
*/
pub struct RngModuleInput {
    pub chance_cv: u16,
    pub bias_cv: u16,
    pub trig_mode: bool,
    pub enable_cv: bool,
}

/**
Subset of input parameters needed on every timestep
Analog values are 12-bit (0-4095)
*/
pub struct RngModuleInputShort {
    pub chance_pot: u16,
    pub enable_cv: bool,
}

/**
Module output.
Analog values are 12-bit (0-4095)
*/
pub struct RngModuleOutput {
    pub clock_led_on: bool,
    pub enable_led_on: bool,
    pub output_a: bool,
    pub output_b: bool,
    pub analog_out: u16,
}

#[derive(PartialEq, Eq)]
pub enum Channel {
    A,
    B,
    Neither,
}

#[derive(PartialEq, Eq)]
pub enum TriggerMode {
    Trigger,
    Gate,
}

struct CurrentOutputState {
    channel: Channel,
    trigger_mode: TriggerMode,
    analog_out: u16,
    clock_trigger_time_ms: u32,
}

pub struct RngModule<const MAX_BUFFER_SIZE: u8, const NUM_LEDS: u8>
where
    [(); MAX_BUFFER_SIZE as usize]: Sized,
    [(); NUM_LEDS as usize]: Sized,
{
    pub buffer: [u16; MAX_BUFFER_SIZE as usize],
    forward_backward: bool,
    buffer_size: u8,
    cursor: u8,
    current_output: Option<CurrentOutputState>,
    display_mode: DisplayMode,
    last_rendered_led_display: Cell<u8>,
    prng: ParallelLfsr,
}

impl<const MAX_BUFFER_SIZE: u8, const NUM_LEDS: u8> RngModule<MAX_BUFFER_SIZE, NUM_LEDS>
where
    [(); MAX_BUFFER_SIZE as usize]: Sized,
    [(); NUM_LEDS as usize]: Sized,
{
    pub fn new(mut prng: ParallelLfsr) -> Self {
        let buffer: [u16; MAX_BUFFER_SIZE as usize] = core::array::from_fn(|_| prng.next() >> 4);
        Self {
            buffer,
            forward_backward: false,
            buffer_size: 8,
            cursor: 0,
            display_mode: DisplayMode::ShowBuffer,
            current_output: None,
            last_rendered_led_display: Cell::new(0),
            prng,
        }
    }

    pub fn adjust_buffer_size(&mut self, change: SizeAdjustment, current_time: u32) {
        // TODO maybe only edit a candidate value and lock it in after a delay so
        // cursor position doesn't get messed up when hovering a small buffer size
        match change {
            SizeAdjustment::PowersOfTwo(delta) => {
                if delta == 0 {
                    return;
                }
                let n = if self.forward_backward {
                    -(self.buffer_size as i8)
                } else {
                    self.buffer_size as i8
                };
                let result = step_in_powers_of_2(n, delta);
                self.buffer_size = result.unsigned_abs().min(MAX_BUFFER_SIZE);
                self.forward_backward = result < 0;
            }
            SizeAdjustment::ExactDelta(delta) => {
                if delta == 0 {
                    return;
                }
                let mut n: i8 = if self.forward_backward {
                    -(self.buffer_size as i8)
                } else {
                    self.buffer_size as i8
                };
                if n >= 1 {
                    n -= 2;
                };
                n += delta;
                if n >= -1 {
                    n += 2;
                }
                self.buffer_size = n.unsigned_abs().min(MAX_BUFFER_SIZE);
                self.forward_backward = n < 0;
            }
        }
        // Should this be mod or max? or just reset to 0?
        self.cursor = self.cursor % self.buffer_size;
        self.display_mode = DisplayMode::ShowBufferLengthSince(current_time)
    }

    fn binary_representation_as_display_buffer(n: i8) -> u8 {
        let mut result: u8 = 0;
        let n_abs = n.unsigned_abs() as u16;
        for i in 0..NUM_LEDS - 1 {
            result.write_bit(i, n_abs & (1 << i as u16) != 0);
        }
        if n < 0 {
            result.set_bit(NUM_LEDS - 1);
        }
        result
    }

    pub fn render_display_if_needed<RENDERER>(&self, bias: u16, render_display: RENDERER)
    where
        RENDERER: FnOnce(&[u16; NUM_LEDS as usize]) -> Result<(), ()>,
    {
        let bit_vector_to_display = match self.display_mode {
            DisplayMode::ShowBuffer => {
                const ACTIVE_VALUE_LED_INDEX: u8 = 3;
                let mut result: u8 = 0;
                for i in 0..NUM_LEDS {
                    let buffer_idx = (i + self.cursor)
                        .subtract_mod(ACTIVE_VALUE_LED_INDEX, self.buffer_size)
                        as usize;
                    result.write_bit(NUM_LEDS - 1 - i, self.buffer[buffer_idx] <= bias);
                }
                result
            }
            DisplayMode::ShowBufferLengthSince(_) => {
                Self::binary_representation_as_display_buffer(if self.forward_backward {
                    -(self.buffer_size as i8)
                } else {
                    self.buffer_size as i8
                })
            }
        };

        if bit_vector_to_display == self.last_rendered_led_display.get() {
            return;
        }

        let to_write: [u16; NUM_LEDS as usize] = core::array::from_fn(|i| {
            if bit_vector_to_display.get_bit(i as u8) {
                SCALE_MAX
            } else {
                0u16
            }
        });
        if let Ok(()) = render_display(&to_write) {
            self.last_rendered_led_display.set(bit_vector_to_display);
        }
    }

    /**
    Main functionality called on every clock input. Rotates the buffer, maybe
    mutates it, then outputs the current analog value and triggers.
    */
    pub fn handle_clock_trigger(
        &mut self,
        current_time_ms: u32,
        input: &RngModuleInput,
    ) -> RngModuleOutput {
        let is_enabled = input.enable_cv && input.chance_cv > 0;

        self.cursor = (self.cursor + 1) % self.buffer_size;
        if is_enabled && (self.prng.next() >> 6) < input.chance_cv {
            self.buffer[self.cursor as usize] = self.prng.next() >> 4
        }

        let is_channel_b = self.buffer[self.cursor as usize] <= input.bias_cv;
        let analog_out = self.buffer[self.cursor as usize];

        self.current_output = Some(CurrentOutputState {
            clock_trigger_time_ms: current_time_ms,
            channel: if is_channel_b { Channel::B } else { Channel::A },
            trigger_mode: if input.trig_mode {
                TriggerMode::Trigger
            } else {
                TriggerMode::Gate
            },
            analog_out,
        });

        RngModuleOutput {
            clock_led_on: true,
            enable_led_on: is_enabled,
            output_a: !is_channel_b,
            output_b: is_channel_b,
            analog_out,
        }
    }

    /**
    Update the outputs as time passes to turn off LEDs and/or triggers
    */
    pub fn time_step(
        &mut self,
        current_time_ms: u32,
        input: &RngModuleInputShort,
    ) -> RngModuleOutput {
        if let DisplayMode::ShowBufferLengthSince(start_time) = self.display_mode {
            if current_time_ms > start_time + BUFFER_LEN_DISPLAY_TIME_MS {
                self.display_mode = DisplayMode::ShowBuffer;
            }
        }

        let enabled = input.enable_cv && input.chance_pot > 0;
        match self.current_output {
            Some(ref mut current_output) => {
                let time_since_last_clock = current_time_ms - current_output.clock_trigger_time_ms;

                if current_output.trigger_mode == TriggerMode::Trigger
                    && time_since_last_clock > TRIG_TIME_MS
                {
                    current_output.channel = Channel::Neither;
                }

                let clock_led_on = time_since_last_clock < TRIG_LED_TIME_MS;

                RngModuleOutput {
                    clock_led_on,
                    enable_led_on: enabled,
                    output_a: current_output.channel == Channel::A,
                    output_b: current_output.channel == Channel::B,
                    analog_out: current_output.analog_out,
                }
            }
            None => RngModuleOutput {
                clock_led_on: false,
                enable_led_on: enabled,
                output_a: false,
                output_b: false,
                analog_out: 0,
            },
        }
    }
}
