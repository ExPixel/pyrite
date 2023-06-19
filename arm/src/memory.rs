use crate::{cpu::Cycles, Registers};

pub trait Memory {
    fn code32(&mut self, address: u32, _sequential: bool, cycles: Option<&mut Cycles>) -> u32 {
        self.load32(address, cycles)
    }

    fn code16(&mut self, address: u32, _sequential: bool, cycles: Option<&mut Cycles>) -> u16 {
        self.load16(address, cycles)
    }

    fn load32(&mut self, address: u32, cycles: Option<&mut Cycles>) -> u32 {
        self.load16(address, None) as u32
            | ((self.load16(address.wrapping_add(2), cycles) as u32) << 16)
    }

    fn load16(&mut self, address: u32, cycles: Option<&mut Cycles>) -> u16 {
        self.load8(address, None) as u16
            | ((self.load8(address.wrapping_add(1), cycles) as u16) << 8)
    }

    fn load8(&mut self, address: u32, cycles: Option<&mut Cycles>) -> u8;

    fn store32(&mut self, address: u32, value: u32, cycles: Option<&mut Cycles>) {
        self.store16(address, value as u16, None);
        self.store16(address.wrapping_add(2), (value >> 16) as u16, cycles);
    }

    fn store16(&mut self, address: u32, value: u16, cycles: Option<&mut Cycles>) {
        self.store8(address, value as u8, None);
        self.store8(address.wrapping_add(1), (value >> 8) as u8, cycles);
    }

    fn store8(&mut self, address: u32, value: u8, cycles: Option<&mut Cycles>);

    fn load_multiple(
        &mut self,
        base_address: u32,
        register_mask: u16,
        transfer_type: BlockDataTransferType,
        registers: &mut Registers,
        cycles: Option<&mut Cycles>,
    ) {
        let (offset_before, offset_after) = transfer_type.offsets();
        let mut address = base_address;
        let mut cycles_tmp = Cycles::zero();
        for register in 0..16 {
            if (register_mask & (1 << register)) == 0 {
                continue;
            }
            address = address.wrapping_add(offset_before);
            registers.write(register, self.load32(address, Some(&mut cycles_tmp)));
            address = address.wrapping_add(offset_after);
        }

        if let Some(cycles) = cycles {
            *cycles += cycles_tmp;
        }
    }

    fn store_multiple(
        &mut self,
        base_address: u32,
        register_mask: u16,
        transfer_type: BlockDataTransferType,
        registers: &mut Registers,
        cycles: Option<&mut Cycles>,
    ) {
        let (offset_before, offset_after) = transfer_type.offsets();
        let mut address = base_address;
        let mut cycles_tmp = Cycles::zero();
        for register in 0..16 {
            if (register_mask & (1 << register)) == 0 {
                continue;
            }
            address = address.wrapping_add(offset_before);
            let value = registers.read(register);
            self.store32(address, value, Some(&mut cycles_tmp));
            address = address.wrapping_add(offset_after);
        }

        if let Some(cycles) = cycles {
            *cycles += cycles_tmp;
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum BlockDataTransferType {
    IncrementBefore,
    DecrementBefore,
    IncrementAfter,
    DecrementAfter,
}

impl BlockDataTransferType {
    #[inline]
    pub fn offsets(self) -> (u32, u32) {
        match self {
            BlockDataTransferType::IncrementBefore => (4, 0),
            BlockDataTransferType::DecrementBefore => (-4i32 as u32, 0),
            BlockDataTransferType::IncrementAfter => (0, 4),
            BlockDataTransferType::DecrementAfter => (0, -4i32 as u32),
        }
    }
}
