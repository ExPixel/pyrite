use arm::emu::{Cpu, Memory};

use crate::hardware::GbaMemoryMappedHardware;

impl GbaMemoryMappedHardware {
    pub fn view8(&self, _address: u32) -> u8 {
        0
    }

    pub fn view16(&self, _address: u32) -> u16 {
        0
    }

    pub fn view32(&self, _address: u32) -> u32 {
        0
    }
}

impl Memory for GbaMemoryMappedHardware {
    fn load8(&mut self, _address: u32, _cpu: &mut Cpu) -> (u8, arm::emu::Waitstates) {
        (0, arm::emu::Waitstates::zero())
    }

    fn store8(&mut self, _address: u32, _value: u8, _cpu: &mut Cpu) -> arm::emu::Waitstates {
        arm::emu::Waitstates::zero()
    }
}
