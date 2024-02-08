//! This module abstracts the core functionality of the RNG module in a
//! more-or-less hardare-independent way.

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

#[derive(PartialEq, Eq)]
enum RenderedDisplay {
    BufferSize { size: u8, forward_backgward: bool },
    BufferValues { cursor: u8, bias: u16 },
    None,
}

/**
Module input.
Analog values are 12-bit (0-4095)
*/
pub struct RngModuleInput {
    pub chance_pot: u16,
    pub bias_pot: u16,
    pub bias_cv: u16,
    pub trig_mode: bool,
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

pub struct RngModule<const MAX_BUFFER_SIZE: u8, const NUM_LEDS: u8>
where
    [(); MAX_BUFFER_SIZE as usize]: Sized,
    [(); NUM_LEDS as usize]: Sized,
{
    pub buffer: [u16; MAX_BUFFER_SIZE as usize],
    forward_backward: bool,
    buffer_size: u8,
    cursor: u8,
    display_mode: DisplayMode,
    last_rendered_led_display: u8,
    last_clock_trigger_ms: u32,
}

impl<const MAX_BUFFER_SIZE: u8, const NUM_LEDS: u8> RngModule<MAX_BUFFER_SIZE, NUM_LEDS>
where
    [(); MAX_BUFFER_SIZE as usize]: Sized,
    [(); NUM_LEDS as usize]: Sized,
{
    pub fn new(prng: &mut ParallelLfsr) -> Self {
        // let mut buffer = [0u16; MAX_BUFFER_SIZE as usize];
        // buffer[0] = SCALE_MAX;
        // buffer[1] = SCALE_MAX;
        // buffer[3] = SCALE_MAX;
        let buffer: [u16; MAX_BUFFER_SIZE as usize] =
            core::array::from_fn(|_| prng.next() % SCALE_MAX);
        Self {
            buffer,
            forward_backward: false,
            buffer_size: 8,
            cursor: 0,
            display_mode: DisplayMode::ShowBuffer,
            last_rendered_led_display: 0,
            last_clock_trigger_ms: u32::MAX,
        }
    }

    pub fn adjust_buffer_size(&mut self, change: SizeAdjustment, current_time: u32) {
        // TODO maybe only edit a candidate value and lock it in after a delay so
        // cusor position doesn't get messed up when hovering a small buffer size
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

    pub fn render_display_if_needed<RENDERER>(
        &mut self,
        current_time: u32,
        bias: u16,
        render_display: RENDERER,
    ) where
        RENDERER: FnOnce(&[u16; NUM_LEDS as usize]) -> Result<(), ()>,
    {
        if let DisplayMode::ShowBufferLengthSince(start_time) = self.display_mode {
            if current_time > start_time + BUFFER_LEN_DISPLAY_TIME_MS {
                self.display_mode = DisplayMode::ShowBuffer;
            }
        }

        let bit_vector_to_display = match self.display_mode {
            DisplayMode::ShowBuffer => {
                const ACTIVE_VALUE_LED_INDEX: u8 = 3;
                let mut result: u8 = 0;
                for i in 0..NUM_LEDS {
                    let buffer_idx = (i + self.cursor)
                        .subtract_mod(ACTIVE_VALUE_LED_INDEX, self.buffer_size)
                        as usize;
                    result.write_bit(NUM_LEDS - 1 - i, self.buffer[buffer_idx] > bias);
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

        if bit_vector_to_display == self.last_rendered_led_display {
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
            self.last_rendered_led_display = bit_vector_to_display;
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
        self.last_clock_trigger_ms = current_time_ms;
        // TODO
        self.cursor = (self.cursor + 1) % self.buffer_size;
        RngModuleOutput {
            clock_led_on: false,
            enable_led_on: false,
            output_a: false,
            output_b: false,
            analog_out: self.buffer[self.cursor as usize],
        }
    }

    /**
    Update the outputs as time passes to turn off LEDs and/or tirggers
    */
    pub fn time_step(&mut self, current_time_ms: u32) -> RngModuleOutput {
        todo!()
    }
}
