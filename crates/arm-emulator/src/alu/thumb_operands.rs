use util::bits::BitOps;

use crate::Registers;

pub struct AddSubtractImm3;
pub struct AddSubtractReg3;

/// The value of a register
pub struct RegValue<const REG: u32>;

/// The register at the bit range START..=END
pub struct RegAt<const START: u32, const END: u32>;
/// The value of the register at the bit range START..=END
pub struct RegAtValue<const START: u32, const END: u32>;
pub struct WordAlignedPc;
/// The register itself (e.g. <ConstReg<15> as ExtractThumbOperand>::extract() == 15)
pub struct ConstReg<const REG: u32>;
pub struct ThumbRegisterList;
pub struct ThumbRegisterListWithLr;
pub struct ThumbRegisterListWithPc;

impl ExtractThumbOperand for ThumbRegisterList {
    fn extract(instr: u32, _registers: &Registers) -> u32 {
        instr & 0xFF
    }
}

impl ExtractThumbOperand for ThumbRegisterListWithLr {
    fn extract(instr: u32, _registers: &Registers) -> u32 {
        (instr & 0xFF) | (1 << 14)
    }
}

impl ExtractThumbOperand for ThumbRegisterListWithPc {
    fn extract(instr: u32, _registers: &Registers) -> u32 {
        (instr & 0xFF) | (1 << 15)
    }
}

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

impl<const START: u32, const END: u32> ExtractThumbOperand for RegAt<START, END> {
    fn extract(instr: u32, _registers: &Registers) -> u32 {
        instr.get_bit_range(START..=END)
    }
}

impl<const START: u32, const END: u32> ExtractThumbOperand for RegAtValue<START, END> {
    fn extract(instr: u32, registers: &Registers) -> u32 {
        let reg = instr.get_bit_range(START..=END);
        registers.read(reg)
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
