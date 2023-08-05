use arm::emu::Cycles;
use pyrite_derive::IoRegister;

use crate::{
    events::{GbaEvent, SharedGbaScheduler},
    GbaVideoOutput,
};

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

    pub(crate) reg_dispcnt: RegDispcnt,
    pub(crate) reg_dispstat: RegDispstat,
    pub(crate) reg_vcount: RegVcount,
}

impl GbaVideo {
    pub(crate) fn new(scheduler: SharedGbaScheduler) -> GbaVideo {
        GbaVideo {
            line_buffer: [0; VISIBLE_LINE_WIDTH],
            scheduler,

            reg_dispcnt: Default::default(),
            reg_dispstat: Default::default(),
            reg_vcount: Default::default(),
        }
    }

    fn render_line(&mut self, line: u16, video: &mut dyn GbaVideoOutput) {
        let d = std::time::SystemTime::UNIX_EPOCH.elapsed().unwrap();
        self.line_buffer
            .fill(line.wrapping_mul(d.as_millis() as u16));
        video.gba_line_ready(line as usize, &self.line_buffer);
    }

    pub(crate) fn reset(&mut self) {
        self.reg_vcount.set_current_scanline(LINE_COUNT as u16 - 1);
        self.begin_hdraw();
    }

    pub(crate) fn begin_hdraw(&mut self) {
        self.scheduler.schedule(GbaEvent::HBlank, HDRAW_CYCLES);

        let mut current_scanline = self.reg_vcount.current_scanline();
        if current_scanline >= (LINE_COUNT - 1) as u16 {
            current_scanline = 0;
        } else {
            current_scanline += 1;
        }
        self.reg_vcount.set_current_scanline(current_scanline);
    }

    pub(crate) fn begin_hblank(&mut self, video: &mut dyn GbaVideoOutput) {
        self.scheduler.schedule(GbaEvent::HDraw, HBLANK_CYCLES);

        let current_scanline = self.reg_vcount.current_scanline();
        if current_scanline < VISIBLE_LINE_COUNT as _ {
            self.render_line(current_scanline, video);
        }
    }
}

/// 4000000h - DISPCNT - LCD Control (Read/Write)
///   Bit   Expl.
///   0-2   BG Mode                (0-5=Video Mode 0-5, 6-7=Prohibited)
///   3     Reserved / CGB Mode    (0=GBA, 1=CGB; can be set only by BIOS opcodes)
///   4     Display Frame Select   (0-1=Frame 0-1) (for BG Modes 4,5 only)
///   5     H-Blank Interval Free  (1=Allow access to OAM during H-Blank)
///   6     OBJ Character VRAM Mapping (0=Two dimensional, 1=One dimensional)
///   7     Forced Blank           (1=Allow FAST access to VRAM,Palette,OAM)
///   8     Screen Display BG0  (0=Off, 1=On)
///   9     Screen Display BG1  (0=Off, 1=On)
///   10    Screen Display BG2  (0=Off, 1=On)
///   11    Screen Display BG3  (0=Off, 1=On)
///   12    Screen Display OBJ  (0=Off, 1=On)
///   13    Window 0 Display Flag   (0=Off, 1=On)
///   14    Window 1 Display Flag   (0=Off, 1=On)
///   15    OBJ Window Display Flag (0=Off, 1=On)
#[derive(IoRegister, Copy, Clone)]
#[field(bg_mode: BgMode = 0..=2)]
#[field(display_frame_select: DisplayFrame = 4)]
#[field(hblank_interval_free: bool = 5)]
#[field(obj_character_vram_mapping: ObjCharVramMapping = 6)]
#[field(forced_blank: bool = 7)]
#[field(screen_display_bg0: bool = 8)]
#[field(screen_display_bg1: bool = 9)]
#[field(screen_display_bg2: bool = 10)]
#[field(screen_display_bg3: bool = 11)]
#[field(screen_display_obj: bool = 12)]
#[field(window0_display: bool = 13)]
#[field(window1_display: bool = 14)]
#[field(obj_window_display: bool = 15)]
pub struct RegDispcnt {
    value: u16,
}

/// 4000004h - DISPSTAT - General LCD Status (Read/Write)
/// Display status and Interrupt control. The H-Blank conditions are generated once per scanline, including for the 'hidden' scanlines during V-Blank.
///   Bit   Expl.
///   0     V-Blank flag   (Read only) (1=VBlank) (set in line 160..226; not 227)
///   1     H-Blank flag   (Read only) (1=HBlank) (toggled in all lines, 0..227)
///   2     V-Counter flag (Read only) (1=Match)  (set in selected line)     (R)
///   3     V-Blank IRQ Enable         (1=Enable)                          (R/W)
///   4     H-Blank IRQ Enable         (1=Enable)                          (R/W)
///   5     V-Counter IRQ Enable       (1=Enable)                          (R/W)
///   6     Not used (0) / DSi: LCD Initialization Ready (0=Busy, 1=Ready)   (R)
///   7     Not used (0) / NDS: MSB of V-Vcount Setting (LYC.Bit8) (0..262)(R/W)
///   8-15  V-Count Setting (LYC)      (0..227)                            (R/W)
/// The V-Count-Setting value is much the same as LYC of older gameboys, when its value is identical to the content of the VCOUNT register then the V-Counter flag is set (Bit 2), and (if enabled in Bit 5) an interrupt is requested.
/// Although the drawing time is only 960 cycles (240*4), the H-Blank flag is "0" for a total of 1006 cycles.
#[derive(IoRegister, Copy, Clone)]
#[field(vblank_flag: readonly<bool> = 0)]
#[field(hblank_flag: readonly<bool> = 1)]
#[field(v_counter_flag: readonly<bool> = 2)]
#[field(vblank_irq_enable: bool = 3)]
#[field(hblank_irq_enable: bool = 3)]
#[field(v_counter_irq_enable: bool = 3)]
#[field(v_count_setting: u16 = 8..=15)]
pub struct RegDispstat {
    value: u16,
}

/// 4000006h - VCOUNT - Vertical Counter (Read only)
/// Indicates the currently drawn scanline, values in range from 160..227 indicate 'hidden' scanlines within VBlank area.
///   Bit   Expl.
///   0-7   Current Scanline (LY)      (0..227)                              (R)
///   8     Not used (0) / NDS: MSB of Current Scanline (LY.Bit8) (0..262)   (R)
///   9-15  Not Used (0)
/// Note: This is much the same than the 'LY' register of older gameboys.
#[derive(IoRegister, Copy, Clone)]
#[field(current_scanline: readonly<u16> = 0..=7)]
#[field(not_used_bit_8: readonly<u16> = 8)]
pub struct RegVcount {
    value: u16,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum BgMode {
    Mode0,
    Mode1,
    Mode2,
    Mode3,
    Mode4,
    Mode5,
    Invalid6,
    Invalid7,
}

impl From<u16> for BgMode {
    fn from(value: u16) -> Self {
        match value {
            0 => BgMode::Mode0,
            1 => BgMode::Mode1,
            2 => BgMode::Mode2,
            3 => BgMode::Mode3,
            4 => BgMode::Mode4,
            5 => BgMode::Mode5,
            6 => BgMode::Invalid6,
            7 => BgMode::Invalid7,
            _ => unreachable!(),
        }
    }
}

impl From<BgMode> for u16 {
    fn from(value: BgMode) -> Self {
        match value {
            BgMode::Mode0 => 0,
            BgMode::Mode1 => 1,
            BgMode::Mode2 => 2,
            BgMode::Mode3 => 3,
            BgMode::Mode4 => 4,
            BgMode::Mode5 => 5,
            BgMode::Invalid6 => 6,
            BgMode::Invalid7 => 7,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum DisplayFrame {
    Mode0,
    Mode1,
}

impl From<DisplayFrame> for u16 {
    fn from(value: DisplayFrame) -> Self {
        match value {
            DisplayFrame::Mode0 => 0,
            DisplayFrame::Mode1 => 1,
        }
    }
}

impl From<u16> for DisplayFrame {
    fn from(value: u16) -> Self {
        if value == 0 {
            DisplayFrame::Mode0
        } else {
            DisplayFrame::Mode1
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ObjCharVramMapping {
    OneDimensional,
    TwoDimensional,
}

impl From<ObjCharVramMapping> for u16 {
    fn from(value: ObjCharVramMapping) -> Self {
        match value {
            ObjCharVramMapping::OneDimensional => 0,
            ObjCharVramMapping::TwoDimensional => 1,
        }
    }
}

impl From<u16> for ObjCharVramMapping {
    fn from(value: u16) -> Self {
        if value == 0 {
            ObjCharVramMapping::OneDimensional
        } else {
            ObjCharVramMapping::TwoDimensional
        }
    }
}

#[inline]
pub const fn rgb16(r: u16, g: u16, b: u16) -> u16 {
    (r & 0x1F) | ((g & 0x1F) << 5) | ((b & 0x1F) << 10)
}
