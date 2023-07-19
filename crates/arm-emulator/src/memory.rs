use crate::clock::Waitstates;

pub trait Memory {
    fn load32(&mut self, address: u32, access: AccessType) -> (u32, Waitstates) {
        let (lo, wait_lo) = self.load16(address, access);
        let (hi, wait_hi) = self.load16(address.wrapping_add(2), access);
        ((lo as u32) | ((hi as u32) << 16), wait_lo + wait_hi)
    }

    fn load16(&mut self, address: u32, access: AccessType) -> (u16, Waitstates) {
        let (lo, wait_lo) = self.load8(address, access);
        let (hi, wait_hi) = self.load8(address.wrapping_add(1), access);
        ((lo as u16) | ((hi as u16) << 8), wait_lo + wait_hi)
    }

    fn load8(&mut self, address: u32, access: AccessType) -> (u8, Waitstates);

    fn store32(&mut self, address: u32, value: u32, access: AccessType) -> Waitstates {
        let wait_lo = self.store16(address, value as u16, access);
        let wait_hi = self.store16(address.wrapping_add(2), (value >> 16) as u16, access);
        wait_lo + wait_hi
    }

    fn store16(&mut self, address: u32, value: u16, access: AccessType) -> Waitstates {
        let wait_lo = self.store8(address, value as u8, access);
        let wait_hi = self.store8(address.wrapping_add(1), (value >> 8) as u8, access);
        wait_lo + wait_hi
    }

    fn store8(&mut self, address: u32, value: u8, access: AccessType) -> Waitstates;
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum AccessType {
    Sequential,
    NonSequential,
}
