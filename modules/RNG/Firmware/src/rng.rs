//! This module abstracts the core functionality of the RNG module in a
//! more-or-less hardare-independent way.

use fm_lib::{number_utils::step_in_powers_of_2, rng::ParallelLfsr};

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

pub struct RngModule<const MAX_BUFFER_SIZE: u8, const NUM_LEDS: u8>
where
    [(); MAX_BUFFER_SIZE as usize]: Sized,
    [(); NUM_LEDS as usize]: Sized,
{
    buffer: [u16; MAX_BUFFER_SIZE as usize],
    forward_backward: bool,
    buffer_size: u8,
    cursor: u8,
    display_mode: DisplayMode,
    display_needs_update: bool,
    last_clock_trigger_ms: u32,
}

impl<const MAX_BUFFER_SIZE: u8, const NUM_LEDS: u8> RngModule<MAX_BUFFER_SIZE, NUM_LEDS>
where
    [(); MAX_BUFFER_SIZE as usize]: Sized,
    [(); NUM_LEDS as usize]: Sized,
{
    pub fn new(prng: &mut ParallelLfsr) -> Self {
        let mut buffer: [u16; MAX_BUFFER_SIZE as usize] =
            unsafe { core::mem::MaybeUninit::uninit().assume_init() };
        for i in 0..MAX_BUFFER_SIZE {
            buffer[i as usize] = prng.next();
        }
        Self {
            buffer,
            forward_backward: false,
            buffer_size: 8,
            cursor: 0,
            display_mode: DisplayMode::ShowBuffer,
            display_needs_update: true,
            last_clock_trigger_ms: u32::MAX,
        }
    }

    pub fn adjust_buffer_size(&mut self, change: SizeAdjustment, current_time: u32) {
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
        self.display_needs_update = true;
        self.display_mode = DisplayMode::ShowBufferLengthSince(current_time)
    }

    fn binary_representation_as_display_buffer(n: i8) -> [u16; NUM_LEDS as usize] {
        let mut result = [0u16; NUM_LEDS as usize];
        let n_abs = n.unsigned_abs() as u16;
        for i in 0..NUM_LEDS - 1 {
            result[i as usize] = if n_abs & (1 << i as u16) == 0 {
                0
            } else {
                0xfff
            };
        }
        if n < 0 {
            result[NUM_LEDS as usize - 1] = 0xfff;
        }
        result
    }

    pub fn render_display_if_needed<RENDERER>(
        &mut self,
        current_time: u32,
        render_display: RENDERER,
    ) where
        RENDERER: FnOnce(&[u16; NUM_LEDS as usize]) -> Result<(), ()>,
    {
        if let DisplayMode::ShowBufferLengthSince(start_time) = self.display_mode {
            if current_time > start_time + BUFFER_LEN_DISPLAY_TIME_MS {
                self.display_mode = DisplayMode::ShowBuffer;
                self.display_needs_update = true;
            }
        }
        if self.display_needs_update {
            let to_write = match self.display_mode {
                DisplayMode::ShowBuffer => [0xfffu16; NUM_LEDS as usize], // DEBUG PLACEHOLDER
                DisplayMode::ShowBufferLengthSince(_) => {
                    Self::binary_representation_as_display_buffer(if self.forward_backward {
                        -(self.buffer_size as i8)
                    } else {
                        self.buffer_size as i8
                    })
                }
            };
            if let Ok(()) = render_display(&to_write) {
                self.display_needs_update = false;
            }
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
        todo!()
    }

    /**
    Update the outputs as time passes to turn off LEDs and/or tirggers
    */
    pub fn time_step(&mut self, current_time_ms: u32) -> RngModuleOutput {
        todo!()
    }
}

struct RngModuleInput {
    chance_pot: u16,
    bias_pot: u16,
    bias_cv: u16,
    trig_mode: bool,
    enable_cv: bool,
}

struct RngModuleOutput {
    clock_led_on: bool,
    enable_led_on: bool,
    output_a: bool,
    output_b: bool,
    analog_out: u16,
}
