use std::any::Any;

use crate::{clock::Waitstates, Cpu};

pub trait Memory {
    fn load32(&mut self, address: u32, cpu: &mut Cpu) -> (u32, Waitstates) {
        let (lo, wait_lo) = self.load16(address, cpu);
        let (hi, wait_hi) = self.load16(address.wrapping_add(2), cpu);
        ((lo as u32) | ((hi as u32) << 16), wait_lo + wait_hi)
    }

    fn load16(&mut self, address: u32, cpu: &mut Cpu) -> (u16, Waitstates) {
        let (lo, wait_lo) = self.load8(address, cpu);
        let (hi, wait_hi) = self.load8(address.wrapping_add(1), cpu);
        ((lo as u16) | ((hi as u16) << 8), wait_lo + wait_hi)
    }

    fn load8(&mut self, address: u32, cpu: &mut Cpu) -> (u8, Waitstates);

    fn store32(&mut self, address: u32, value: u32, cpu: &mut Cpu) -> Waitstates {
        let wait_lo = self.store16(address, value as u16, cpu);
        let wait_hi = self.store16(address.wrapping_add(2), (value >> 16) as u16, cpu);
        wait_lo + wait_hi
    }

    fn store16(&mut self, address: u32, value: u16, cpu: &mut Cpu) -> Waitstates {
        let wait_lo = self.store8(address, value as u8, cpu);
        let wait_hi = self.store8(address.wrapping_add(1), (value >> 8) as u8, cpu);
        wait_lo + wait_hi
    }

    fn store8(&mut self, address: u32, value: u8, cpu: &mut Cpu) -> Waitstates;

    fn as_any(&self) -> &dyn Any;
    fn as_mut_any(&mut self) -> &mut dyn Any;
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum AccessType {
    Sequential,
    NonSequential,
}
