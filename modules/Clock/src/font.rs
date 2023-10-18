use avr_progmem::progmem;
use avr_progmem::wrapper::ProgMem;

pub const fn get_glyph_size_bytes(glyph_width: u8, glyph_height: u8) -> usize {
    return usize::div_ceil(glyph_height as usize, u8::BITS as usize) * glyph_width as usize;
}
pub const fn get_font_buffer_size(glyph_width: u8, glyph_height: u8) -> usize {
    return get_glyph_size_bytes(glyph_width, glyph_height) * 96;
}

progmem! {
    static progmem PRO_FONT_22_RAW_BYTES:  [u8; get_font_buffer_size(12, 22)] = *include_bytes!("../jupyter/font.raw");
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
