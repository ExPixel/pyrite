pub mod line;
mod mode3;
mod mode4;
pub mod registers;

use arm::emu::Cycles;

use crate::{
    events::{GbaEvent, SharedGbaScheduler},
    memory::VRAM_SIZE,
    GbaVideoOutput,
};

use self::{
    line::{BlendContext, GbaLine},
    registers::{BgMode, GbaVideoRegisters},
};

use super::palette::Palette;

pub const VISIBLE_LINE_WIDTH: usize = 240;
pub const VISIBLE_LINE_COUNT: usize = 160;
pub const LINE_COUNT: usize = 228;
pub const VISIBLE_PIXELS: usize = VISIBLE_LINE_WIDTH * VISIBLE_LINE_COUNT;
pub const HDRAW_CYCLES: Cycles = Cycles::new(960);
pub const HBLANK_CYCLES: Cycles = Cycles::new(272);

pub type LineBuffer = [u16; VISIBLE_LINE_WIDTH];
pub type ScreenBuffer = [u16; VISIBLE_PIXELS];

pub struct GbaVideo {
    pub(crate) line: GbaLine,
    scheduler: SharedGbaScheduler,
    pub(crate) registers: GbaVideoRegisters,
    pub(crate) frame: u64,
}

impl GbaVideo {
    pub(crate) fn new(scheduler: SharedGbaScheduler) -> GbaVideo {
        GbaVideo {
            line: GbaLine::default(),
            scheduler,
            registers: GbaVideoRegisters::default(),
            frame: 0,
        }
    }

    fn render_line(&mut self, line: u16, video: &mut dyn GbaVideoOutput, context: HBlankContext) {
        let mut unhandled_mode = false;

        let render_context = RenderContext::new(line, &self.registers, context.vram);
        self.line.clear(context.palette);
        match self.registers.dispcnt.bg_mode() {
            BgMode::Mode0 => unhandled_mode = true,
            BgMode::Mode1 => unhandled_mode = true,
            BgMode::Mode3 => mode3::render(&mut self.line, render_context),
            BgMode::Mode2 => unhandled_mode = true,
            BgMode::Mode4 => mode4::render(&mut self.line, render_context),
            BgMode::Mode5 => unhandled_mode = true,
            BgMode::Invalid6 => unhandled_mode = true,
            BgMode::Invalid7 => unhandled_mode = true,
        }

        let mut buffer = [0u16; VISIBLE_LINE_WIDTH];
        if unhandled_mode {
            buffer.fill(rgb5(0x1F, 0, 0x1F));
        } else {
            let context = BlendContext::with_hblank(&self.registers, context);
            self.line.blend(&mut buffer, context);
        }
        video.gba_line_ready(line as usize, &buffer);

        if line == (VISIBLE_LINE_COUNT - 1) as u16 {
            self.frame += 1;
        }
    }

    pub(crate) fn reset(&mut self) {
        self.registers
            .vcount
            .set_current_scanline(LINE_COUNT as u16 - 1);
        self.begin_hdraw();
    }

    pub(crate) fn begin_hdraw(&mut self) {
        self.scheduler.schedule(GbaEvent::HBlank, HDRAW_CYCLES);

        let mut current_scanline = self.registers.vcount.current_scanline();
        if current_scanline >= (LINE_COUNT - 1) as u16 {
            current_scanline = 0;
        } else {
            current_scanline += 1;
        }

        self.registers
            .dispstat
            .set_vblank_flag(current_scanline >= VISIBLE_LINE_COUNT as u16);
        self.registers.dispstat.set_hblank_flag(false);
        self.registers.vcount.set_current_scanline(current_scanline);
    }

    pub(crate) fn begin_hblank(&mut self, video: &mut dyn GbaVideoOutput, context: HBlankContext) {
        self.scheduler.schedule(GbaEvent::HDraw, HBLANK_CYCLES);

        self.registers.dispstat.set_hblank_flag(true);
        let current_scanline = self.registers.vcount.current_scanline();
        if current_scanline < VISIBLE_LINE_COUNT as _ {
            self.render_line(current_scanline, video, context);
        }
    }

    #[inline]
    pub fn current_scanline(&self) -> u16 {
        self.registers.vcount.current_scanline()
    }
}

#[derive(Copy, Clone)]
pub struct HBlankContext<'a> {
    pub palette: &'a Palette,
    pub vram: &'a [u8; VRAM_SIZE],
}

#[derive(Copy, Clone)]
struct RenderContext<'a> {
    pub vram: &'a [u8; VRAM_SIZE],
    pub line: u16,
    pub registers: &'a GbaVideoRegisters,
}

impl<'a> RenderContext<'a> {
    pub fn new(line: u16, registers: &'a GbaVideoRegisters, vram: &'a [u8; VRAM_SIZE]) -> Self {
        Self {
            line,
            vram,
            registers,
        }
    }
}

#[inline]
pub const fn rgb5(r: u16, g: u16, b: u16) -> u16 {
    (r & 0x1F) | ((g & 0x1F) << 5) | ((b & 0x1F) << 10) | 0x8000
}
