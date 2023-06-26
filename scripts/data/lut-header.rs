// GENERATED BY scripts/generate-lookup-tables.py
use super::cpu::InstrFn;
use super::{alu, arm, thumb};
use crate::transfer::{
    Ldr, LdrB, PostDecrement, PostIncrement, PreDecrement, PreIncrement, SDTImmOffset, Str, StrB,
};
use util::bits::BitOps as _;

pub fn decode_arm_opcode(opcode: u32) -> InstrFn {
    let opcode_row = opcode.get_bit_range(20..=27);
    let opcode_col = opcode.get_bit_range(4..=7);
    let opcode_idx = (opcode_row * 16) + opcode_col;
    ARM_OPCODE_TABLE[opcode_idx as usize]
}

pub fn decode_thumb_opcode(opcode: u32) -> InstrFn {
    let opcode_row = opcode.get_bit_range(12..=15);
    let opcode_col = opcode.get_bit_range(8..=11);
    let opcode_idx = (opcode_row * 16) + opcode_col;
    THUMB_OPCODE_TABLE[opcode_idx as usize]
}

pub const S_FLAG_SET: bool = true;
pub const S_FLAG_CLR: bool = false;
pub const FORCE_USER_MODE: bool = false;
