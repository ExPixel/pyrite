pub mod palette;
pub mod video;

use self::{palette::Palette, video::GbaVideo};

pub struct GbaMemoryMappedHardware {
    pub bios: Box<[u8; BIOS_SIZE]>,
    pub ewram: Box<[u8; EWRAM_SIZE]>,
    pub iwram: Box<[u8; IWRAM_SIZE]>,
    pub video: Box<GbaVideo>,
    pub palram: Box<Palette>,
    pub vram: Box<[u8; VRAM_SIZE]>,
    pub oam: Box<[u8; OAM_SIZE]>,
    pub gamepak: Vec<u8>,
}

impl GbaMemoryMappedHardware {
    pub fn new() -> Self {
        Self {
            video: Box::default(),
            bios: Box::new([0; BIOS_SIZE]),
            ewram: Box::new([0; EWRAM_SIZE]),
            iwram: Box::new([0; IWRAM_SIZE]),
            palram: Box::default(),
            vram: Box::new([0; VRAM_SIZE]),
            oam: Box::new([0; OAM_SIZE]),
            gamepak: Vec::new(),
        }
    }
}

impl Default for GbaMemoryMappedHardware {
    fn default() -> Self {
        Self::new()
    }
}

pub const REGION_BIOS: u32 = 0x0;
pub const REGION_UNUSED_1: u32 = 0x1;
pub const REGION_EWRAM: u32 = 0x2;
pub const REGION_IWRAM: u32 = 0x3;
pub const REGION_IOREGS: u32 = 0x4;
pub const REGION_PAL: u32 = 0x5;
pub const REGION_VRAM: u32 = 0x6;
pub const REGION_OAM: u32 = 0x7;
pub const REGION_GAMEPAK0_LO: u32 = 0x8;
pub const REGION_GAMEPAK0_HI: u32 = 0x9;
pub const REGION_GAMEPAK1_LO: u32 = 0xA;
pub const REGION_GAMEPAK1_HI: u32 = 0xB;
pub const REGION_GAMEPAK2_LO: u32 = 0xC;
pub const REGION_GAMEPAK2_HI: u32 = 0xD;
pub const REGION_SRAM: u32 = 0xE;

pub const BIOS_SIZE: usize = 0x4000;
pub const EWRAM_SIZE: usize = 0x40000;
pub const IWRAM_SIZE: usize = 0x8000;
pub const PAL_SIZE: usize = 0x400;
pub const VRAM_SIZE: usize = 0x18000;
pub const OAM_SIZE: usize = 0x400;
pub const IOREGS_SIZE: usize = 0x20A;

pub const EWRAM_MASK: u32 = 0x3FFFF;
pub const IWRAM_MASK: u32 = 0x7FFF;
pub const PAL_MASK: u32 = 0x3FF;
pub const OAM_MASK: u32 = 0x3FF;
pub const ROM_MAX_MASK: u32 = 0xFFFFFF;
