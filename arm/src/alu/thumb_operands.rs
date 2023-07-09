use util::bits::BitOps;

use crate::Registers;

pub struct AddSubtractImm3;
pub struct AddSubtractReg3;

impl ExtractThumbOperand for AddSubtractImm3 {
    #[inline]
    fn extract(instr: u32, _registers: &Registers) -> u32 {
        instr.get_bit_range(6..=8)
    }
}

impl ExtractThumbOperand for AddSubtractReg3 {
    #[inline]
    fn extract(instr: u32, registers: &Registers) -> u32 {
        let rn = instr.get_bit_range(6..=8);
        registers.read(rn)
    }
}

pub trait ExtractThumbOperand {
    fn extract(instr: u32, registers: &Registers) -> u32;
}
