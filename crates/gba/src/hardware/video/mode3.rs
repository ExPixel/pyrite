use byteorder::{ByteOrder, LittleEndian};

use crate::memory::VRAM_SIZE;

use super::{
    line::{GbaLine, Layer, Pixel},
    RenderContext, VISIBLE_LINE_WIDTH,
};

pub(super) fn render(line: &mut GbaLine, context: RenderContext) {
    #[cfg(feature = "pyrite-profiling")]
    pyrite_profiling::scope!("mode3::render");

    assert!(context.line < 160);
    let frame_buffer = Mode3FrameBuffer::new(context.vram);
    for x in 0..VISIBLE_LINE_WIDTH {
        line.put(Layer::Bg2, x, frame_buffer.get_pixel(context.line, x));
    }
}

struct Mode3FrameBuffer<'a> {
    buffer: &'a [u8; 0x12C00],
}

impl<'a> Mode3FrameBuffer<'a> {
    const MODE3_FRAMEBUFFER_SIZE: usize = 0x12C00;
    const MODE3_LINE_SIZE: usize = VISIBLE_LINE_WIDTH * 2;

    pub fn new(vram: &'a [u8; VRAM_SIZE]) -> Self {
        Self {
            buffer: (&vram[..Self::MODE3_FRAMEBUFFER_SIZE]).try_into().unwrap(),
        }
    }

    pub fn get_pixel(&self, line: u16, x: usize) -> Pixel {
        let line_offset = (line as usize) * Self::MODE3_LINE_SIZE;
        let offset = line_offset + (x * 2);
        let pixel = LittleEndian::read_u16(&self.buffer[offset..]);
        Pixel::new(pixel)
    }
}
