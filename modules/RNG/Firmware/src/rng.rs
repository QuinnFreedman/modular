trait LowerPowerOfTwo {
    /**
    Returns the largest power of two less than the given number, or 0
    */
    fn lower_power_of_two(self) -> Self;
}

impl LowerPowerOfTwo for u8 {
    fn lower_power_of_two(self) -> Self {
        if self <= 1 {
            return 0;
        }
        let n = self - 1;

        let first_set_bit_index = Self::BITS - n.leading_zeros() - 1;

        1u8 << first_set_bit_index
    }
}

pub enum SizeAdjustment {
    PowersOfTwo(i8),
    ExactDelta(i8),
}

pub enum DisplayMode {
    ShowBuffer,
    ShowBufferLengthSince(u32),
}

const BUFFER_LEN_DISPLAY_TIME_MS: u32 = 3000;

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
}

impl<const MAX_BUFFER_SIZE: u8, const NUM_LEDS: u8> RngModule<MAX_BUFFER_SIZE, NUM_LEDS>
where
    [(); MAX_BUFFER_SIZE as usize]: Sized,
    [(); NUM_LEDS as usize]: Sized,
{
    pub fn new() -> Self {
        Self {
            buffer: [0u16; MAX_BUFFER_SIZE as usize],
            forward_backward: false,
            buffer_size: 8,
            cursor: 0,
            display_mode: DisplayMode::ShowBuffer,
            display_needs_update: true,
        }
    }

    pub fn adjust_buffer_size(&mut self, change: SizeAdjustment, current_time: u32) {
        match change {
            SizeAdjustment::PowersOfTwo(original_delta) => {
                if original_delta == 0 {
                    return;
                }
                let delta_positive = original_delta > 0;
                let mut delta = original_delta.unsigned_abs();
                while delta > 0 {
                    let buffer_size_positive = !self.forward_backward;
                    self.buffer_size = if delta_positive == buffer_size_positive {
                        (self.buffer_size + 1)
                            .next_power_of_two()
                            .min(MAX_BUFFER_SIZE)
                    } else {
                        self.buffer_size.lower_power_of_two()
                    };
                    if self.buffer_size == 0 || (self.buffer_size == 1 && self.forward_backward) {
                        if delta_positive {
                            self.buffer_size = 1;
                            self.forward_backward = false;
                        } else {
                            self.buffer_size = 2;
                            self.forward_backward = true;
                        }
                    }
                    delta -= 1;
                }
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
}
