pub fn disasm(instr: u32) -> ArmInstr {
    let cond = Condition::from((instr >> 28) & 0xF);

    if (instr & 0x0E000000 == 0x02000000)
        || (instr & 0x0E000010 == 0x00000000)
        || (instr & 0x0E000090 == 0x00000010)
    {
        ArmInstr::DataProc {
            cond,
            proc: DataProc::from((instr >> 21) & 0xF),
            s: (instr & 0x00100000) != 0,
            rd: Register::from((instr >> 12) & 0xF),
            rn: Register::from((instr >> 16) & 0xF),
            op2: if (instr & 0x02000000) == 0 {
                DataProcOperand2::from_register(instr)
            } else {
                DataProcOperand2::from_imm(instr)
            },
        }
    } else {
        ArmInstr::Undefined { cond, instr }
    }
}

#[derive(Debug, Clone)]
pub enum ArmInstr {
    DataProc {
        cond: Condition,
        proc: DataProc,
        s: bool,
        rd: Register,
        rn: Register,
        op2: DataProcOperand2,
    },

    Undefined {
        cond: Condition,
        instr: u32,
    },
}

impl ArmInstr {
    pub(crate) fn write_mnemonic(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArmInstr::Undefined { cond, .. } => f.write_fmt(format_args!("undef{cond}")),
            ArmInstr::DataProc { cond, proc, s, .. } => {
                if matches!(
                    proc,
                    DataProc::Tst | DataProc::Teq | DataProc::Cmp | DataProc::Cmn
                ) {
                    f.write_fmt(format_args!("{proc}{cond}"))
                } else {
                    f.write_fmt(format_args!(
                        "{proc}{cond}{s}",
                        s = if *s { "s" } else { "" }
                    ))
                }
            }
        }
    }

    pub(crate) fn write_arguments(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArmInstr::Undefined { instr, .. } => f.write_fmt(format_args!("0x{:08x}", instr)),
            ArmInstr::DataProc {
                proc, rd, rn, op2, ..
            } => match proc {
                DataProc::Mov | DataProc::Mvn => f.write_fmt(format_args!("{rd}, {op2}")),
                DataProc::Tst | DataProc::Teq | DataProc::Cmp | DataProc::Cmn => {
                    f.write_fmt(format_args!("{rn}, {op2}"))
                }
                _ => f.write_fmt(format_args!("{rd}, {rn}, {op2}")),
            },
        }
    }

    pub(crate) fn write_comment(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArmInstr::Undefined { .. } => Ok(()),
            ArmInstr::DataProc { .. } => Ok(()),
        }
    }

    pub fn mnemonic(&self) -> crate::Mnemonic<'_, Self> {
        crate::Mnemonic(self)
    }

    pub fn arguments(&self) -> crate::Arguments<'_, Self> {
        crate::Arguments(self)
    }

    pub fn comment(&self) -> crate::Comment<'_, Self> {
        crate::Comment(self)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum DataProcOperand2 {
    Immediate(u32),
    Register(Register, Option<Shift>),
}

impl DataProcOperand2 {
    pub fn from_register(val: u32) -> Self {
        let rm = Register::from(val & 0xF);
        let shift = Shift::from(val >> 4);

        if matches!(shift, Shift::Imm(ImmShift::Lsl(0))) {
            DataProcOperand2::Register(rm, None)
        } else {
            DataProcOperand2::Register(rm, Some(shift))
        }
    }

    pub fn from_imm(val: u32) -> Self {
        let imm = val & 0xFF;
        let rot = (val >> 8) & 0xF;
        DataProcOperand2::Immediate(imm.rotate_right(rot * 2))
    }
}

impl std::fmt::Display for DataProcOperand2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataProcOperand2::Immediate(imm) => f.write_fmt(format_args!("#{}", imm)),
            DataProcOperand2::Register(reg, shift) => match shift {
                Some(shift) => f.write_fmt(format_args!("{}, {}", reg, shift)),
                None => f.write_fmt(format_args!("{}", reg)),
            },
        }
    }
}

#[derive(Debug, Copy, Clone)]
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
            Shift::Imm(imm) => f.write_fmt(format_args!("{}", imm)),
            Shift::Reg(reg) => f.write_fmt(format_args!("{}", reg)),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ImmShift {
    Lsl(u8),
    Lsr(u8),
    Asr(u8),
    Ror(u8),
    Rrx,
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
            ImmShift::Lsl(imm) => f.write_fmt(format_args!("lsl #{}", imm)),
            ImmShift::Lsr(imm) => f.write_fmt(format_args!("lsr #{}", imm)),
            ImmShift::Asr(imm) => f.write_fmt(format_args!("asr #{}", imm)),
            ImmShift::Ror(imm) => f.write_fmt(format_args!("ror #{}", imm)),
            ImmShift::Rrx => f.write_str("rrx"),
        }
    }
}

#[derive(Debug, Copy, Clone)]
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
            RegShift::Lsl(r) => f.write_fmt(format_args!("lsl {}", r)),
            RegShift::Lsr(r) => f.write_fmt(format_args!("lsr {}", r)),
            RegShift::Asr(r) => f.write_fmt(format_args!("asr {}", r)),
            RegShift::Ror(r) => f.write_fmt(format_args!("ror {}", r)),
        }
    }
}

#[derive(Debug, Copy, Clone)]
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

#[derive(Debug, Copy, Clone)]
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

#[cfg(test)]
mod tests {
    use super::disasm;
    use arm_devkit::LinkerScriptWeakRef;
    use std::sync::RwLock;

    #[test]
    fn disasm_undef() {
        let dis = disasm(0xE777F777);
        assert_eq!("undef", dis.mnemonic().to_string());
        assert_eq!("0xe777f777", dis.arguments().to_string());
        assert_eq!("", dis.comment().to_string());
    }

    macro_rules! make_test {
        ($name:ident, $source:literal, $mnemonic:literal, $arguments:literal) => {
            #[test]
            fn $name() {
                let asm = assemble_one($source).unwrap();
                let dis = disasm(asm);
                assert_eq!($mnemonic, dis.mnemonic().to_string());
                assert_eq!($arguments, dis.arguments().to_string());
                assert_eq!("", dis.comment().to_string());
            }
        };

        ($name:ident, $source:literal, $mnemonic:literal, $arguments:literal, $comment:literal) => {
            #[test]
            fn $name() {
                let asm = assemble_one($source).unwrap();
                let dis = disasm(asm);
                assert_eq!($mnemonic, dis.mnemonic().to_string());
                assert_eq!($arguments, dis.arguments().to_string());
                assert_eq!($comment, dis.comment().to_string());
            }
        };
    }

    macro_rules! make_tests {
        ($([$name:ident, $source:literal, $mnemonic:literal, $arguments:literal $(, $comment:literal)?]),+ $(,)?) => {
            $(make_test!($name, $source, $mnemonic, $arguments $(, $comment)?);)+
        };
    }

    // DATA PROCESSING INSTRUCTIONS
    #[rustfmt::skip]
    make_tests! {
        // AND
        [disasm_and_imm, "and r0, r1, #4", "and", "r0, r1, #4"],
        [disasm_ands_imm, "ands r0, r1, #4", "ands", "r0, r1, #4"],
        [disasm_and_reg, "and r0, r1, r2", "and", "r0, r1, r2"],
        [disasm_and_reg_lsl_imm, "and r0, r1, r2, lsl #4", "and", "r0, r1, r2, lsl #4"],
        [disasm_and_reg_lsl_reg, "and r0, r1, r2, lsl r4", "and", "r0, r1, r2, lsl r4"],
        [disasm_and_reg_lsr_imm, "and r0, r1, r2, lsr #4", "and", "r0, r1, r2, lsr #4"],
        [disasm_and_reg_lsr_imm_32, "and r0, r1, r2, lsr #32", "and", "r0, r1, r2, lsr #32"],
        [disasm_and_reg_lsr_reg, "and r0, r1, r2, lsr r4", "and", "r0, r1, r2, lsr r4"],
        [disasm_and_reg_asr_imm, "and r0, r1, r2, asr #4", "and", "r0, r1, r2, asr #4"],
        [disasm_and_reg_asr_imm_32, "and r0, r1, r2, asr #32", "and", "r0, r1, r2, asr #32"],
        [disasm_and_reg_asr_reg, "and r0, r1, r2, asr r4", "and", "r0, r1, r2, asr r4"],
        [disasm_and_reg_ror_imm, "and r0, r1, r2, ror #4", "and", "r0, r1, r2, ror #4"],
        [disasm_and_reg_ror_reg, "and r0, r1, r2, ror r4", "and", "r0, r1, r2, ror r4"],
        [disasm_and_reg_rrx, "and r0, r1, r2, rrx", "and", "r0, r1, r2, rrx"],

        // EOR
        [disasm_eor_imm, "eor r0, r1, #4", "eor", "r0, r1, #4"],
        [disasm_eors_imm, "eors r0, r1, #4", "eors", "r0, r1, #4"],
        [disasm_eor_reg, "eor r0, r1, r2", "eor", "r0, r1, r2"],
        [disasm_eor_reg_lsl_imm, "eor r0, r1, r2, lsl #4", "eor", "r0, r1, r2, lsl #4"],
        [disasm_eor_reg_lsl_reg, "eor r0, r1, r2, lsl r4", "eor", "r0, r1, r2, lsl r4"],
        [disasm_eor_reg_lsr_imm, "eor r0, r1, r2, lsr #4", "eor", "r0, r1, r2, lsr #4"],
        [disasm_eor_reg_lsr_imm_32, "eor r0, r1, r2, lsr #32", "eor", "r0, r1, r2, lsr #32"],
        [disasm_eor_reg_lsr_reg, "eor r0, r1, r2, lsr r4", "eor", "r0, r1, r2, lsr r4"],
        [disasm_eor_reg_asr_imm, "eor r0, r1, r2, asr #4", "eor", "r0, r1, r2, asr #4"],
        [disasm_eor_reg_asr_imm_32, "eor r0, r1, r2, asr #32", "eor", "r0, r1, r2, asr #32"],
        [disasm_eor_reg_asr_reg, "eor r0, r1, r2, asr r4", "eor", "r0, r1, r2, asr r4"],
        [disasm_eor_reg_ror_imm, "eor r0, r1, r2, ror #4", "eor", "r0, r1, r2, ror #4"],
        [disasm_eor_reg_ror_reg, "eor r0, r1, r2, ror r4", "eor", "r0, r1, r2, ror r4"],
        [disasm_eor_reg_rrx, "eor r0, r1, r2, rrx", "eor", "r0, r1, r2, rrx"],

        // SUB
        [disasm_sub_imm, "sub r0, r1, #4", "sub", "r0, r1, #4"],
        [disasm_subs_imm, "subs r0, r1, #4", "subs", "r0, r1, #4"],
        [disasm_sub_reg, "sub r0, r1, r2", "sub", "r0, r1, r2"],
        [disasm_sub_reg_lsl_imm, "sub r0, r1, r2, lsl #4", "sub", "r0, r1, r2, lsl #4"],
        [disasm_sub_reg_lsl_reg, "sub r0, r1, r2, lsl r4", "sub", "r0, r1, r2, lsl r4"],
        [disasm_sub_reg_lsr_imm, "sub r0, r1, r2, lsr #4", "sub", "r0, r1, r2, lsr #4"],
        [disasm_sub_reg_lsr_imm_32, "sub r0, r1, r2, lsr #32", "sub", "r0, r1, r2, lsr #32"],
        [disasm_sub_reg_lsr_reg, "sub r0, r1, r2, lsr r4", "sub", "r0, r1, r2, lsr r4"],
        [disasm_sub_reg_asr_imm, "sub r0, r1, r2, asr #4", "sub", "r0, r1, r2, asr #4"],
        [disasm_sub_reg_asr_imm_32, "sub r0, r1, r2, asr #32", "sub", "r0, r1, r2, asr #32"],
        [disasm_sub_reg_asr_reg, "sub r0, r1, r2, asr r4", "sub", "r0, r1, r2, asr r4"],
        [disasm_sub_reg_ror_imm, "sub r0, r1, r2, ror #4", "sub", "r0, r1, r2, ror #4"],
        [disasm_sub_reg_ror_reg, "sub r0, r1, r2, ror r4", "sub", "r0, r1, r2, ror r4"],
        [disasm_sub_reg_rrx, "sub r0, r1, r2, rrx", "sub", "r0, r1, r2, rrx"],

        // RSB
        [disasm_rsb_imm, "rsb r0, r1, #4", "rsb", "r0, r1, #4"],
        [disasm_rsbs_imm, "rsbs r0, r1, #4", "rsbs", "r0, r1, #4"],
        [disasm_rsb_reg, "rsb r0, r1, r2", "rsb", "r0, r1, r2"],
        [disasm_rsb_reg_lsl_imm, "rsb r0, r1, r2, lsl #4", "rsb", "r0, r1, r2, lsl #4"],
        [disasm_rsb_reg_lsl_reg, "rsb r0, r1, r2, lsl r4", "rsb", "r0, r1, r2, lsl r4"],
        [disasm_rsb_reg_lsr_imm, "rsb r0, r1, r2, lsr #4", "rsb", "r0, r1, r2, lsr #4"],
        [disasm_rsb_reg_lsr_imm_32, "rsb r0, r1, r2, lsr #32", "rsb", "r0, r1, r2, lsr #32"],
        [disasm_rsb_reg_lsr_reg, "rsb r0, r1, r2, lsr r4", "rsb", "r0, r1, r2, lsr r4"],
        [disasm_rsb_reg_asr_imm, "rsb r0, r1, r2, asr #4", "rsb", "r0, r1, r2, asr #4"],
        [disasm_rsb_reg_asr_imm_32, "rsb r0, r1, r2, asr #32", "rsb", "r0, r1, r2, asr #32"],
        [disasm_rsb_reg_asr_reg, "rsb r0, r1, r2, asr r4", "rsb", "r0, r1, r2, asr r4"],
        [disasm_rsb_reg_ror_imm, "rsb r0, r1, r2, ror #4", "rsb", "r0, r1, r2, ror #4"],
        [disasm_rsb_reg_ror_reg, "rsb r0, r1, r2, ror r4", "rsb", "r0, r1, r2, ror r4"],
        [disasm_rsb_reg_rrx, "rsb r0, r1, r2, rrx", "rsb", "r0, r1, r2, rrx"],

        // ADD
        [disasm_add_imm, "add r0, r1, #4", "add", "r0, r1, #4"],
        [disasm_adds_imm, "adds r0, r1, #4", "adds", "r0, r1, #4"],
        [disasm_add_reg, "add r0, r1, r2", "add", "r0, r1, r2"],
        [disasm_add_reg_lsl_imm, "add r0, r1, r2, lsl #4", "add", "r0, r1, r2, lsl #4"],
        [disasm_add_reg_lsl_reg, "add r0, r1, r2, lsl r4", "add", "r0, r1, r2, lsl r4"],
        [disasm_add_reg_lsr_imm, "add r0, r1, r2, lsr #4", "add", "r0, r1, r2, lsr #4"],
        [disasm_add_reg_lsr_imm_32, "add r0, r1, r2, lsr #32", "add", "r0, r1, r2, lsr #32"],
        [disasm_add_reg_lsr_reg, "add r0, r1, r2, lsr r4", "add", "r0, r1, r2, lsr r4"],
        [disasm_add_reg_asr_imm, "add r0, r1, r2, asr #4", "add", "r0, r1, r2, asr #4"],
        [disasm_add_reg_asr_imm_32, "add r0, r1, r2, asr #32", "add", "r0, r1, r2, asr #32"],
        [disasm_add_reg_asr_reg, "add r0, r1, r2, asr r4", "add", "r0, r1, r2, asr r4"],
        [disasm_add_reg_ror_imm, "add r0, r1, r2, ror #4", "add", "r0, r1, r2, ror #4"],
        [disasm_add_reg_ror_reg, "add r0, r1, r2, ror r4", "add", "r0, r1, r2, ror r4"],
        [disasm_add_reg_rrx, "add r0, r1, r2, rrx", "add", "r0, r1, r2, rrx"],

        // ADC
        [disasm_adc_imm, "adc r0, r1, #4", "adc", "r0, r1, #4"],
        [disasm_adcs_imm, "adcs r0, r1, #4", "adcs", "r0, r1, #4"],
        [disasm_adc_reg, "adc r0, r1, r2", "adc", "r0, r1, r2"],
        [disasm_adc_reg_lsl_imm, "adc r0, r1, r2, lsl #4", "adc", "r0, r1, r2, lsl #4"],
        [disasm_adc_reg_lsl_reg, "adc r0, r1, r2, lsl r4", "adc", "r0, r1, r2, lsl r4"],
        [disasm_adc_reg_lsr_imm, "adc r0, r1, r2, lsr #4", "adc", "r0, r1, r2, lsr #4"],
        [disasm_adc_reg_lsr_imm_32, "adc r0, r1, r2, lsr #32", "adc", "r0, r1, r2, lsr #32"],
        [disasm_adc_reg_lsr_reg, "adc r0, r1, r2, lsr r4", "adc", "r0, r1, r2, lsr r4"],
        [disasm_adc_reg_asr_imm, "adc r0, r1, r2, asr #4", "adc", "r0, r1, r2, asr #4"],
        [disasm_adc_reg_asr_imm_32, "adc r0, r1, r2, asr #32", "adc", "r0, r1, r2, asr #32"],
        [disasm_adc_reg_asr_reg, "adc r0, r1, r2, asr r4", "adc", "r0, r1, r2, asr r4"],
        [disasm_adc_reg_ror_imm, "adc r0, r1, r2, ror #4", "adc", "r0, r1, r2, ror #4"],
        [disasm_adc_reg_ror_reg, "adc r0, r1, r2, ror r4", "adc", "r0, r1, r2, ror r4"],
        [disasm_adc_reg_rrx, "adc r0, r1, r2, rrx", "adc", "r0, r1, r2, rrx"],

        // SBC
        [disasm_sbc_imm, "sbc r0, r1, #4", "sbc", "r0, r1, #4"],
        [disasm_sbcs_imm, "sbcs r0, r1, #4", "sbcs", "r0, r1, #4"],
        [disasm_sbc_reg, "sbc r0, r1, r2", "sbc", "r0, r1, r2"],
        [disasm_sbc_reg_lsl_imm, "sbc r0, r1, r2, lsl #4", "sbc", "r0, r1, r2, lsl #4"],
        [disasm_sbc_reg_lsl_reg, "sbc r0, r1, r2, lsl r4", "sbc", "r0, r1, r2, lsl r4"],
        [disasm_sbc_reg_lsr_imm, "sbc r0, r1, r2, lsr #4", "sbc", "r0, r1, r2, lsr #4"],
        [disasm_sbc_reg_lsr_imm_32, "sbc r0, r1, r2, lsr #32", "sbc", "r0, r1, r2, lsr #32"],
        [disasm_sbc_reg_lsr_reg, "sbc r0, r1, r2, lsr r4", "sbc", "r0, r1, r2, lsr r4"],
        [disasm_sbc_reg_asr_imm, "sbc r0, r1, r2, asr #4", "sbc", "r0, r1, r2, asr #4"],
        [disasm_sbc_reg_asr_imm_32, "sbc r0, r1, r2, asr #32", "sbc", "r0, r1, r2, asr #32"],
        [disasm_sbc_reg_asr_reg, "sbc r0, r1, r2, asr r4", "sbc", "r0, r1, r2, asr r4"],
        [disasm_sbc_reg_ror_imm, "sbc r0, r1, r2, ror #4", "sbc", "r0, r1, r2, ror #4"],
        [disasm_sbc_reg_ror_reg, "sbc r0, r1, r2, ror r4", "sbc", "r0, r1, r2, ror r4"],
        [disasm_sbc_reg_rrx, "sbc r0, r1, r2, rrx", "sbc", "r0, r1, r2, rrx"],

        // RSC
        [disasm_rsc_imm, "rsc r0, r1, #4", "rsc", "r0, r1, #4"],
        [disasm_rscs_imm, "rscs r0, r1, #4", "rscs", "r0, r1, #4"],
        [disasm_rsc_reg, "rsc r0, r1, r2", "rsc", "r0, r1, r2"],
        [disasm_rsc_reg_lsl_imm, "rsc r0, r1, r2, lsl #4", "rsc", "r0, r1, r2, lsl #4"],
        [disasm_rsc_reg_lsl_reg, "rsc r0, r1, r2, lsl r4", "rsc", "r0, r1, r2, lsl r4"],
        [disasm_rsc_reg_lsr_imm, "rsc r0, r1, r2, lsr #4", "rsc", "r0, r1, r2, lsr #4"],
        [disasm_rsc_reg_lsr_imm_32, "rsc r0, r1, r2, lsr #32", "rsc", "r0, r1, r2, lsr #32"],
        [disasm_rsc_reg_lsr_reg, "rsc r0, r1, r2, lsr r4", "rsc", "r0, r1, r2, lsr r4"],
        [disasm_rsc_reg_asr_imm, "rsc r0, r1, r2, asr #4", "rsc", "r0, r1, r2, asr #4"],
        [disasm_rsc_reg_asr_imm_32, "rsc r0, r1, r2, asr #32", "rsc", "r0, r1, r2, asr #32"],
        [disasm_rsc_reg_asr_reg, "rsc r0, r1, r2, asr r4", "rsc", "r0, r1, r2, asr r4"],
        [disasm_rsc_reg_ror_imm, "rsc r0, r1, r2, ror #4", "rsc", "r0, r1, r2, ror #4"],
        [disasm_rsc_reg_ror_reg, "rsc r0, r1, r2, ror r4", "rsc", "r0, r1, r2, ror r4"],
        [disasm_rsc_reg_rrx, "rsc r0, r1, r2, rrx", "rsc", "r0, r1, r2, rrx"],

        // ORR
        [disasm_orr_imm, "orr r0, r1, #4", "orr", "r0, r1, #4"],
        [disasm_orrs_imm, "orrs r0, r1, #4", "orrs", "r0, r1, #4"],
        [disasm_orr_reg, "orr r0, r1, r2", "orr", "r0, r1, r2"],
        [disasm_orr_reg_lsl_imm, "orr r0, r1, r2, lsl #4", "orr", "r0, r1, r2, lsl #4"],
        [disasm_orr_reg_lsl_reg, "orr r0, r1, r2, lsl r4", "orr", "r0, r1, r2, lsl r4"],
        [disasm_orr_reg_lsr_imm, "orr r0, r1, r2, lsr #4", "orr", "r0, r1, r2, lsr #4"],
        [disasm_orr_reg_lsr_imm_32, "orr r0, r1, r2, lsr #32", "orr", "r0, r1, r2, lsr #32"],
        [disasm_orr_reg_lsr_reg, "orr r0, r1, r2, lsr r4", "orr", "r0, r1, r2, lsr r4"],
        [disasm_orr_reg_asr_imm, "orr r0, r1, r2, asr #4", "orr", "r0, r1, r2, asr #4"],
        [disasm_orr_reg_asr_imm_32, "orr r0, r1, r2, asr #32", "orr", "r0, r1, r2, asr #32"],
        [disasm_orr_reg_asr_reg, "orr r0, r1, r2, asr r4", "orr", "r0, r1, r2, asr r4"],
        [disasm_orr_reg_ror_imm, "orr r0, r1, r2, ror #4", "orr", "r0, r1, r2, ror #4"],
        [disasm_orr_reg_ror_reg, "orr r0, r1, r2, ror r4", "orr", "r0, r1, r2, ror r4"],
        [disasm_orr_reg_rrx, "orr r0, r1, r2, rrx", "orr", "r0, r1, r2, rrx"],

        // BIC
        [disasm_bic_imm, "bic r0, r1, #4", "bic", "r0, r1, #4"],
        [disasm_bics_imm, "bics r0, r1, #4", "bics", "r0, r1, #4"],
        [disasm_bic_reg, "bic r0, r1, r2", "bic", "r0, r1, r2"],
        [disasm_bic_reg_lsl_imm, "bic r0, r1, r2, lsl #4", "bic", "r0, r1, r2, lsl #4"],
        [disasm_bic_reg_lsl_reg, "bic r0, r1, r2, lsl r4", "bic", "r0, r1, r2, lsl r4"],
        [disasm_bic_reg_lsr_imm, "bic r0, r1, r2, lsr #4", "bic", "r0, r1, r2, lsr #4"],
        [disasm_bic_reg_lsr_imm_32, "bic r0, r1, r2, lsr #32", "bic", "r0, r1, r2, lsr #32"],
        [disasm_bic_reg_lsr_reg, "bic r0, r1, r2, lsr r4", "bic", "r0, r1, r2, lsr r4"],
        [disasm_bic_reg_asr_imm, "bic r0, r1, r2, asr #4", "bic", "r0, r1, r2, asr #4"],
        [disasm_bic_reg_asr_imm_32, "bic r0, r1, r2, asr #32", "bic", "r0, r1, r2, asr #32"],
        [disasm_bic_reg_asr_reg, "bic r0, r1, r2, asr r4", "bic", "r0, r1, r2, asr r4"],
        [disasm_bic_reg_ror_imm, "bic r0, r1, r2, ror #4", "bic", "r0, r1, r2, ror #4"],
        [disasm_bic_reg_ror_reg, "bic r0, r1, r2, ror r4", "bic", "r0, r1, r2, ror r4"],
        [disasm_bic_reg_rrx, "bic r0, r1, r2, rrx", "bic", "r0, r1, r2, rrx"],

        // TST
        [disasm_tst_imm, "tst r1, #4", "tst", "r1, #4"],
        [disasm_tsts_imm, "tsts r1, #4", "tst", "r1, #4"],
        [disasm_tst_reg, "tst r1, r2", "tst", "r1, r2"],
        [disasm_tst_reg_lsl_imm, "tst r1, r2, lsl #4", "tst", "r1, r2, lsl #4"],
        [disasm_tst_reg_lsl_reg, "tst r1, r2, lsl r4", "tst", "r1, r2, lsl r4"],
        [disasm_tst_reg_lsr_imm, "tst r1, r2, lsr #4", "tst", "r1, r2, lsr #4"],
        [disasm_tst_reg_lsr_imm_32, "tst r1, r2, lsr #32", "tst", "r1, r2, lsr #32"],
        [disasm_tst_reg_lsr_reg, "tst r1, r2, lsr r4", "tst", "r1, r2, lsr r4"],
        [disasm_tst_reg_asr_imm, "tst r1, r2, asr #4", "tst", "r1, r2, asr #4"],
        [disasm_tst_reg_asr_imm_32, "tst r1, r2, asr #32", "tst", "r1, r2, asr #32"],
        [disasm_tst_reg_asr_reg, "tst r1, r2, asr r4", "tst", "r1, r2, asr r4"],
        [disasm_tst_reg_ror_imm, "tst r1, r2, ror #4", "tst", "r1, r2, ror #4"],
        [disasm_tst_reg_ror_reg, "tst r1, r2, ror r4", "tst", "r1, r2, ror r4"],
        [disasm_tst_reg_rrx, "tst r1, r2, rrx", "tst", "r1, r2, rrx"],

        // TEQ
        [disasm_teq_imm, "teq r1, #4", "teq", "r1, #4"],
        [disasm_teqs_imm, "teqs r1, #4", "teq", "r1, #4"],
        [disasm_teq_reg, "teq r1, r2", "teq", "r1, r2"],
        [disasm_teq_reg_lsl_imm, "teq r1, r2, lsl #4", "teq", "r1, r2, lsl #4"],
        [disasm_teq_reg_lsl_reg, "teq r1, r2, lsl r4", "teq", "r1, r2, lsl r4"],
        [disasm_teq_reg_lsr_imm, "teq r1, r2, lsr #4", "teq", "r1, r2, lsr #4"],
        [disasm_teq_reg_lsr_imm_32, "teq r1, r2, lsr #32", "teq", "r1, r2, lsr #32"],
        [disasm_teq_reg_lsr_reg, "teq r1, r2, lsr r4", "teq", "r1, r2, lsr r4"],
        [disasm_teq_reg_asr_imm, "teq r1, r2, asr #4", "teq", "r1, r2, asr #4"],
        [disasm_teq_reg_asr_imm_32, "teq r1, r2, asr #32", "teq", "r1, r2, asr #32"],
        [disasm_teq_reg_asr_reg, "teq r1, r2, asr r4", "teq", "r1, r2, asr r4"],
        [disasm_teq_reg_ror_imm, "teq r1, r2, ror #4", "teq", "r1, r2, ror #4"],
        [disasm_teq_reg_ror_reg, "teq r1, r2, ror r4", "teq", "r1, r2, ror r4"],
        [disasm_teq_reg_rrx, "teq r1, r2, rrx", "teq", "r1, r2, rrx"],

        // CMP
        [disasm_cmp_imm, "cmp r1, #4", "cmp", "r1, #4"],
        [disasm_cmps_imm, "cmps r1, #4", "cmp", "r1, #4"],
        [disasm_cmp_reg, "cmp r1, r2", "cmp", "r1, r2"],
        [disasm_cmp_reg_lsl_imm, "cmp r1, r2, lsl #4", "cmp", "r1, r2, lsl #4"],
        [disasm_cmp_reg_lsl_reg, "cmp r1, r2, lsl r4", "cmp", "r1, r2, lsl r4"],
        [disasm_cmp_reg_lsr_imm, "cmp r1, r2, lsr #4", "cmp", "r1, r2, lsr #4"],
        [disasm_cmp_reg_lsr_imm_32, "cmp r1, r2, lsr #32", "cmp", "r1, r2, lsr #32"],
        [disasm_cmp_reg_lsr_reg, "cmp r1, r2, lsr r4", "cmp", "r1, r2, lsr r4"],
        [disasm_cmp_reg_asr_imm, "cmp r1, r2, asr #4", "cmp", "r1, r2, asr #4"],
        [disasm_cmp_reg_asr_imm_32, "cmp r1, r2, asr #32", "cmp", "r1, r2, asr #32"],
        [disasm_cmp_reg_asr_reg, "cmp r1, r2, asr r4", "cmp", "r1, r2, asr r4"],
        [disasm_cmp_reg_ror_imm, "cmp r1, r2, ror #4", "cmp", "r1, r2, ror #4"],
        [disasm_cmp_reg_ror_reg, "cmp r1, r2, ror r4", "cmp", "r1, r2, ror r4"],
        [disasm_cmp_reg_rrx, "cmp r1, r2, rrx", "cmp", "r1, r2, rrx"],

        // CMN
        [disasm_cmn_imm, "cmn r1, #4", "cmn", "r1, #4"],
        [disasm_cmns_imm, "cmns r1, #4", "cmn", "r1, #4"],
        [disasm_cmn_reg, "cmn r1, r2", "cmn", "r1, r2"],
        [disasm_cmn_reg_lsl_imm, "cmn r1, r2, lsl #4", "cmn", "r1, r2, lsl #4"],
        [disasm_cmn_reg_lsl_reg, "cmn r1, r2, lsl r4", "cmn", "r1, r2, lsl r4"],
        [disasm_cmn_reg_lsr_imm, "cmn r1, r2, lsr #4", "cmn", "r1, r2, lsr #4"],
        [disasm_cmn_reg_lsr_imm_32, "cmn r1, r2, lsr #32", "cmn", "r1, r2, lsr #32"],
        [disasm_cmn_reg_lsr_reg, "cmn r1, r2, lsr r4", "cmn", "r1, r2, lsr r4"],
        [disasm_cmn_reg_asr_imm, "cmn r1, r2, asr #4", "cmn", "r1, r2, asr #4"],
        [disasm_cmn_reg_asr_imm_32, "cmn r1, r2, asr #32", "cmn", "r1, r2, asr #32"],
        [disasm_cmn_reg_asr_reg, "cmn r1, r2, asr r4", "cmn", "r1, r2, asr r4"],
        [disasm_cmn_reg_ror_imm, "cmn r1, r2, ror #4", "cmn", "r1, r2, ror #4"],
        [disasm_cmn_reg_ror_reg, "cmn r1, r2, ror r4", "cmn", "r1, r2, ror r4"],
        [disasm_cmn_reg_rrx, "cmn r1, r2, rrx", "cmn", "r1, r2, rrx"],

        // MOV
        [disasm_mov_imm, "mov r1, #4", "mov", "r1, #4"],
        [disasm_movs_imm, "movs r1, #4", "movs", "r1, #4"],
        [disasm_mov_reg, "mov r1, r2", "mov", "r1, r2"],
        [disasm_mov_reg_lsl_imm, "mov r1, r2, lsl #4", "mov", "r1, r2, lsl #4"],
        [disasm_mov_reg_lsl_reg, "mov r1, r2, lsl r4", "mov", "r1, r2, lsl r4"],
        [disasm_mov_reg_lsr_imm, "mov r1, r2, lsr #4", "mov", "r1, r2, lsr #4"],
        [disasm_mov_reg_lsr_imm_32, "mov r1, r2, lsr #32", "mov", "r1, r2, lsr #32"],
        [disasm_mov_reg_lsr_reg, "mov r1, r2, lsr r4", "mov", "r1, r2, lsr r4"],
        [disasm_mov_reg_asr_imm, "mov r1, r2, asr #4", "mov", "r1, r2, asr #4"],
        [disasm_mov_reg_asr_imm_32, "mov r1, r2, asr #32", "mov", "r1, r2, asr #32"],
        [disasm_mov_reg_asr_reg, "mov r1, r2, asr r4", "mov", "r1, r2, asr r4"],
        [disasm_mov_reg_ror_imm, "mov r1, r2, ror #4", "mov", "r1, r2, ror #4"],
        [disasm_mov_reg_ror_reg, "mov r1, r2, ror r4", "mov", "r1, r2, ror r4"],
        [disasm_mov_reg_rrx, "mov r1, r2, rrx", "mov", "r1, r2, rrx"],

        // MVN
        [disasm_mvn_imm, "mvn r1, #4", "mvn", "r1, #4"],
        [disasm_mvns_imm, "mvns r1, #4", "mvns", "r1, #4"],
        [disasm_mvn_reg, "mvn r1, r2", "mvn", "r1, r2"],
        [disasm_mvn_reg_lsl_imm, "mvn r1, r2, lsl #4", "mvn", "r1, r2, lsl #4"],
        [disasm_mvn_reg_lsl_reg, "mvn r1, r2, lsl r4", "mvn", "r1, r2, lsl r4"],
        [disasm_mvn_reg_lsr_imm, "mvn r1, r2, lsr #4", "mvn", "r1, r2, lsr #4"],
        [disasm_mvn_reg_lsr_imm_32, "mvn r1, r2, lsr #32", "mvn", "r1, r2, lsr #32"],
        [disasm_mvn_reg_lsr_reg, "mvn r1, r2, lsr r4", "mvn", "r1, r2, lsr r4"],
        [disasm_mvn_reg_asr_imm, "mvn r1, r2, asr #4", "mvn", "r1, r2, asr #4"],
        [disasm_mvn_reg_asr_imm_32, "mvn r1, r2, asr #32", "mvn", "r1, r2, asr #32"],
        [disasm_mvn_reg_asr_reg, "mvn r1, r2, asr r4", "mvn", "r1, r2, asr r4"],
        [disasm_mvn_reg_ror_imm, "mvn r1, r2, ror #4", "mvn", "r1, r2, ror #4"],
        [disasm_mvn_reg_ror_reg, "mvn r1, r2, ror r4", "mvn", "r1, r2, ror r4"],
        [disasm_mvn_reg_rrx, "mvn r1, r2, rrx", "mvn", "r1, r2, rrx"],
    }

    // CONDITION CODES
    #[rustfmt::skip]
    make_tests! {
        // AND / ANDS
        [disasm_and_eq, "andeq r0, r1, #4", "andeq", "r0, r1, #4"],
        [disasm_ands_eq, "andeqs r0, r1, #4", "andeqs", "r0, r1, #4"],
        [disasm_and_ne, "andne r0, r1, #4", "andne", "r0, r1, #4"],
        [disasm_ands_ne, "andnes r0, r1, #4", "andnes", "r0, r1, #4"],
        [disasm_and_cs, "andcs r0, r1, #4", "andcs", "r0, r1, #4"],
        [disasm_ands_cs, "andcss r0, r1, #4", "andcss", "r0, r1, #4"],
        [disasm_and_cc, "andcc r0, r1, #4", "andcc", "r0, r1, #4"],
        [disasm_ands_cc, "andccs r0, r1, #4", "andccs", "r0, r1, #4"],
        [disasm_and_mi, "andmi r0, r1, #4", "andmi", "r0, r1, #4"],
        [disasm_ands_mi, "andmis r0, r1, #4", "andmis", "r0, r1, #4"],
        [disasm_and_pl, "andpl r0, r1, #4", "andpl", "r0, r1, #4"],
        [disasm_ands_pl, "andpls r0, r1, #4", "andpls", "r0, r1, #4"],
        [disasm_and_vs, "andvs r0, r1, #4", "andvs", "r0, r1, #4"],
        [disasm_ands_vs, "andvss r0, r1, #4", "andvss", "r0, r1, #4"],
        [disasm_and_vc, "andvc r0, r1, #4", "andvc", "r0, r1, #4"],
        [disasm_ands_vc, "andvcs r0, r1, #4", "andvcs", "r0, r1, #4"],
        [disasm_and_hi, "andhi r0, r1, #4", "andhi", "r0, r1, #4"],
        [disasm_ands_hi, "andhis r0, r1, #4", "andhis", "r0, r1, #4"],
        [disasm_and_ls, "andls r0, r1, #4", "andls", "r0, r1, #4"],
        [disasm_ands_ls, "andlss r0, r1, #4", "andlss", "r0, r1, #4"],
        [disasm_and_ge, "andge r0, r1, #4", "andge", "r0, r1, #4"],
        [disasm_ands_ge, "andges r0, r1, #4", "andges", "r0, r1, #4"],
        [disasm_and_lt, "andlt r0, r1, #4", "andlt", "r0, r1, #4"],
        [disasm_ands_lt, "andlts r0, r1, #4", "andlts", "r0, r1, #4"],
        [disasm_and_gt, "andgt r0, r1, #4", "andgt", "r0, r1, #4"],
        [disasm_ands_gt, "andgts r0, r1, #4", "andgts", "r0, r1, #4"],
        [disasm_and_le, "andle r0, r1, #4", "andle", "r0, r1, #4"],
        [disasm_ands_le, "andles r0, r1, #4", "andles", "r0, r1, #4"],

        // TST / TSTS
        [disasm_tst_eq, "tsteq r1, #4", "tsteq", "r1, #4"],
        [disasm_tsts_eq, "tsteqs r1, #4", "tsteq", "r1, #4"],
        [disasm_tst_ne, "tstne r1, #4", "tstne", "r1, #4"],
        [disasm_tsts_ne, "tstnes r1, #4", "tstne", "r1, #4"],
        [disasm_tst_cs, "tstcs r1, #4", "tstcs", "r1, #4"],
        [disasm_tsts_cs, "tstcss r1, #4", "tstcs", "r1, #4"],
        [disasm_tst_cc, "tstcc r1, #4", "tstcc", "r1, #4"],
        [disasm_tsts_cc, "tstccs r1, #4", "tstcc", "r1, #4"],
        [disasm_tst_mi, "tstmi r1, #4", "tstmi", "r1, #4"],
        [disasm_tsts_mi, "tstmis r1, #4", "tstmi", "r1, #4"],
        [disasm_tst_pl, "tstpl r1, #4", "tstpl", "r1, #4"],
        [disasm_tsts_pl, "tstpls r1, #4", "tstpl", "r1, #4"],
        [disasm_tst_vs, "tstvs r1, #4", "tstvs", "r1, #4"],
        [disasm_tsts_vs, "tstvss r1, #4", "tstvs", "r1, #4"],
        [disasm_tst_vc, "tstvc r1, #4", "tstvc", "r1, #4"],
        [disasm_tsts_vc, "tstvcs r1, #4", "tstvc", "r1, #4"],
        [disasm_tst_hi, "tsthi r1, #4", "tsthi", "r1, #4"],
        [disasm_tsts_hi, "tsthis r1, #4", "tsthi", "r1, #4"],
        [disasm_tst_ls, "tstls r1, #4", "tstls", "r1, #4"],
        [disasm_tsts_ls, "tstlss r1, #4", "tstls", "r1, #4"],
        [disasm_tst_ge, "tstge r1, #4", "tstge", "r1, #4"],
        [disasm_tsts_ge, "tstges r1, #4", "tstge", "r1, #4"],
        [disasm_tst_lt, "tstlt r1, #4", "tstlt", "r1, #4"],
        [disasm_tsts_lt, "tstlts r1, #4", "tstlt", "r1, #4"],
        [disasm_tst_gt, "tstgt r1, #4", "tstgt", "r1, #4"],
        [disasm_tsts_gt, "tstgts r1, #4", "tstgt", "r1, #4"],
        [disasm_tst_le, "tstle r1, #4", "tstle", "r1, #4"],
        [disasm_tsts_le, "tstles r1, #4", "tstle", "r1, #4"],
    }

    // REGISTERS
    #[rustfmt::skip]
    make_tests! {
        [disasm_mov_r0, "mov r0, r0", "mov", "r0, r0"],
        [disasm_mov_r1, "mov r0, r1", "mov", "r0, r1"],
        [disasm_mov_r2, "mov r0, r2", "mov", "r0, r2"],
        [disasm_mov_r3, "mov r0, r3", "mov", "r0, r3"],
        [disasm_mov_r4, "mov r0, r4", "mov", "r0, r4"],
        [disasm_mov_r5, "mov r0, r5", "mov", "r0, r5"],
        [disasm_mov_r6, "mov r0, r6", "mov", "r0, r6"],
        [disasm_mov_r7, "mov r0, r7", "mov", "r0, r7"],
        [disasm_mov_r8, "mov r0, r8", "mov", "r0, r8"],
        [disasm_mov_r9, "mov r0, r9", "mov", "r0, r9"],
        [disasm_mov_r10, "mov r0, r10", "mov", "r0, r10"],
        [disasm_mov_r11, "mov r0, r11", "mov", "r0, r11"],
        [disasm_mov_r12, "mov r0, r12", "mov", "r0, r12"],
        [disasm_mov_r13, "mov r0, r13", "mov", "r0, sp"],
        [disasm_mov_r14, "mov r0, r14", "mov", "r0, lr"],
        [disasm_mov_r15, "mov r0, r15", "mov", "r0, pc"],
        [disasm_mov_sp, "mov r0, sp", "mov", "r0, sp"],
        [disasm_mov_lr, "mov r0, lr", "mov", "r0, lr"],
        [disasm_mov_pc, "mov r0, pc", "mov", "r0, pc"],
    }

    fn assemble_one(source: &str) -> std::io::Result<u32> {
        static LINKER_SCRIPT: RwLock<Option<LinkerScriptWeakRef>> = RwLock::new(None);

        let guard = LINKER_SCRIPT.read().unwrap();
        let maybe_linker_script = guard.as_ref().and_then(|ls| ls.upgrade());
        drop(guard);
        let linker_script = if let Some(linker_script) = maybe_linker_script {
            linker_script
        } else {
            let linker_script = arm_devkit::LinkerScript::new(arm_devkit::SIMPLE_LINKER_SCRIPT)?;
            LINKER_SCRIPT
                .write()
                .unwrap()
                .replace(linker_script.clone().weak());
            linker_script
        };

        let assembled = arm_devkit::arm::assemble(source, linker_script)?;
        assert!(assembled.len() >= 4);

        let instr = (assembled[0] as u32)
            | ((assembled[1] as u32) << 8)
            | ((assembled[2] as u32) << 16)
            | ((assembled[3] as u32) << 24);
        Ok(instr)
    }
}
