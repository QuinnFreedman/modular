use core::marker::ConstParamTy;

use avr_progmem::progmem;
use avr_progmem::wrapper::ProgMem;

/**
Rendering text using embedded_graphics with standard fonts is way too slow and memory intensive. I created a custom font
format to make it possible using the following optimizations:

1. Only include ASCII characters 32-127. The first 32 ASCII values aren't renderable (except TAB) to leaving these out
   saves room. All text rendering is done in ASCII (for performance), so no unicode.
2. Only store the raw bitmap of each character, no metadata. In some cases, with large font sizes, it might take less
   memory to store a tightly cropped bitmap for each character along with size and offset metadata (like the BDF format)
   but that would require more work to parse the font file and more computation at run time.
3. Store the character bitmaps in the same column-major format that the MiniBuffer uses and the actual SSD1306 chip uses.
   That way, the glyphs can be directly memcopied to the buffer with little computation.
4. Store the font data in PROGMEM (the program storage space) instead of RAM. Rust is not really designed to work on
   Harvard architecture (like the ATmega) so by default, if you have some const data, it will put that in RAM
   (.data or .bss). The ATmega has very limited RAM, so we need to go out of our way to store the font data in ROM
   (.text or .progmem.data), which has more space.
*/

pub const fn get_glyph_size_bytes(glyph_width: u8, glyph_height: u8) -> usize {
    return usize::div_ceil(glyph_height as usize, u8::BITS as usize) * glyph_width as usize;
}
pub const fn get_font_buffer_size(glyph_width: u8, glyph_height: u8, charset: CharSet) -> usize {
    let num_characters = match charset {
        CharSet::VisibleAscii => 96,
        CharSet::AlphanumericOnly => 75,
        CharSet::NumeralsOnly => 10,
    };
    return get_glyph_size_bytes(glyph_width, glyph_height) * num_characters;
}

progmem! {
    static progmem PRO_FONT_22_RAW_BYTES:  [u8; get_font_buffer_size(12, 22, CharSet::AlphanumericOnly)] = *include_bytes!("../assets/profont_22_alphanum.bin");
    static progmem PRO_FONT_29_RAW_BYTES:  [u8; get_font_buffer_size(16, 29, CharSet::NumeralsOnly)] = *include_bytes!("../assets/profont_29_numeric.bin");
}

#[allow(dead_code)]
#[derive(PartialEq, Eq, ConstParamTy)]
pub enum CharSet {
    VisibleAscii,
    AlphanumericOnly,
    NumeralsOnly,
}

pub struct ProgmemBitmapFont<const GLYPH_WIDTH: u8, const GLYPH_HEIGHT: u8, const CHARSET: CharSet>
where
    [(); get_font_buffer_size(GLYPH_WIDTH, GLYPH_HEIGHT, CHARSET)]: Sized,
{
    raw_bytes: ProgMem<[u8; get_font_buffer_size(GLYPH_WIDTH, GLYPH_HEIGHT, CHARSET)]>,
}

trait HasGlyphSizeBytes<const GLYPH_WIDTH: u8, const GLYPH_HEIGHT: u8> {}

impl<const GLYPH_WIDTH: u8, const GLYPH_HEIGHT: u8, const CHARSET: CharSet>
    ProgmemBitmapFont<GLYPH_WIDTH, GLYPH_HEIGHT, CHARSET>
where
    [(); get_font_buffer_size(GLYPH_WIDTH, GLYPH_HEIGHT, CHARSET)]: Sized,
    [(); get_glyph_size_bytes(GLYPH_WIDTH, GLYPH_HEIGHT)]: Sized,
{
    pub fn get_glyph(&self, c: u8) -> [u8; get_glyph_size_bytes(GLYPH_WIDTH, GLYPH_HEIGHT)] {
        let (char_idx_start, char_idx_end) = match CHARSET {
            CharSet::VisibleAscii => (32u8, 127u8),
            CharSet::AlphanumericOnly => (48u8, 122u8),
            CharSet::NumeralsOnly => (48u8, 57u8),
        };
        debug_assert!(c >= char_idx_start && c <= char_idx_end);
        self.raw_bytes
            .load_sub_array::<{ get_glyph_size_bytes(GLYPH_WIDTH, GLYPH_HEIGHT) }>(
                (c - char_idx_start) as usize * get_glyph_size_bytes(GLYPH_WIDTH, GLYPH_HEIGHT),
            )
    }

    pub const fn new(
        array: ProgMem<[u8; get_font_buffer_size(GLYPH_WIDTH, GLYPH_HEIGHT, CHARSET)]>,
    ) -> Self {
        Self { raw_bytes: array }
    }
}

pub static PRO_FONT_22: ProgmemBitmapFont<12, 22, { CharSet::AlphanumericOnly }> =
    ProgmemBitmapFont::<12, 22, { CharSet::AlphanumericOnly }>::new(PRO_FONT_22_RAW_BYTES);

pub static PRO_FONT_29_NUMERIC: ProgmemBitmapFont<16, 29, { CharSet::NumeralsOnly }> =
    ProgmemBitmapFont::<16, 29, { CharSet::NumeralsOnly }>::new(PRO_FONT_29_RAW_BYTES);
