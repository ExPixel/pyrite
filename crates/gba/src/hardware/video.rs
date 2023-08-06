pub mod line;
pub mod registers;

use arm::emu::Cycles;
use pyrite_derive::IoRegister;

use crate::{
    events::{GbaEvent, SharedGbaScheduler},
    GbaVideoOutput,
};

use self::registers::GbaVideoRegisters;

pub const VISIBLE_LINE_WIDTH: usize = 240;
pub const VISIBLE_LINE_COUNT: usize = 160;
pub const LINE_COUNT: usize = 228;
pub const VISIBLE_PIXELS: usize = VISIBLE_LINE_WIDTH * VISIBLE_LINE_COUNT;
pub const HDRAW_CYCLES: Cycles = Cycles::new(960);
pub const HBLANK_CYCLES: Cycles = Cycles::new(272);

pub type LineBuffer = [u16; VISIBLE_LINE_WIDTH];
pub type ScreenBuffer = [u16; VISIBLE_PIXELS];

pub struct GbaVideo {
    pub(crate) line_buffer: [u16; VISIBLE_LINE_WIDTH],
    scheduler: SharedGbaScheduler,
    registers: GbaVideoRegisters,
}

impl GbaVideo {
    pub(crate) fn new(scheduler: SharedGbaScheduler) -> GbaVideo {
        GbaVideo {
            line_buffer: [0; VISIBLE_LINE_WIDTH],
            scheduler,
            registers: GbaVideoRegisters::default(),
        }
    }

    fn render_line(&mut self, line: u16, video: &mut dyn GbaVideoOutput) {
        let d = std::time::SystemTime::UNIX_EPOCH.elapsed().unwrap();
        self.line_buffer.fill(d.as_nanos() as u16);
        video.gba_line_ready(line as usize, &self.line_buffer);
    }

    pub(crate) fn reset(&mut self) {
        self.registers
            .reg_vcount
            .set_current_scanline(LINE_COUNT as u16 - 1);
        self.begin_hdraw();
    }

    pub(crate) fn begin_hdraw(&mut self) {
        self.scheduler.schedule(GbaEvent::HBlank, HDRAW_CYCLES);

        let mut current_scanline = self.registers.reg_vcount.current_scanline();
        if current_scanline >= (LINE_COUNT - 1) as u16 {
            current_scanline = 0;
        } else {
            current_scanline += 1;
        }
        self.registers
            .reg_vcount
            .set_current_scanline(current_scanline);
    }

    pub(crate) fn begin_hblank(&mut self, video: &mut dyn GbaVideoOutput) {
        self.scheduler.schedule(GbaEvent::HDraw, HBLANK_CYCLES);

        let current_scanline = self.registers.reg_vcount.current_scanline();
        if current_scanline < VISIBLE_LINE_COUNT as _ {
            self.render_line(current_scanline, video);
        }
    }
}

#[inline]
pub const fn rgb16(r: u16, g: u16, b: u16) -> u16 {
    (r & 0x1F) | ((g & 0x1F) << 5) | ((b & 0x1F) << 10)
}
