use crate::{lookup::decode_thumb_opcode, Arguments, DisasmOptions, Mnemonic};

pub fn disasm_thumb(instr: u16, address: u32, options: &DisasmOptions) -> ThumbInstruction {
    (decode_thumb_opcode(instr))(instr, address, options)
}

pub enum ThumbInstruction {
    Undefined,
}

impl ThumbInstruction {
    pub fn mnemonic<'s>(&'s self, options: &'s DisasmOptions) -> Mnemonic<Self> {
        Mnemonic(self, options)
    }

    pub fn arguments<'s>(&'s self, options: &'s DisasmOptions) -> Arguments<Self> {
        Arguments(self, options)
    }

    pub(crate) fn write_mnemonic(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        o: &DisasmOptions,
    ) -> std::fmt::Result {
        match self {
            ThumbInstruction::Undefined => {
                write!(f, "undef")
            }
        }
    }

    pub(crate) fn write_args(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        _o: &DisasmOptions,
    ) -> std::fmt::Result {
        match self {
            ThumbInstruction::Undefined => write!(f, "???"),
        }
    }
}

pub fn disasm_add_sp(_instr: u16, _address: u32, _options: &DisasmOptions) -> ThumbInstruction {
    ThumbInstruction::Undefined
}

pub fn disasm_add_subtract(
    _instr: u16,
    _address: u32,
    _options: &DisasmOptions,
) -> ThumbInstruction {
    ThumbInstruction::Undefined
}

pub fn disasm_alu_operation(
    _instr: u16,
    _address: u32,
    _options: &DisasmOptions,
) -> ThumbInstruction {
    ThumbInstruction::Undefined
}

pub fn disasm_bkpt(_instr: u16, _address: u32, _options: &DisasmOptions) -> ThumbInstruction {
    ThumbInstruction::Undefined
}

pub fn disasm_bl_complete(
    _instr: u16,
    _address: u32,
    _options: &DisasmOptions,
) -> ThumbInstruction {
    ThumbInstruction::Undefined
}

pub fn disasm_bl_setup(_instr: u16, _address: u32, _options: &DisasmOptions) -> ThumbInstruction {
    ThumbInstruction::Undefined
}

pub fn disasm_block_data_transfer(
    _instr: u16,
    _address: u32,
    _options: &DisasmOptions,
) -> ThumbInstruction {
    ThumbInstruction::Undefined
}

pub fn disasm_blx(_instr: u16, _address: u32, _options: &DisasmOptions) -> ThumbInstruction {
    ThumbInstruction::Undefined
}

pub fn disasm_bx(_instr: u16, _address: u32, _options: &DisasmOptions) -> ThumbInstruction {
    ThumbInstruction::Undefined
}

pub fn disasm_conditional_branch(
    _instr: u16,
    _address: u32,
    _options: &DisasmOptions,
) -> ThumbInstruction {
    ThumbInstruction::Undefined
}

pub fn disasm_hi_register_op(
    _instr: u16,
    _address: u32,
    _options: &DisasmOptions,
) -> ThumbInstruction {
    ThumbInstruction::Undefined
}

pub fn disasm_load_address(
    _instr: u16,
    _address: u32,
    _options: &DisasmOptions,
) -> ThumbInstruction {
    ThumbInstruction::Undefined
}

pub fn disasm_mov_compare_add_subtract_imm(
    _instr: u16,
    _address: u32,
    _options: &DisasmOptions,
) -> ThumbInstruction {
    ThumbInstruction::Undefined
}

pub fn disasm_move_shifted_register(
    _instr: u16,
    _address: u32,
    _options: &DisasmOptions,
) -> ThumbInstruction {
    ThumbInstruction::Undefined
}

pub fn disasm_single_data_transfer(
    _instr: u16,
    _address: u32,
    _options: &DisasmOptions,
) -> ThumbInstruction {
    ThumbInstruction::Undefined
}

pub fn disasm_swi(_instr: u16, _address: u32, _options: &DisasmOptions) -> ThumbInstruction {
    ThumbInstruction::Undefined
}

pub fn disasm_unconditional_branch(
    _instr: u16,
    _address: u32,
    _options: &DisasmOptions,
) -> ThumbInstruction {
    ThumbInstruction::Undefined
}

pub fn disasm_undefined(_instr: u16, _address: u32, _options: &DisasmOptions) -> ThumbInstruction {
    ThumbInstruction::Undefined
}
