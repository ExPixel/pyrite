use std::fmt::Write;

use crate::{lookup::decode_arm_opcode, DisasmOptions};

pub fn disasm_arm(instr: u32, address: u32, options: &DisasmOptions) -> ArmInstruction {
    (decode_arm_opcode(instr))(instr, address, options)
}

pub enum ArmInstruction {
    Undefined,
}

pub fn disasm_b(_instr: u32, _address: u32, _options: &DisasmOptions) -> ArmInstruction {
    ArmInstruction::Undefined
}

pub fn disasm_bkpt(_instr: u32, _address: u32, _options: &DisasmOptions) -> ArmInstruction {
    ArmInstruction::Undefined
}

pub fn disasm_bl(_instr: u32, _address: u32, _options: &DisasmOptions) -> ArmInstruction {
    ArmInstruction::Undefined
}

pub fn disasm_block_data_transfer(
    instr: u32,
    address: u32,
    options: &DisasmOptions,
) -> ArmInstruction {
    ArmInstruction::Undefined
}

pub fn disasm_blx(_instr: u32, _address: u32, _options: &DisasmOptions) -> ArmInstruction {
    ArmInstruction::Undefined
}

pub fn disasm_bx(_instr: u32, _address: u32, _options: &DisasmOptions) -> ArmInstruction {
    ArmInstruction::Undefined
}

pub fn disasm_clz(_instr: u32, _address: u32, _options: &DisasmOptions) -> ArmInstruction {
    ArmInstruction::Undefined
}

pub fn disasm_coprocessor_instr(
    instr: u32,
    address: u32,
    options: &DisasmOptions,
) -> ArmInstruction {
    ArmInstruction::Undefined
}

pub fn disasm_dataproc(_instr: u32, _address: u32, _options: &DisasmOptions) -> ArmInstruction {
    ArmInstruction::Undefined
}

pub fn disasm_m_extension_undefined(
    instr: u32,
    address: u32,
    options: &DisasmOptions,
) -> ArmInstruction {
    ArmInstruction::Undefined
}

pub fn disasm_mrs(_instr: u32, _address: u32, _options: &DisasmOptions) -> ArmInstruction {
    ArmInstruction::Undefined
}

pub fn disasm_msr(_instr: u32, _address: u32, _options: &DisasmOptions) -> ArmInstruction {
    ArmInstruction::Undefined
}

pub fn disasm_mul(_instr: u32, _address: u32, _options: &DisasmOptions) -> ArmInstruction {
    ArmInstruction::Undefined
}

pub fn disasm_mul_long(_instr: u32, _address: u32, _options: &DisasmOptions) -> ArmInstruction {
    ArmInstruction::Undefined
}

pub fn disasm_single_data_transfer(
    instr: u32,
    address: u32,
    options: &DisasmOptions,
) -> ArmInstruction {
    ArmInstruction::Undefined
}

pub fn disasm_swi(_instr: u32, _address: u32, _options: &DisasmOptions) -> ArmInstruction {
    ArmInstruction::Undefined
}

pub fn disasm_swp(_instr: u32, _address: u32, _options: &DisasmOptions) -> ArmInstruction {
    ArmInstruction::Undefined
}

pub fn disasm_undefined(_instr: u32, _address: u32, _options: &DisasmOptions) -> ArmInstruction {
    ArmInstruction::Undefined
}
