mod events;
mod hardware;
pub mod memory;

use arm::emu::{Cpu, CpuMode, InstructionSet};
use events::SharedGbaScheduler;
use hardware::CUSTOM_BIOS;
pub use hardware::{video, GbaMemoryMappedHardware};

pub struct Gba {
    pub cpu: Cpu,
    pub mapped: GbaMemoryMappedHardware,
    pub scheduler: SharedGbaScheduler,
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
        self.mapped.reset();
    }

    pub fn step(&mut self, video_out: &mut dyn GbaVideoOutput, audio_out: &mut dyn GbaAudioOutput) {
        let _unused = audio_out;

        self.cpu.step(&mut self.mapped);

        self.mapped.video.current_line = (self.mapped.video.current_line + 1) % 240;
        if self.mapped.video.current_line < 160 {
            static mut COLOR: u16 = 0;
            for c in self.mapped.video.line_buffer.iter_mut() {
                unsafe {
                    COLOR = COLOR.wrapping_add(self.mapped.video.current_line as u16 % 4);
                };
                *c = unsafe { COLOR };
            }

            video_out.gba_line_ready(
                self.mapped.video.current_line,
                &self.mapped.video.line_buffer,
            );
        }
    }

    pub fn set_gamepak(&mut self, gamepak: Vec<u8>) {
        self.mapped.set_gamepak(gamepak);
    }
}

impl Default for Gba {
    fn default() -> Self {
        Self::new()
    }
}

// SAFETY: don't let the scheduler escape the GBA step function
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
