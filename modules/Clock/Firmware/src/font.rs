use avr_progmem::progmem;
use avr_progmem::wrapper::ProgMem;

/**
Rendering text using ebeded_graphics with standard fonts is way too slow and memory intensive. I created a custom font
format to make it possible using the following optimiations:

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
pub const fn get_font_buffer_size(glyph_width: u8, glyph_height: u8) -> usize {
    return get_glyph_size_bytes(glyph_width, glyph_height) * 96;
}

progmem! {
    static progmem PRO_FONT_22_RAW_BYTES:  [u8; get_font_buffer_size(12, 22)] = *include_bytes!("../assets/profont_22.bin");
}

pub struct ProgmemBitmapFont<const GLPYPH_WIDTH: u8, const GLYPH_HEIGHT: u8>
where
    [(); get_font_buffer_size(GLPYPH_WIDTH, GLYPH_HEIGHT)]: Sized,
{
    raw_bytes: ProgMem<[u8; get_font_buffer_size(GLPYPH_WIDTH, GLYPH_HEIGHT)]>,
}

trait HasGlyphSizeBytes<const GLYPH_WIDTH: u8, const GLYPH_HEIGHT: u8> {}

impl<const GLYPH_WIDTH: u8, const GLYPH_HEIGHT: u8> ProgmemBitmapFont<GLYPH_WIDTH, GLYPH_HEIGHT>
where
    [(); get_font_buffer_size(GLYPH_WIDTH, GLYPH_HEIGHT)]: Sized,
    [(); get_glyph_size_bytes(GLYPH_WIDTH, GLYPH_HEIGHT)]: Sized,
{
    pub fn get_glyph(&self, c: u8) -> [u8; get_glyph_size_bytes(GLYPH_WIDTH, GLYPH_HEIGHT)] {
        debug_assert!(c >= 32 && c < 128);
        self.raw_bytes
            .load_sub_array::<{ get_glyph_size_bytes(GLYPH_WIDTH, GLYPH_HEIGHT) }>(
                (c - 32) as usize * get_glyph_size_bytes(GLYPH_WIDTH, GLYPH_HEIGHT),
            )
    }

    pub const fn new(
        array: ProgMem<[u8; get_font_buffer_size(GLYPH_WIDTH, GLYPH_HEIGHT)]>,
    ) -> Self {
        Self { raw_bytes: array }
    }
}

pub static PRO_FONT_22: ProgmemBitmapFont<12, 22> =
    ProgmemBitmapFont::<12, 22>::new(PRO_FONT_22_RAW_BYTES);
