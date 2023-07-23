mod hardware;
pub mod memory;

use arm::emu::{Cpu, CpuMode, InstructionSet};
pub use hardware::video;
use hardware::{GbaMemoryMappedHardware, CUSTOM_BIOS};

pub struct Gba {
    cpu: Cpu,
    mapped: GbaMemoryMappedHardware,
}

impl Gba {
    pub fn new() -> Self {
        let mut mmh = GbaMemoryMappedHardware::new();
        assert!(CUSTOM_BIOS.len() <= memory::BIOS_SIZE);
        mmh.bios[..CUSTOM_BIOS.len()].copy_from_slice(CUSTOM_BIOS);

        let cpu = Cpu::new(InstructionSet::Arm, CpuMode::System, &mut mmh);
        Self { cpu, mapped: mmh }
    }

    /// Hard reset.
    pub fn reset(&mut self) {
        self.cpu.branch(0, &mut self.mapped);
        self.mapped.reset();
    }

    pub fn step(&mut self, video_out: &mut dyn GbaVideoOutput, audio_out: &mut dyn GbaAudioOutput) {
        let _unused = audio_out;

        // FIXME debug code
        if self.cpu.next_execution_address() < 0x08000004 {
            if self.cpu.next_execution_address() == 0x08000000 {
                tracing::debug!("GBA entry point reached");
            }
            self.cpu.step(&mut self.mapped);
        }

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

    pub fn cpu(&self) -> &Cpu {
        &self.cpu
    }

    pub fn hardware(&self) -> &GbaMemoryMappedHardware {
        &self.mapped
    }
}

impl Default for Gba {
    fn default() -> Self {
        Self::new()
    }
}

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
