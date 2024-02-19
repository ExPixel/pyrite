use crate::{
    memory::VRAM_SIZE,
    video::line::{Pixel, PixelAttrs},
};

use super::{line::GbaLine, registers::DisplayFrame, RenderContext, VISIBLE_LINE_WIDTH};

pub(super) fn render(line: &mut GbaLine, context: RenderContext) {
    #[cfg(feature = "puffin")]
    puffin::profile_function!();

    // FIXME figure out of this actually does anything. For now I just
    //      have it here so I remember to test it later. Maybe it gets rid
    //      of some bounds checks ???
    assert!(context.line < 160);

    let frame = context.registers.dispcnt.display_frame_select();
    let frame_buffer = Mode4FrameBuffer::new(context.vram, frame);
    let attrs = PixelAttrs::default();

    for x in 0..VISIBLE_LINE_WIDTH {
        let pixel = frame_buffer.get_pixel(context.line, x);
        if pixel != 0 {
            line.push(x, Pixel::new(attrs, pixel));
        }
    }
}

struct Mode4FrameBuffer<'a> {
    buffer: &'a [u8; Mode4FrameBuffer::MODE4_FRAMEBUFFER_SIZE],
}

impl<'a> Mode4FrameBuffer<'a> {
    const MODE4_FRAMEBUFFER_SIZE: usize = 0x9600;
    const MODE4_LINE_SIZE: usize = VISIBLE_LINE_WIDTH;

    pub fn new(vram: &'a [u8; VRAM_SIZE], frame: DisplayFrame) -> Self {
        let buffer = match frame {
            DisplayFrame::Frame0 => &vram[..Self::MODE4_FRAMEBUFFER_SIZE],
            DisplayFrame::Frame1 => &vram[Self::MODE4_FRAMEBUFFER_SIZE..],
        };

        Self {
            buffer: buffer.try_into().unwrap(),
        }
    }

    pub fn get_pixel(&self, line: u16, x: usize) -> u8 {
        let line_offset = (line as usize) * Self::MODE4_LINE_SIZE;
        let offset = line_offset + x;
        self.buffer[offset]
    }
}
