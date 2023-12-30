use display_interface::{DisplayError, WriteOnlyDataCommand};
use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::{Dimensions, DrawTarget, Point, Size},
    primitives::Rectangle,
    Pixel,
};
use ssd1306::{mode::BasicMode, size::DisplaySize, Ssd1306};

use crate::font::{get_font_buffer_size, get_glyph_size_bytes, CharSet, ProgmemBitmapFont};

const BYTE_SIZE: usize = u8::BITS as usize;

/**
Storing the full screen buffer in memory takes a lot of memory and is also relatively
slow, since the full buffer hast to be transmitted to the display driver over SPI
every time it updates. But, drawing directly to the screen can be very difficult if
the shapes you're drawing don't exactly line up with the underlying pages of the
display driver, and could also lead to flickering from non-sequential updates.

As a compromise, the mini buffer is a variable-size buffer that can back just a small
portion of the screen. It can be drawn to like a display using embedded_graphics, and
then can be efficiently copied to the display driver using the blit function.
 */
#[repr(transparent)]
pub struct MiniBuffer<const WIDTH: usize, const HEIGHT: usize>([u8; WIDTH * HEIGHT / BYTE_SIZE])
where
    [(); WIDTH * HEIGHT / BYTE_SIZE]: Sized;

impl<const WIDTH: usize, const HEIGHT: usize> DrawTarget for MiniBuffer<WIDTH, HEIGHT>
where
    [(); WIDTH * HEIGHT / BYTE_SIZE]: Sized,
{
    type Color = BinaryColor;

    type Error = ();

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = embedded_graphics::Pixel<Self::Color>>,
    {
        for Pixel(point, color) in pixels {
            let x = point.x.clamp(0, 255) as usize;
            let y = point.y.clamp(0, 255) as usize;

            let bit_offset = x * HEIGHT + y;
            let bytes = bit_offset / BYTE_SIZE;
            let bits = bit_offset % BYTE_SIZE;
            if bytes >= self.0.len() {
                return Ok(());
            };
            let byte = self.0[bytes];
            let bit_mask = 1 << bits;
            if color.is_on() {
                self.0[bytes] = byte | bit_mask;
            } else {
                self.0[bytes] = byte & !bit_mask;
            }
        }
        Ok(())
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        let fill = match color {
            BinaryColor::Off => 0u8,
            BinaryColor::On => 0xffu8,
        };
        self.0.fill(fill);
        Ok(())
    }

    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        let width = area.size.width.min(usize::MAX as u32) as usize;
        let height = area.size.height.min(usize::MAX as u32) as usize;
        let x = area.top_left.x.clamp(0, usize::MAX as i32) as usize;
        let y = area.top_left.y.clamp(0, usize::MAX as i32) as usize;

        self.fast_fill(x, y, width, height, color);
        Ok(())
    }
}

impl<const WIDTH: usize, const HEIGHT: usize> MiniBuffer<WIDTH, HEIGHT>
where
    [(); WIDTH * HEIGHT / BYTE_SIZE]: Sized,
{
    /**
    Efficiently copy the contents of the buffer to the SSD1306 driver in BasicMode
    */
    pub fn blit<DI, SIZE>(
        &self,
        display: &mut Ssd1306<DI, SIZE, BasicMode>,
        x: u8,
        y: u8,
    ) -> Result<(), DisplayError>
    where
        DI: WriteOnlyDataCommand,
        SIZE: DisplaySize,
    {
        if y % 8 != 0 {
            return Err(DisplayError::OutOfBoundsError);
        }
        let (display_width, display_height) = display.dimensions();
        if x + WIDTH as u8 > display_width as u8 || y + HEIGHT as u8 > display_height as u8 {
            return Err(DisplayError::OutOfBoundsError);
        }
        display.set_draw_area((x, y), (x + WIDTH as u8, y + HEIGHT as u8))?;
        display.draw(&self.0)?;
        Ok(())
    }

    fn get_idx(&self, col: usize, page: usize) -> usize {
        return col * HEIGHT / BYTE_SIZE + page;
    }
    fn write_byte_if_in_bounds(&mut self, col: usize, page: usize, value: u8, color: &TextColor) {
        if page >= HEIGHT / BYTE_SIZE {
            return;
        }
        if col >= WIDTH {
            return;
        }
        let idx = self.get_idx(col, page);
        return self.0[idx] = match color {
            TextColor::BinaryOn => value,
            TextColor::BinaryOff => !value,
            TextColor::BinaryOnTransparent => self.0[idx] | value,
            TextColor::BinaryOffTransparent => self.0[idx] & !value,
            TextColor::InvertTransparent => (self.0[idx] & !value) | (!self.0[idx] & value),
        };
    }

    pub const fn new() -> Self {
        if HEIGHT % BYTE_SIZE != 0 {
            panic!()
        }
        let buffer = [0u8; WIDTH * HEIGHT / BYTE_SIZE];
        return MiniBuffer(buffer);
    }

    #[inline(never)]
    pub fn fast_draw_image(
        &mut self,
        x: usize,
        y: usize,
        img_width: u8,
        img_height: u8,
        raw_data: &[u8],
        color: &TextColor,
    ) {
        self.fast_draw_image_inline(x, y, img_width, img_height, raw_data, color)
    }

    #[inline(always)]
    fn fast_draw_image_inline(
        &mut self,
        x: usize,
        y: usize,
        img_width: u8,
        img_height: u8,
        raw_data: &[u8],
        color: &TextColor,
    ) {
        let y_offset_bytes = y / BYTE_SIZE;
        let y_offset_bits = (y % BYTE_SIZE) as u8;
        let img_height_bytes: u8 = img_height.div_ceil(u8::BITS as u8);

        for col_in_glyph in 0..img_width {
            let col_in_buff = col_in_glyph as usize + x;
            let mut last_byte_in_glyph = 0u8;
            for byte_idx_in_glyph in 0..img_height_bytes {
                let byte_in_glyph = raw_data[col_in_glyph as usize * img_height_bytes as usize
                    + byte_idx_in_glyph as usize];
                let byte_to_write = if y_offset_bits == 0 {
                    byte_in_glyph
                } else {
                    (byte_in_glyph << y_offset_bits)
                        | (last_byte_in_glyph >> (u8::BITS as u8 - y_offset_bits))
                };

                last_byte_in_glyph = byte_in_glyph;
                self.write_byte_if_in_bounds(
                    col_in_buff,
                    byte_idx_in_glyph as usize + y_offset_bytes,
                    byte_to_write,
                    color,
                )
            }
            if img_height + y_offset_bits > img_height_bytes * (u8::BITS as u8) {
                debug_assert_ne!(y_offset_bits, 0);
                self.write_byte_if_in_bounds(
                    col_in_buff,
                    img_height_bytes as usize + y_offset_bytes,
                    last_byte_in_glyph >> (u8::BITS as u8 - y_offset_bits),
                    color,
                )
            }
        }
    }

    /**
    Drawing text with embedded_graphics requires loading the full font into memory,
    which there isn't room for in the atmega. Also, it is relatively slow. This
    function takes advantage of specifically formatted font data stored in PROGMEM
    and the column-major layout of the buffer to draw text very efficiently.
    */
    pub fn fast_draw_ascii_text<
        const GLYPH_WIDTH: u8,
        const GLYPH_HEIGHT: u8,
        const CHARSET: CharSet,
    >(
        &mut self,
        horizontal: Justify,
        vertical: Justify,
        text: &[u8],
        font: &'static ProgmemBitmapFont<GLYPH_WIDTH, GLYPH_HEIGHT, CHARSET>,
        color: &TextColor,
    ) where
        [(); get_font_buffer_size(GLYPH_WIDTH, GLYPH_HEIGHT, CHARSET)]: Sized,
        [(); get_glyph_size_bytes(GLYPH_WIDTH, GLYPH_HEIGHT)]: Sized,
    {
        let text_width = GLYPH_WIDTH as usize * text.len();
        // NOTE: if we want to allow negative offset, we could make x and y i16 and
        // add a boudns check to the loop then cast back, but it's faster to not as
        // long as it's not used
        let x = match horizontal {
            Justify::Start(offset) => offset,
            Justify::Center(offset) => offset.saturating_sub(text_width / 2),
            Justify::End(offset) => offset.saturating_sub(text_width),
        };
        let y = match vertical {
            Justify::Start(offset) => offset,
            Justify::Center(offset) => offset.saturating_sub(GLYPH_HEIGHT as usize / 2),
            Justify::End(offset) => offset.saturating_sub(GLYPH_HEIGHT as usize),
        };

        let mut cursor = x;
        for ascii_char in text {
            let glyph = font.get_glyph(*ascii_char);

            self.fast_draw_image_inline(cursor, y, GLYPH_WIDTH, GLYPH_HEIGHT, &glyph, color);

            cursor += GLYPH_WIDTH as usize;
        }
    }

    pub fn fast_rect(
        &mut self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        color: BinaryColor,
        thickness: usize,
    ) {
        self.fast_fill(x, y, width, thickness, color);
        self.fast_fill(x, y + height - thickness, width, thickness, color);
        self.fast_fill(x, y, thickness, height, color);
        self.fast_fill(x + width - thickness, y, thickness, height, color);
    }

    pub fn fast_fill(
        &mut self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        color: BinaryColor,
    ) {
        if height == 0 || width == 0 {
            return;
        }

        let fill = match color {
            BinaryColor::Off => 0u8,
            BinaryColor::On => 0xffu8,
        };
        let left_bound_inclusive = x.clamp(0, WIDTH) as usize;
        let right_bound_exclusive = (left_bound_inclusive + width).min(WIDTH);
        let upper_bound_inclusive = y.clamp(0, HEIGHT) as usize;
        let lower_bound_exclusive = (upper_bound_inclusive + height).min(HEIGHT);

        // if start and end are on the same page, handle that with a special case
        if upper_bound_inclusive / BYTE_SIZE == (lower_bound_exclusive - 1) / BYTE_SIZE {
            let byte_idx = upper_bound_inclusive / BYTE_SIZE;
            let bit_offset = upper_bound_inclusive % BYTE_SIZE;
            let bit_offset_from_end = ((byte_idx + 1) * BYTE_SIZE) - lower_bound_exclusive;
            let mask = (0xffu8 << bit_offset) & (0xffu8 >> bit_offset_from_end);
            for col in left_bound_inclusive..right_bound_exclusive {
                let idx = self.get_idx(col, byte_idx);
                self.0[idx] = match color {
                    BinaryColor::Off => self.0[idx] & !mask,
                    BinaryColor::On => self.0[idx] | mask,
                };
            }
            return;
        }

        for col in left_bound_inclusive..right_bound_exclusive {
            let mut vcursor_bits = upper_bound_inclusive;
            // (maybe) fill first partial byte
            let bit_shift_in_first_byte = vcursor_bits % BYTE_SIZE;
            if bit_shift_in_first_byte != 0 {
                let idx = self.get_idx(col, vcursor_bits / BYTE_SIZE);
                self.0[idx] = match color {
                    BinaryColor::Off => self.0[idx] & !(0xffu8 << bit_shift_in_first_byte),
                    BinaryColor::On => self.0[idx] | 0xffu8 << bit_shift_in_first_byte,
                };
                vcursor_bits += BYTE_SIZE - bit_shift_in_first_byte;
            }
            // fill all full bytes in column
            debug_assert!(vcursor_bits % BYTE_SIZE == 0);
            while lower_bound_exclusive - vcursor_bits >= 8 {
                let idx = self.get_idx(col, vcursor_bits / BYTE_SIZE);
                self.0[idx] = fill;
                vcursor_bits += 8;
            }
            // (maybe) fill last partial byte
            let remaining_bits_to_fill = lower_bound_exclusive - vcursor_bits;
            if remaining_bits_to_fill > 0 {
                let last_byte_mask = 0xffu8 >> (BYTE_SIZE - remaining_bits_to_fill);
                let idx = self.get_idx(col, vcursor_bits / BYTE_SIZE);
                self.0[idx] = match color {
                    BinaryColor::Off => self.0[idx] & !last_byte_mask,
                    BinaryColor::On => self.0[idx] | last_byte_mask,
                };
            }
        }
    }
}

#[allow(dead_code)]
pub enum Justify {
    Start(usize),
    Center(usize),
    End(usize),
}

#[allow(dead_code)]
pub enum TextColor {
    BinaryOn,
    BinaryOff,
    BinaryOnTransparent,
    BinaryOffTransparent,
    InvertTransparent,
}

impl<const WIDTH: usize, const HEIGHT: usize> Dimensions for MiniBuffer<WIDTH, HEIGHT>
where
    [(); WIDTH * HEIGHT / BYTE_SIZE]: Sized,
{
    fn bounding_box(&self) -> Rectangle {
        return Rectangle::new(Point::new(0, 0), Size::new(WIDTH as u32, HEIGHT as u32));
    }
}
