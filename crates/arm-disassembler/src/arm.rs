use util::bits::BitOps;

use crate::{lookup::decode_arm_opcode, Arguments, DisasmOptions, Mnemonic};

pub fn disasm_arm(instr: u32, address: u32, options: &DisasmOptions) -> ArmInstruction {
    (decode_arm_opcode(instr))(instr, address, options)
}

pub enum ArmInstruction {
    Undefined { condition: Condition },
}

impl ArmInstruction {
    pub fn mnemonic<'s>(&'s self, options: &'s DisasmOptions) -> Mnemonic<Self> {
        Mnemonic(self, options)
    }

    pub fn arguments<'s>(&'s self, options: &'s DisasmOptions) -> Arguments<Self> {
        Arguments(self, options)
    }

    pub fn condition(&self) -> Condition {
        match self {
            &ArmInstruction::Undefined { condition, .. } => condition,
        }
    }

    pub(crate) fn write_mnemonic(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        o: &DisasmOptions,
    ) -> std::fmt::Result {
        match self {
            ArmInstruction::Undefined { condition } => {
                write!(f, "undef{}", condition.as_str(o.uppercase_mnemonic))
            }
        }
    }

    pub(crate) fn write_args(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        _o: &DisasmOptions,
    ) -> std::fmt::Result {
        match self {
            ArmInstruction::Undefined { .. } => write!(f, "???"),
        }
    }
}

pub fn disasm_b(instr: u32, _address: u32, _options: &DisasmOptions) -> ArmInstruction {
    let condition = Condition::from_bits(instr.get_bit_range(28..=31));
    ArmInstruction::Undefined { condition }
}

pub fn disasm_bkpt(instr: u32, _address: u32, _options: &DisasmOptions) -> ArmInstruction {
    let condition = Condition::from_bits(instr.get_bit_range(28..=31));
    ArmInstruction::Undefined { condition }
}

pub fn disasm_bl(instr: u32, _address: u32, _options: &DisasmOptions) -> ArmInstruction {
    let condition = Condition::from_bits(instr.get_bit_range(28..=31));
    ArmInstruction::Undefined { condition }
}

pub fn disasm_block_data_transfer(
    instr: u32,
    _address: u32,
    _options: &DisasmOptions,
) -> ArmInstruction {
    let condition = Condition::from_bits(instr.get_bit_range(28..=31));
    ArmInstruction::Undefined { condition }
}

pub fn disasm_blx(instr: u32, _address: u32, _options: &DisasmOptions) -> ArmInstruction {
    let condition = Condition::from_bits(instr.get_bit_range(28..=31));
    ArmInstruction::Undefined { condition }
}

pub fn disasm_bx(instr: u32, _address: u32, _options: &DisasmOptions) -> ArmInstruction {
    let condition = Condition::from_bits(instr.get_bit_range(28..=31));
    ArmInstruction::Undefined { condition }
}

pub fn disasm_clz(instr: u32, _address: u32, _options: &DisasmOptions) -> ArmInstruction {
    let condition = Condition::from_bits(instr.get_bit_range(28..=31));
    ArmInstruction::Undefined { condition }
}

pub fn disasm_coprocessor_instr(
    instr: u32,
    _address: u32,
    _options: &DisasmOptions,
) -> ArmInstruction {
    let condition = Condition::from_bits(instr.get_bit_range(28..=31));
    ArmInstruction::Undefined { condition }
}

pub fn disasm_dataproc(instr: u32, _address: u32, _options: &DisasmOptions) -> ArmInstruction {
    let condition = Condition::from_bits(instr.get_bit_range(28..=31));
    ArmInstruction::Undefined { condition }
}

pub fn disasm_m_extension_undefined(
    instr: u32,
    _address: u32,
    _options: &DisasmOptions,
) -> ArmInstruction {
    let condition = Condition::from_bits(instr.get_bit_range(28..=31));
    ArmInstruction::Undefined { condition }
}

pub fn disasm_mrs(instr: u32, _address: u32, _options: &DisasmOptions) -> ArmInstruction {
    let condition = Condition::from_bits(instr.get_bit_range(28..=31));
    ArmInstruction::Undefined { condition }
}

pub fn disasm_msr(instr: u32, _address: u32, _options: &DisasmOptions) -> ArmInstruction {
    let condition = Condition::from_bits(instr.get_bit_range(28..=31));
    ArmInstruction::Undefined { condition }
}

pub fn disasm_mul(instr: u32, _address: u32, _options: &DisasmOptions) -> ArmInstruction {
    let condition = Condition::from_bits(instr.get_bit_range(28..=31));
    ArmInstruction::Undefined { condition }
}

pub fn disasm_mul_long(instr: u32, _address: u32, _options: &DisasmOptions) -> ArmInstruction {
    let condition = Condition::from_bits(instr.get_bit_range(28..=31));
    ArmInstruction::Undefined { condition }
}

pub fn disasm_single_data_transfer(
    instr: u32,
    _address: u32,
    _options: &DisasmOptions,
) -> ArmInstruction {
    let condition = Condition::from_bits(instr.get_bit_range(28..=31));
    ArmInstruction::Undefined { condition }
}

pub fn disasm_swi(instr: u32, _address: u32, _options: &DisasmOptions) -> ArmInstruction {
    let condition = Condition::from_bits(instr.get_bit_range(28..=31));
    ArmInstruction::Undefined { condition }
}

pub fn disasm_swp(instr: u32, _address: u32, _options: &DisasmOptions) -> ArmInstruction {
    let condition = Condition::from_bits(instr.get_bit_range(28..=31));
    ArmInstruction::Undefined { condition }
}

pub fn disasm_undefined(instr: u32, _address: u32, _options: &DisasmOptions) -> ArmInstruction {
    let condition = Condition::from_bits(instr.get_bit_range(28..=31));
    ArmInstruction::Undefined { condition }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Condition {
    Eq = 0,
    Ne,
    Cs,
    Cc,
    Mi,
    Pl,
    Vs,
    Vc,
    Hi,
    Ls,
    Ge,
    Lt,
    Gt,
    Le,
    Al,
    Nv,
}

impl Condition {
    fn from_bits(bits: u32) -> Self {
        match bits {
            0x0 => Self::Eq,
            0x1 => Self::Ne,
            0x2 => Self::Cs,
            0x3 => Self::Cc,
            0x4 => Self::Mi,
            0x5 => Self::Pl,
            0x6 => Self::Vs,
            0x7 => Self::Vc,
            0x8 => Self::Hi,
            0x9 => Self::Ls,
            0xA => Self::Ge,
            0xB => Self::Lt,
            0xC => Self::Gt,
            0xD => Self::Le,
            0xE => Self::Al,
            0xF => Self::Nv,
            _ => unreachable!(),
        }
    }

    fn as_str(self, uppercase: bool) -> &'static str {
        if uppercase {
            return UPPER_STR[self as usize];
        } else {
            return LOWER_STR[self as usize];
        }

        const LOWER_STR: [&str; 16] = [
            "eq", "ne", "cs", "cc", "mi", "pl", "vs", "vc", "hi", "ls", "ge", "lt", "gt", "le", "",
            "nv",
        ];
        const UPPER_STR: [&str; 16] = [
            "EQ", "NE", "CS", "CC", "MI", "PL", "VS", "VC", "HI", "LS", "GE", "LT", "GT", "LE", "",
            "NV",
        ];
    }
}
