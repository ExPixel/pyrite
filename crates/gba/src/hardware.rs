pub mod palette;
pub mod system_control;
pub mod video;

use crate::memory::{BIOS_SIZE, EWRAM_SIZE, IWRAM_SIZE, OAM_SIZE, VRAM_SIZE};

use self::{
    palette::Palette,
    system_control::{RegInternalMemoryControl, SystemControl},
    video::GbaVideo,
};

pub struct GbaMemoryMappedHardware {
    pub bios: Box<[u8; BIOS_SIZE]>,
    pub ewram: Box<[u8; EWRAM_SIZE]>,
    pub iwram: Box<[u8; IWRAM_SIZE]>,

    pub video: Box<GbaVideo>,
    pub system_control: SystemControl,

    pub palram: Box<Palette>,
    pub vram: Box<[u8; VRAM_SIZE]>,
    pub oam: Box<[u8; OAM_SIZE]>,

    pub(crate) gamepak_mask: usize,
    pub(crate) gamepak: Vec<u8>,

    /// The last value ready from memory.
    pub(crate) last_read_value: u32,
    /// The last value read from BIOS.
    pub(crate) last_bios_value: u32,
}

impl GbaMemoryMappedHardware {
    pub fn new() -> Self {
        Self {
            bios: Box::new([0; BIOS_SIZE]),
            ewram: Box::new([0; EWRAM_SIZE]),
            iwram: Box::new([0; IWRAM_SIZE]),

            video: Box::default(),
            system_control: SystemControl::default(),

            palram: Box::default(),
            vram: Box::new([0; VRAM_SIZE]),
            oam: Box::new([0; OAM_SIZE]),

            gamepak_mask: 0,
            gamepak: vec![0; 4],

            last_read_value: 0,
            last_bios_value: 0,
        }
    }

    /// Called after a hard reset of the GBA.
    pub(crate) fn reset(&mut self) {
        self.system_control
            .write_internal_memory_control(RegInternalMemoryControl::DEFAULT);
    }

    pub fn set_gamepak(&mut self, mut new_gamepak: Vec<u8>) {
        assert!(!new_gamepak.is_empty());
        let gamepak_size = new_gamepak.len().next_power_of_two();
        new_gamepak.resize(gamepak_size, 0);
        self.gamepak = new_gamepak;
        self.gamepak_mask = gamepak_size - 1;
    }
}

impl Default for GbaMemoryMappedHardware {
    fn default() -> Self {
        Self::new()
    }
}

pub const CUSTOM_BIOS: &[u8] = include_bytes!("../../../roms/custom/custom-bios.bin");
