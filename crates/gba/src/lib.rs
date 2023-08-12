mod events;
mod hardware;
pub mod memory;

use arm::emu::{Cpu, CpuMode, Cycles, InstructionSet};
use events::{GbaEvent, SharedGbaScheduler};
pub use hardware::{video, GbaMemoryMappedHardware};
use hardware::{video::HBlankContext, CUSTOM_BIOS};

pub const NOP_ROM: [u8; 4] = [0xFE, 0xFF, 0xFF, 0xEA];

pub struct Gba {
    pub cpu: Cpu,
    pub mapped: GbaMemoryMappedHardware,
    scheduler: SharedGbaScheduler,
}

impl Gba {
    pub fn new() -> Self {
        let scheduler = SharedGbaScheduler::default();

        let mut mmh = GbaMemoryMappedHardware::new(scheduler.clone());
        assert!(CUSTOM_BIOS.len() <= memory::BIOS_SIZE);
        mmh.bios[..CUSTOM_BIOS.len()].copy_from_slice(CUSTOM_BIOS);

        let cpu = Cpu::new(InstructionSet::Arm, CpuMode::System, &mut mmh);
        Self {
            cpu,
            mapped: mmh,
            scheduler,
        }
    }

    /// Hard reset.
    pub fn reset(&mut self) {
        self.cpu.branch(0, &mut self.mapped);
        self.scheduler.clear();
        self.mapped.reset();
    }

    pub fn step(&mut self, video_out: &mut dyn GbaVideoOutput, audio_out: &mut dyn GbaAudioOutput) {
        let _unused = audio_out;

        let mut cycles = self.cpu.step(&mut self.mapped);
        while let Some(event) = self.scheduler.tick(&mut cycles) {
            self.handle_event(event, cycles, video_out);
        }
    }

    fn handle_event(&mut self, event: GbaEvent, _late: Cycles, video_out: &mut dyn GbaVideoOutput) {
        match event {
            GbaEvent::HDraw => self.mapped.video.begin_hdraw(),
            GbaEvent::HBlank => {
                let context = HBlankContext {
                    palette: &self.mapped.palram,
                    vram: &self.mapped.vram,
                };
                self.mapped.video.begin_hblank(video_out, context);
            }
            GbaEvent::Test => unreachable!(),
        }
    }

    pub fn set_gamepak(&mut self, gamepak: Vec<u8>) {
        self.mapped.set_gamepak(gamepak);
    }

    pub fn set_noop_gamepak(&mut self) {
        self.mapped.set_gamepak(NOP_ROM.to_vec());
    }

    pub fn frame_count(&self) -> u64 {
        self.mapped.video.frame
    }
}

impl Default for Gba {
    fn default() -> Self {
        Self::new()
    }
}

// SAFETY: don't let the scheduler escape the GBA
unsafe impl Send for Gba {}
unsafe impl Sync for Gba {}

pub struct NoopGbaAudioOutput;

pub trait GbaVideoOutput {
    fn gba_line_ready(&mut self, line: usize, data: &video::LineBuffer);
}

pub struct NoopGbaVideoOutput;

impl GbaVideoOutput for NoopGbaVideoOutput {
    fn gba_line_ready(&mut self, _line: usize, _data: &video::LineBuffer) {
        // NOOP
    }
}

pub trait GbaAudioOutput {}

impl GbaAudioOutput for NoopGbaAudioOutput {}
