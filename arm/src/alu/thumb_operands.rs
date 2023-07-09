use util::bits::BitOps;

use crate::Registers;

pub struct AddSubtractImm3;
pub struct AddSubtractReg3;

/// The value of a register
pub struct RegValue<const REG: u32>;
pub struct WordAlignedPc;
/// The register itself (e.g. <ConstReg<15> as ExtractThumbOperand>::extract() == 15)
pub struct ConstReg<const REG: u32>;

impl<const REG: u32> ExtractThumbOperand for ConstReg<REG> {
    fn extract(_instr: u32, _registers: &Registers) -> u32 {
        REG
    }
}

impl<const REG: u32> ExtractThumbOperand for RegValue<REG> {
    fn extract(_instr: u32, registers: &Registers) -> u32 {
        registers.read(REG)
    }
}

impl ExtractThumbOperand for WordAlignedPc {
    fn extract(_instr: u32, registers: &Registers) -> u32 {
        registers.read(15) & !0x3
    }
}

impl ExtractThumbOperand for AddSubtractImm3 {
    fn extract(instr: u32, _registers: &Registers) -> u32 {
        instr.get_bit_range(6..=8)
    }
}

impl ExtractThumbOperand for AddSubtractReg3 {
    fn extract(instr: u32, registers: &Registers) -> u32 {
        let rn = instr.get_bit_range(6..=8);
        registers.read(rn)
    }
}

pub trait ExtractThumbOperand {
    fn extract(instr: u32, registers: &Registers) -> u32;
}
