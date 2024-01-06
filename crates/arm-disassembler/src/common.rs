use std::fmt::Write as _;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Register {
    R0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
}

impl From<Register> for u32 {
    fn from(val: Register) -> Self {
        match val {
            Register::R0 => 0x0,
            Register::R1 => 0x1,
            Register::R2 => 0x2,
            Register::R3 => 0x3,
            Register::R4 => 0x4,
            Register::R5 => 0x5,
            Register::R6 => 0x6,
            Register::R7 => 0x7,
            Register::R8 => 0x8,
            Register::R9 => 0x9,
            Register::R10 => 0xA,
            Register::R11 => 0xB,
            Register::R12 => 0xC,
            Register::R13 => 0xD,
            Register::R14 => 0xE,
            Register::R15 => 0xF,
        }
    }
}

impl From<u32> for Register {
    fn from(val: u32) -> Self {
        match val {
            0x0 => Register::R0,
            0x1 => Register::R1,
            0x2 => Register::R2,
            0x3 => Register::R3,
            0x4 => Register::R4,
            0x5 => Register::R5,
            0x6 => Register::R6,
            0x7 => Register::R7,
            0x8 => Register::R8,
            0x9 => Register::R9,
            0xA => Register::R10,
            0xB => Register::R11,
            0xC => Register::R12,
            0xD => Register::R13,
            0xE => Register::R14,
            0xF => Register::R15,
            _ => unreachable!(),
        }
    }
}

impl From<u16> for Register {
    fn from(val: u16) -> Self {
        Register::from(val as u32)
    }
}

impl std::fmt::Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Register::R0 => f.pad("r0"),
            Register::R1 => f.pad("r1"),
            Register::R2 => f.pad("r2"),
            Register::R3 => f.pad("r3"),
            Register::R4 => f.pad("r4"),
            Register::R5 => f.pad("r5"),
            Register::R6 => f.pad("r6"),
            Register::R7 => f.pad("r7"),
            Register::R8 => f.pad("r8"),
            Register::R9 => f.pad("r9"),
            Register::R10 => f.pad("r10"),
            Register::R11 => f.pad("r11"),
            Register::R12 => f.pad("r12"),
            Register::R13 => f.pad("sp"),
            Register::R14 => f.pad("lr"),
            Register::R15 => f.pad("pc"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DataProc {
    And,
    Eor,
    Sub,
    Rsb,
    Add,
    Adc,
    Sbc,
    Rsc,
    Tst,
    Teq,
    Cmp,
    Cmn,
    Orr,
    Mov,
    Bic,
    Mvn,
}

impl From<u32> for DataProc {
    fn from(val: u32) -> Self {
        match val {
            0x0 => DataProc::And,
            0x1 => DataProc::Eor,
            0x2 => DataProc::Sub,
            0x3 => DataProc::Rsb,
            0x4 => DataProc::Add,
            0x5 => DataProc::Adc,
            0x6 => DataProc::Sbc,
            0x7 => DataProc::Rsc,
            0x8 => DataProc::Tst,
            0x9 => DataProc::Teq,
            0xA => DataProc::Cmp,
            0xB => DataProc::Cmn,
            0xC => DataProc::Orr,
            0xD => DataProc::Mov,
            0xE => DataProc::Bic,
            0xF => DataProc::Mvn,
            _ => unreachable!(),
        }
    }
}

impl std::fmt::Display for DataProc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataProc::And => f.pad("and"),
            DataProc::Eor => f.pad("eor"),
            DataProc::Sub => f.pad("sub"),
            DataProc::Rsb => f.pad("rsb"),
            DataProc::Add => f.pad("add"),
            DataProc::Adc => f.pad("adc"),
            DataProc::Sbc => f.pad("sbc"),
            DataProc::Rsc => f.pad("rsc"),
            DataProc::Tst => f.pad("tst"),
            DataProc::Teq => f.pad("teq"),
            DataProc::Cmp => f.pad("cmp"),
            DataProc::Cmn => f.pad("cmn"),
            DataProc::Orr => f.pad("orr"),
            DataProc::Mov => f.pad("mov"),
            DataProc::Bic => f.pad("bic"),
            DataProc::Mvn => f.pad("mvn"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RegisterOrImmediate {
    Immediate(u32),
    Register(Register),
    ShiftedRegister(Register, Shift),
}

impl RegisterOrImmediate {
    pub fn from_maybe_shifted_register(val: u32) -> Self {
        let rm = Register::from(val & 0xF);
        let shift = Shift::from(val >> 4);

        if matches!(shift, Shift::Imm(ImmShift::Lsl(0))) {
            RegisterOrImmediate::Register(rm)
        } else {
            RegisterOrImmediate::ShiftedRegister(rm, shift)
        }
    }

    pub fn from_rotated_imm(val: u32) -> Self {
        let imm = val & 0xFF;
        let rot = (val >> 8) & 0xF;
        RegisterOrImmediate::Immediate(imm.rotate_right(rot * 2))
    }
}

impl std::fmt::Display for RegisterOrImmediate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegisterOrImmediate::Immediate(imm) => write!(f, "#{imm}"),
            RegisterOrImmediate::Register(reg) => write!(f, "{}", reg),
            RegisterOrImmediate::ShiftedRegister(reg, shift) => write!(f, "{}, {}", reg, shift),
        }
    }
}

impl std::fmt::LowerHex for RegisterOrImmediate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegisterOrImmediate::Immediate(imm) => write!(f, "#0x{imm:x}"),
            _ => std::fmt::Display::fmt(self, f),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ShiftType {
    Lsl,
    Lsr,
    Asr,
    Ror,
    Rrx,
}

impl std::fmt::Display for ShiftType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShiftType::Lsl => f.pad("lsl"),
            ShiftType::Lsr => f.pad("lsr"),
            ShiftType::Asr => f.pad("asr"),
            ShiftType::Ror => f.pad("ror"),
            ShiftType::Rrx => f.pad("rrx"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Shift {
    Imm(ImmShift),
    Reg(RegShift),
}

impl From<u32> for Shift {
    fn from(val: u32) -> Self {
        if val & 0x01 == 0 {
            Shift::Imm(ImmShift::from(val))
        } else {
            Shift::Reg(RegShift::from(val))
        }
    }
}

impl std::fmt::Display for Shift {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Shift::Imm(imm) => write!(f, "{}", imm),
            Shift::Reg(reg) => write!(f, "{}", reg),
        }
    }
}

impl std::fmt::LowerHex for Shift {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Shift::Imm(imm) => write!(f, "#0x{imm:x}"),
            _ => std::fmt::Display::fmt(self, f),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ImmShift {
    Lsl(u8),
    Lsr(u8),
    Asr(u8),
    Ror(u8),
    Rrx,
}

impl ImmShift {
    pub fn mnemonic(&self) -> &'static str {
        match self {
            ImmShift::Lsl(_) => "lsl",
            ImmShift::Lsr(_) => "lsr",
            ImmShift::Asr(_) => "asr",
            ImmShift::Ror(_) => "ror",
            ImmShift::Rrx => "rrx",
        }
    }

    pub fn amount(&self) -> u8 {
        match self {
            ImmShift::Lsl(imm) => *imm,
            ImmShift::Lsr(imm) => *imm,
            ImmShift::Asr(imm) => *imm,
            ImmShift::Ror(imm) => *imm,
            ImmShift::Rrx => 1,
        }
    }
}

impl From<u32> for ImmShift {
    fn from(val: u32) -> Self {
        let imm = ((val >> 3) & 0x1F) as u8;
        match (val >> 1) & 0x3 {
            0x0 => ImmShift::Lsl(imm),
            0x1 => ImmShift::Lsr(if imm == 0 { 32 } else { imm }),
            0x2 => ImmShift::Asr(if imm == 0 { 32 } else { imm }),
            0x3 if imm == 0 => ImmShift::Rrx,
            0x3 => ImmShift::Ror(imm),
            _ => unreachable!(),
        }
    }
}

impl std::fmt::Display for ImmShift {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImmShift::Lsl(imm) => write!(f, "lsl #{imm}"),
            ImmShift::Lsr(imm) => write!(f, "lsr #{imm}"),
            ImmShift::Asr(imm) => write!(f, "asr #{imm}"),
            ImmShift::Ror(imm) => write!(f, "ror #{imm}"),
            ImmShift::Rrx => write!(f, "rrx"),
        }
    }
}

impl std::fmt::LowerHex for ImmShift {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImmShift::Lsl(imm) => write!(f, "lsl #0x{imm:x}"),
            ImmShift::Lsr(imm) => write!(f, "lsr #0x{imm:x}"),
            ImmShift::Asr(imm) => write!(f, "asr #0x{imm:x}"),
            ImmShift::Ror(imm) => write!(f, "ror #0x{imm:x}"),
            ImmShift::Rrx => write!(f, "rrx"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RegShift {
    Lsl(Register),
    Lsr(Register),
    Asr(Register),
    Ror(Register),
}

impl From<u32> for RegShift {
    fn from(val: u32) -> Self {
        let rs = Register::from((val >> 4) & 0xF);
        match (val >> 1) & 0x3 {
            0x0 => RegShift::Lsl(rs),
            0x1 => RegShift::Lsr(rs),
            0x2 => RegShift::Asr(rs),
            0x3 => RegShift::Ror(rs),
            _ => unreachable!(),
        }
    }
}

impl std::fmt::Display for RegShift {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegShift::Lsl(r) => write!(f, "lsl {}", r),
            RegShift::Lsr(r) => write!(f, "lsr {}", r),
            RegShift::Asr(r) => write!(f, "asr {}", r),
            RegShift::Ror(r) => write!(f, "ror {}", r),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DataTransferOp {
    Load,
    Store,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SDTDataType {
    Word,
    Byte,
    Halfword,
    SignedHalfword,
    SignedByte,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DataTransferIndexing {
    Pre,
    Post,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DataTransferDirection {
    Up,
    Down,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct RegisterList(u16);

impl RegisterList {
    pub fn set(&mut self, register: Register) {
        self.0 |= 1 << (u32::from(register));
    }
}

impl From<u16> for RegisterList {
    fn from(val: u16) -> Self {
        RegisterList(val)
    }
}

impl std::fmt::Display for RegisterList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('{')?;

        let mut start: Option<Register> = None;
        let mut end: Option<Register> = None;
        let mut not_first_write = false;
        let mut write_registers = |start: &mut Option<Register>,
                                   end: &mut Option<Register>,
                                   f: &mut std::fmt::Formatter<'_>|
         -> std::fmt::Result {
            let prefix = if not_first_write {
                ","
            } else {
                not_first_write = true;
                ""
            };
            match (*start, *end) {
                (Some(start), None) => write!(f, "{prefix}{start}")?,
                (Some(start), Some(end)) if start == end => write!(f, "{prefix}{start}")?,
                (Some(start), Some(end)) => write!(f, "{prefix}{start}-{end}")?,
                (None, None) | (None, Some(_)) => return Ok(()),
            }
            *start = None;
            *end = None;
            Ok(())
        };

        for register in 0u32..16 {
            let set = ((self.0 >> register) & 0x1) != 0;

            if set && start.is_some() {
                end = Some(Register::from(register));
            } else if set && start.is_none() {
                start = Some(Register::from(register));
            } else if !set && (start.is_some() || end.is_some()) {
                write_registers(&mut start, &mut end, f)?;
            }
        }
        write_registers(&mut start, &mut end, f)?;
        f.write_char('}')
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Condition {
    Eq,
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

impl From<u32> for Condition {
    fn from(val: u32) -> Self {
        match val {
            0x0 => Condition::Eq,
            0x1 => Condition::Ne,
            0x2 => Condition::Cs,
            0x3 => Condition::Cc,
            0x4 => Condition::Mi,
            0x5 => Condition::Pl,
            0x6 => Condition::Vs,
            0x7 => Condition::Vc,
            0x8 => Condition::Hi,
            0x9 => Condition::Ls,
            0xA => Condition::Ge,
            0xB => Condition::Lt,
            0xC => Condition::Gt,
            0xD => Condition::Le,
            0xE => Condition::Al,
            0xF => Condition::Nv,
            _ => unreachable!(),
        }
    }
}

impl std::fmt::Display for Condition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Condition::Eq => f.pad("eq"),
            Condition::Ne => f.pad("ne"),
            Condition::Cs => f.pad("cs"),
            Condition::Cc => f.pad("cc"),
            Condition::Mi => f.pad("mi"),
            Condition::Pl => f.pad("pl"),
            Condition::Vs => f.pad("vs"),
            Condition::Vc => f.pad("vc"),
            Condition::Hi => f.pad("hi"),
            Condition::Ls => f.pad("ls"),
            Condition::Ge => f.pad("ge"),
            Condition::Lt => f.pad("lt"),
            Condition::Gt => f.pad("gt"),
            Condition::Le => f.pad("le"),
            Condition::Al => Ok(()),
            Condition::Nv => f.pad("nv"),
        }
    }
}
