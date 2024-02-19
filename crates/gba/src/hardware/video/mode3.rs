use byteorder::{ByteOrder, LittleEndian};

use crate::{memory::VRAM_SIZE, video::line::Pixel};

use super::{line::GbaLine, RenderContext, VISIBLE_LINE_WIDTH};

pub(super) fn render(line: &mut GbaLine, context: RenderContext) {
    #[cfg(feature = "puffin")]
    puffin::profile_function!();

    // FIXME figure out of this actually does anything. For now I just
    //      have it here so I remember to test it later. Maybe it gets rid
    //      of some bounds checks ???
    assert!(context.line < 160);

    let frame_buffer = Mode3FrameBuffer::new(context.vram);
    for x in 0..VISIBLE_LINE_WIDTH {
        let pixel = frame_buffer.get_pixel(context.line, x);
        line.push(x, Pixel::new_bitmap(pixel));
    }
}

struct Mode3FrameBuffer<'a> {
    buffer: &'a [u8; Mode3FrameBuffer::MODE3_FRAMEBUFFER_SIZE],
}

impl<'a> Mode3FrameBuffer<'a> {
    const MODE3_FRAMEBUFFER_SIZE: usize = 0x12C00;
    const MODE3_LINE_SIZE: usize = VISIBLE_LINE_WIDTH * 2;

    pub fn new(vram: &'a [u8; VRAM_SIZE]) -> Self {
        Self {
            buffer: (&vram[..Self::MODE3_FRAMEBUFFER_SIZE]).try_into().unwrap(),
        }
    }

    pub fn get_pixel(&self, line: u16, x: usize) -> u16 {
        let line_offset = (line as usize) * Self::MODE3_LINE_SIZE;
        let offset = line_offset + (x * 2);
        LittleEndian::read_u16(&self.buffer[offset..])
    }
}
