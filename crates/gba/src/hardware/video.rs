pub const VISIBLE_LINE_WIDTH: usize = 240;
pub const VISIBLE_LINE_COUNT: usize = 160;
pub const VISIBLE_PIXELS: usize = VISIBLE_LINE_WIDTH * VISIBLE_LINE_COUNT;

pub type LineBuffer = [u16; VISIBLE_LINE_WIDTH];
pub type ScreenBuffer = [u16; VISIBLE_PIXELS];

pub struct GbaVideo {
    pub(crate) current_line: usize,
    pub(crate) line_buffer: [u16; VISIBLE_LINE_WIDTH],
}

impl GbaVideo {
    pub(crate) fn new() -> GbaVideo {
        GbaVideo {
            current_line: 240,
            line_buffer: [0; 240],
        }
    }
}

impl Default for GbaVideo {
    fn default() -> Self {
        Self::new()
    }
}

#[inline]
pub const fn rgb16(r: u16, g: u16, b: u16) -> u16 {
    (r & 0x1F) | ((g & 0x1F) << 5) | ((b & 0x1F) << 10)
}
