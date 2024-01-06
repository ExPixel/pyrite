use std::fmt::Write;

use util::bits::BitOps as _;

use crate::{
    common::{
        Condition, DataProc, DataTransferDirection, DataTransferIndexing, DataTransferOp, Register,
        RegisterList, RegisterOrImmediate, SDTDataType,
    },
    MemoryView,
};

type ArmDisasmFn = fn(u32, u32) -> ArmInstr;
#[rustfmt::skip]
const DISASM_TABLE: &[(u32, u32, ArmDisasmFn)] = &[
    (0x0FFFFFF0, 0x012FFF10, disasm_bx),
    (0x0FBF0FFF, 0x010F0000, disasm_mrs),
    (0x0FBFFFF0, 0x0129F000, disasm_msr_all),
    (0x0FBFFFF0, 0x0128F000, disasm_msr_flg_reg),
    (0x0FBFF000, 0x0328F000, disasm_msr_flg_imm),
    (0x0FC000F0, 0x00000090, disasm_mul_and_mla),
    (0x0F8000F0, 0x00800090, disasm_mul_and_mla_long),
    (0x0E000000, 0x04000000, disasm_single_data_transfer), // single data transfer immediate offset
    (0x0E000010, 0x06000000, disasm_single_data_transfer), // single data transfer offset shift by imm
    (0x0FB00FF0, 0x01000090, disasm_single_data_swap),
    (0x0E400F90, 0x00000090, disasm_signed_and_halfword_data_transfer),
    (0x0F000000, 0x0F000000, disasm_software_interrupt),
    (0x0E000000, 0x08000000, disasm_block_data_transfer),
    (0x0E000000, 0x0A000000, disasm_b_and_bl),
    (0x0E000000, 0x02000000, disasm_dataproc), // dataproc immediate op2
    (0x0E000010, 0x00000000, disasm_dataproc), // dataproc op2 shift by imm
    (0x0E000090, 0x00000010, disasm_dataproc), // dataproc op2 shift by reg
];

pub fn disasm(instr: u32, address: u32) -> ArmInstr {
    for &(mask, check, disasm_fn) in DISASM_TABLE {
        if instr & mask == check {
            #[cfg(test)]
            {
                println!("match; address=0x{address:08x}; instr=0x{instr:08x}; mask=0x{mask:08x}; check=0x{check:08x}");
            }

            return disasm_fn(instr, address);
        }
    }

    let cond = Condition::from(instr.get_bit_range(28..=31));
    ArmInstr::Undefined { cond, instr }
}

pub fn disasm_bx(instr: u32, _address: u32) -> ArmInstr {
    let cond = Condition::from(instr.get_bit_range(28..=31));
    ArmInstr::BranchAndExchange {
        cond,
        rn: Register::from(instr & 0xF),
    }
}

pub fn disasm_b_and_bl(instr: u32, address: u32) -> ArmInstr {
    let cond = Condition::from(instr.get_bit_range(28..=31));
    let pc = address.wrapping_add(8);
    let offset = (instr & 0xFFFFFF).sign_extend(24).wrapping_shl(2);
    let target = pc.wrapping_add(offset);
    let link = instr.get_bit(24);
    ArmInstr::Branch { cond, target, link }
}

pub fn disasm_dataproc(instr: u32, _address: u32) -> ArmInstr {
    let cond = Condition::from(instr.get_bit_range(28..=31));
    ArmInstr::DataProc {
        cond,
        proc: DataProc::from((instr >> 21) & 0xF),
        s: (instr & 0x00100000) != 0,
        rd: Register::from((instr >> 12) & 0xF),
        rn: Register::from((instr >> 16) & 0xF),
        op2: if (instr & 0x02000000) == 0 {
            RegisterOrImmediate::from_maybe_shifted_register(instr)
        } else {
            RegisterOrImmediate::from_rotated_imm(instr)
        },
    }
}

pub fn disasm_mrs(instr: u32, _address: u32) -> ArmInstr {
    let cond = Condition::from(instr.get_bit_range(28..=31));
    let rd = Register::from(instr.get_bit_range(12..=15));
    let src = if instr.get_bit(22) {
        Psr::Spsr(false)
    } else {
        Psr::Cpsr(false)
    };
    ArmInstr::PsrToRegister { cond, rd, src }
}

pub fn disasm_msr_all(instr: u32, _address: u32) -> ArmInstr {
    let cond = Condition::from(instr.get_bit_range(28..=31));
    let dst = if instr.get_bit(22) {
        Psr::Spsr(false)
    } else {
        Psr::Cpsr(false)
    };
    let rm = Register::from(instr.get_bit_range(0..=3));
    let src = RegisterOrImmediate::Register(rm);
    ArmInstr::RegisterToPsr { cond, dst, src }
}

pub fn disasm_msr_flg_reg(instr: u32, _address: u32) -> ArmInstr {
    let cond = Condition::from(instr.get_bit_range(28..=31));
    let dst = if instr.get_bit(22) {
        Psr::Spsr(true)
    } else {
        Psr::Cpsr(true)
    };
    let rm = Register::from(instr.get_bit_range(0..=3));
    let src = RegisterOrImmediate::Register(rm);
    ArmInstr::RegisterToPsr { cond, dst, src }
}

pub fn disasm_msr_flg_imm(instr: u32, _address: u32) -> ArmInstr {
    let cond = Condition::from(instr.get_bit_range(28..=31));
    let dst = if instr.get_bit(22) {
        Psr::Spsr(true)
    } else {
        Psr::Cpsr(true)
    };

    let imm = instr.get_bit_range(0..=7);
    let rot = instr.get_bit_range(8..=11);
    let src = RegisterOrImmediate::Immediate(imm.rotate_right(rot * 2));
    ArmInstr::RegisterToPsr { cond, dst, src }
}

pub fn disasm_mul_and_mla(instr: u32, _address: u32) -> ArmInstr {
    let cond = Condition::from(instr.get_bit_range(28..=31));
    let a = instr.get_bit(21);
    let s = instr.get_bit(20);
    let rd = Register::from(instr.get_bit_range(16..=19));
    let rn = Register::from(instr.get_bit_range(12..=15));
    let rs = Register::from(instr.get_bit_range(8..=11));
    let rm = Register::from(instr.get_bit_range(0..=3));

    ArmInstr::Multiply {
        cond,
        a,
        s,
        rd,
        rn,
        rs,
        rm,
    }
}

pub fn disasm_mul_and_mla_long(instr: u32, _address: u32) -> ArmInstr {
    let cond = Condition::from(instr.get_bit_range(28..=31));
    let u = instr.get_bit(22);
    let a = instr.get_bit(21);
    let s = instr.get_bit(20);
    let rd_hi = Register::from(instr.get_bit_range(16..=19));
    let rd_lo = Register::from(instr.get_bit_range(12..=15));
    let rs = Register::from(instr.get_bit_range(8..=11));
    let rm = Register::from(instr.get_bit_range(0..=3));

    ArmInstr::MultiplyLong {
        cond,
        u,
        a,
        s,
        rd_hi,
        rd_lo,
        rs,
        rm,
    }
}

pub fn disasm_single_data_transfer(instr: u32, _address: u32) -> ArmInstr {
    let cond = Condition::from(instr.get_bit_range(28..=31));
    ArmInstr::SingleDataTransfer {
        cond,
        op: if instr.get_bit(20) {
            DataTransferOp::Load
        } else {
            DataTransferOp::Store
        },
        data_type: if instr.get_bit(22) {
            SDTDataType::Byte
        } else {
            SDTDataType::Word
        },
        indexing: if instr.get_bit(24) {
            DataTransferIndexing::Pre
        } else {
            DataTransferIndexing::Post
        },
        direction: if instr.get_bit(23) {
            DataTransferDirection::Up
        } else {
            DataTransferDirection::Down
        },
        writeback: instr.get_bit(21),
        rn: Register::from(instr.get_bit_range(16..=19)),
        rd: Register::from(instr.get_bit_range(12..=15)),
        offset: if instr.get_bit(25) {
            RegisterOrImmediate::from_maybe_shifted_register(instr)
        } else {
            RegisterOrImmediate::Immediate(instr.get_bit_range(0..=11))
        },
    }
}

pub fn disasm_signed_and_halfword_data_transfer(instr: u32, _address: u32) -> ArmInstr {
    let cond = Condition::from(instr.get_bit_range(28..=31));
    ArmInstr::SingleDataTransfer {
        cond,
        op: if instr.get_bit(20) {
            DataTransferOp::Load
        } else {
            DataTransferOp::Store
        },
        data_type: match instr.get_bit_range(5..=6) {
            0b00 => unreachable!("SWP not LDRH/STRH/LDRSB/LDRSH"),
            0b01 => SDTDataType::Halfword,
            0b10 => SDTDataType::SignedByte,
            0b11 => SDTDataType::SignedHalfword,
            _ => unreachable!(),
        },
        indexing: if instr.get_bit(24) {
            DataTransferIndexing::Pre
        } else {
            DataTransferIndexing::Post
        },
        direction: if instr.get_bit(23) {
            DataTransferDirection::Up
        } else {
            DataTransferDirection::Down
        },
        writeback: instr.get_bit(21),
        rn: Register::from(instr.get_bit_range(16..=19)),
        rd: Register::from(instr.get_bit_range(12..=15)),
        offset: RegisterOrImmediate::Register(Register::from(instr.get_bit_range(0..=3))),
    }
}

pub fn disasm_single_data_swap(instr: u32, _address: u32) -> ArmInstr {
    let cond = Condition::from(instr.get_bit_range(28..=31));
    ArmInstr::SingleDataSwap {
        cond,
        b: instr.get_bit(22),
        rn: Register::from(instr.get_bit_range(16..=19)),
        rd: Register::from(instr.get_bit_range(12..=15)),
        rm: Register::from(instr.get_bit_range(0..=3)),
    }
}

pub fn disasm_block_data_transfer(instr: u32, _address: u32) -> ArmInstr {
    let cond = Condition::from(instr.get_bit_range(28..=31));
    ArmInstr::BlockDataTransfer {
        cond,
        op: if instr.get_bit(20) {
            DataTransferOp::Load
        } else {
            DataTransferOp::Store
        },
        direction: if instr.get_bit(23) {
            DataTransferDirection::Up
        } else {
            DataTransferDirection::Down
        },
        indexing: if instr.get_bit(24) {
            DataTransferIndexing::Pre
        } else {
            DataTransferIndexing::Post
        },
        w: instr.get_bit(21),
        s: instr.get_bit(22),
        rn: Register::from(instr.get_bit_range(16..=19)),
        registers: RegisterList::from(instr as u16),
    }
}

pub fn disasm_software_interrupt(instr: u32, _address: u32) -> ArmInstr {
    let cond = Condition::from(instr.get_bit_range(28..=31));
    ArmInstr::SoftwareInterrupt {
        cond,
        comment: instr.get_bit_range(0..=23),
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
        op2: RegisterOrImmediate,
    },

    BranchAndExchange {
        cond: Condition,
        rn: Register,
    },

    Branch {
        cond: Condition,
        target: u32,
        link: bool,
    },

    PsrToRegister {
        cond: Condition,
        rd: Register,
        src: Psr,
    },

    RegisterToPsr {
        cond: Condition,
        dst: Psr,
        src: RegisterOrImmediate,
    },

    Multiply {
        cond: Condition,
        a: bool,
        s: bool,
        rd: Register,
        rn: Register,
        rs: Register,
        rm: Register,
    },

    MultiplyLong {
        cond: Condition,
        u: bool,
        a: bool,
        s: bool,
        rd_hi: Register,
        rd_lo: Register,
        rs: Register,
        rm: Register,
    },

    SingleDataTransfer {
        cond: Condition,
        op: DataTransferOp,
        data_type: SDTDataType,
        direction: DataTransferDirection,
        indexing: DataTransferIndexing,
        writeback: bool,
        rn: Register,
        rd: Register,
        offset: RegisterOrImmediate,
    },

    SingleDataSwap {
        cond: Condition,
        b: bool,
        rn: Register,
        rd: Register,
        rm: Register,
    },

    BlockDataTransfer {
        cond: Condition,
        op: DataTransferOp,
        direction: DataTransferDirection,
        indexing: DataTransferIndexing,
        w: bool,
        s: bool,
        rn: Register,
        registers: RegisterList,
    },

    SoftwareInterrupt {
        cond: Condition,
        comment: u32,
    },

    Undefined {
        cond: Condition,
        instr: u32,
    },
}

impl ArmInstr {
    pub(crate) fn write_mnemonic<W: Write>(&self, mut f: W) -> std::fmt::Result {
        match self {
            ArmInstr::Undefined { cond, .. } => write!(f, "undef{cond}"),
            ArmInstr::DataProc { cond, proc, s, .. } => {
                if matches!(
                    proc,
                    DataProc::Tst | DataProc::Teq | DataProc::Cmp | DataProc::Cmn
                ) {
                    write!(f, "{proc}{cond}")
                } else {
                    write!(f, "{proc}{cond}{s}", s = if *s { "s" } else { "" })
                }
            }
            ArmInstr::Multiply { cond, a, s, .. } => {
                let proc = if *a { "mla" } else { "mul" };
                write!(f, "{proc}{cond}{s}", s = if *s { "s" } else { "" })
            }
            ArmInstr::MultiplyLong { cond, u, a, s, .. } => {
                let proc = if *a { "mlal" } else { "mull" };
                write!(
                    f,
                    "{u}{proc}{cond}{s}",
                    u = if *u { "s" } else { "u" }, // xD
                    s = if *s { "s" } else { "" }
                )
            }
            ArmInstr::BranchAndExchange { cond, .. } => write!(f, "bx{cond}"),
            ArmInstr::Branch { cond, link, .. } => {
                write!(f, "b{cond}{link}", link = if *link { "l" } else { "" })
            }
            ArmInstr::PsrToRegister { cond, .. } => write!(f, "mrs{cond}"),
            ArmInstr::RegisterToPsr { cond, .. } => write!(f, "msr{cond}"),
            ArmInstr::SingleDataTransfer {
                cond,
                op,
                indexing,
                writeback,
                data_type,
                ..
            } => {
                let proc = match op {
                    DataTransferOp::Load => "ldr",
                    DataTransferOp::Store => "str",
                };
                let dt = match data_type {
                    SDTDataType::Byte => "b",
                    SDTDataType::Halfword => "h",
                    SDTDataType::SignedByte => "sb",
                    SDTDataType::SignedHalfword => "sh",
                    SDTDataType::Word => "",
                };
                let t = if matches!(data_type, SDTDataType::Word | SDTDataType::Byte)
                    && *indexing == DataTransferIndexing::Post
                    && *writeback
                {
                    "t"
                } else {
                    ""
                };
                write!(f, "{proc}{cond}{dt}{t}")
            }
            ArmInstr::SingleDataSwap { cond, b, .. } => {
                let b = if *b { "b" } else { "" };
                write!(f, "swp{cond}{b}")
            }
            ArmInstr::BlockDataTransfer {
                cond,
                op,
                direction,
                indexing,
                ..
            } => {
                let proc = match (op, direction, indexing) {
                    (
                        DataTransferOp::Load,
                        DataTransferDirection::Up,
                        DataTransferIndexing::Pre,
                    ) => "ldmib",
                    (
                        DataTransferOp::Load,
                        DataTransferDirection::Up,
                        DataTransferIndexing::Post,
                    ) => "ldmia",
                    (
                        DataTransferOp::Load,
                        DataTransferDirection::Down,
                        DataTransferIndexing::Pre,
                    ) => "ldmdb",
                    (
                        DataTransferOp::Load,
                        DataTransferDirection::Down,
                        DataTransferIndexing::Post,
                    ) => "ldmda",
                    (
                        DataTransferOp::Store,
                        DataTransferDirection::Up,
                        DataTransferIndexing::Pre,
                    ) => "stmib",
                    (
                        DataTransferOp::Store,
                        DataTransferDirection::Up,
                        DataTransferIndexing::Post,
                    ) => "stmia",
                    (
                        DataTransferOp::Store,
                        DataTransferDirection::Down,
                        DataTransferIndexing::Pre,
                    ) => "stmdb",
                    (
                        DataTransferOp::Store,
                        DataTransferDirection::Down,
                        DataTransferIndexing::Post,
                    ) => "stmda",
                };
                write!(f, "{proc}{cond}")
            }
            ArmInstr::SoftwareInterrupt { cond, .. } => write!(f, "swi{cond}"),
        }
    }

    pub(crate) fn write_arguments<W: Write>(&self, mut f: W) -> std::fmt::Result {
        match self {
            ArmInstr::Undefined { instr, .. } => write!(f, "0x{:08x}", instr),
            ArmInstr::DataProc {
                proc, rd, rn, op2, ..
            } => match proc {
                DataProc::Mov | DataProc::Mvn => write!(f, "{rd}, {op2:x}"),
                DataProc::Tst | DataProc::Teq | DataProc::Cmp | DataProc::Cmn => {
                    write!(f, "{rn}, {op2:x}")
                }
                _ => write!(f, "{rd}, {rn}, {op2:x}"),
            },
            ArmInstr::Multiply {
                rd, rn, rs, rm, a, ..
            } => {
                if *a {
                    write!(f, "{rd}, {rm}, {rs}, {rn}")
                } else {
                    write!(f, "{rd}, {rm}, {rs}")
                }
            }
            ArmInstr::MultiplyLong {
                rd_hi,
                rd_lo,
                rs,
                rm,
                ..
            } => {
                write!(f, "{rd_lo}, {rd_hi}, {rm}, {rs}")
            }
            ArmInstr::BranchAndExchange { rn, .. } => write!(f, "{rn}"),
            ArmInstr::Branch { target, .. } => write!(f, "0x{:08x}", target),
            ArmInstr::PsrToRegister { rd, src, .. } => write!(f, "{rd}, {src}"),
            ArmInstr::RegisterToPsr { dst, src, .. } => write!(f, "{dst}, {src:x}"),
            ArmInstr::SingleDataTransfer {
                rd,
                rn,
                indexing,
                writeback,
                offset,
                direction,
                ..
            } => match indexing {
                DataTransferIndexing::Pre => {
                    let w = if *writeback { "!" } else { "" };
                    let u = if *direction == DataTransferDirection::Down {
                        "-"
                    } else {
                        ""
                    };
                    write!(f, "{rd}, [{rn}, {u}{offset:x}]{w}")
                }
                DataTransferIndexing::Post => {
                    let u = if *direction == DataTransferDirection::Down {
                        "-"
                    } else {
                        ""
                    };
                    write!(f, "{rd}, [{rn}], {u}{offset:x}")
                }
            },
            ArmInstr::SingleDataSwap { rn, rd, rm, .. } => {
                write!(f, "{rd}, {rm}, [{rn}]")
            }
            ArmInstr::BlockDataTransfer {
                w,
                s,
                rn,
                registers,
                ..
            } => {
                let w = if *w { "!" } else { "" };
                let s = if *s { "^" } else { "" };
                write!(f, "{rn}{w}, {registers}{s}")
            }
            ArmInstr::SoftwareInterrupt { comment, .. } => write!(f, "#0x{:06x}", comment),
        }
    }

    pub(crate) fn write_comment<W: Write>(
        &self,
        mut f: W,
        addr: u32,
        m: Option<&dyn MemoryView>,
    ) -> std::fmt::Result {
        match *self {
            ArmInstr::DataProc {
                op2: RegisterOrImmediate::Immediate(imm),
                ..
            } => {
                let signed_imm = imm as i32;
                write!(f, "rhs = {signed_imm}")
            }

            ArmInstr::SingleDataTransfer {
                op: DataTransferOp::Load,
                data_type,
                direction,
                indexing,
                rn: Register::R15,
                rd,
                offset: RegisterOrImmediate::Immediate(offset),
                ..
            } => {
                let pc = addr.wrapping_add(8);
                let data_addr = if indexing == DataTransferIndexing::Pre {
                    if direction == DataTransferDirection::Down {
                        pc.wrapping_sub(offset)
                    } else {
                        pc.wrapping_add(offset)
                    }
                } else {
                    pc
                };

                if let Some(m) = m {
                    match data_type {
                        SDTDataType::Word => {
                            let data = m
                                .view32(data_addr & !0x03)
                                .rotate_right(8 * (data_addr % 4));
                            write!(f, "{rd} = 0x{data:08x}")
                        }
                        SDTDataType::Byte => {
                            let data = m.view8(data_addr);
                            write!(f, "{rd} = 0x{data:02x}")
                        }
                        SDTDataType::Halfword => {
                            let data = m.view16(data_addr & !0x1);
                            write!(f, "{rd} = 0x{data:04x}")
                        }
                        SDTDataType::SignedHalfword => {
                            let data = m.view16(data_addr & !0x1) as i16;
                            write!(f, "{rd} = 0x{data:04x}")
                        }
                        SDTDataType::SignedByte => {
                            let data = m.view8(data_addr) as i8;
                            write!(f, "{rd} = 0x{data:02x}")
                        }
                    }
                } else {
                    write!(f, "{rd} = [0x{data_addr:08x}]")
                }
            }

            _ => Ok(()),
        }
    }

    pub fn mnemonic(&self) -> crate::Mnemonic<'_, Self> {
        crate::Mnemonic(self)
    }

    pub fn arguments(&self) -> crate::Arguments<'_, '_, Self> {
        crate::Arguments(self, 0, None)
    }

    pub fn comment<'s>(
        &'s self,
        addr: u32,
        m: Option<&'s dyn MemoryView>,
    ) -> crate::Comment<'s, 's, Self> {
        crate::Comment(self, addr, m)
    }

    pub fn condition(&self) -> Condition {
        match self {
            ArmInstr::Undefined { cond, .. } => *cond,
            ArmInstr::DataProc { cond, .. } => *cond,
            ArmInstr::Multiply { cond, .. } => *cond,
            ArmInstr::MultiplyLong { cond, .. } => *cond,
            ArmInstr::BranchAndExchange { cond, .. } => *cond,
            ArmInstr::Branch { cond, .. } => *cond,
            ArmInstr::PsrToRegister { cond, .. } => *cond,
            ArmInstr::RegisterToPsr { cond, .. } => *cond,
            ArmInstr::SingleDataTransfer { cond, .. } => *cond,
            ArmInstr::SingleDataSwap { cond, .. } => *cond,
            ArmInstr::BlockDataTransfer { cond, .. } => *cond,
            ArmInstr::SoftwareInterrupt { cond, .. } => *cond,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Psr {
    Cpsr(/* flags only */ bool),
    Spsr(/* flags only */ bool),
}

impl std::fmt::Display for Psr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Psr::Cpsr(flags_only) => write!(f, "cpsr{}", if *flags_only { "_flg" } else { "_all" }),
            Psr::Spsr(flags_only) => write!(f, "spsr{}", if *flags_only { "_flg" } else { "_all" }),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::arm::Condition;

    use super::disasm;
    use arm_devkit::LinkerScriptWeakRef;
    use std::sync::RwLock;
    use util::bits::BitOps as _;

    #[test]
    fn disasm_undef() {
        // UNDEFINED RANGE:
        // XXXX011XXXXXXXXXXXXXXXXXXXX1XXXX
        let rand = util::wyhash::WyHash::new(0x8a88c0726f22dadd);
        for bits in rand.take(4096) {
            let bits = bits as u32;
            let instr = (bits & 0xF1FFFFEF) | 0x06000010;
            let cond = Condition::from(instr.get_bit_range(28..=31));
            let dis = disasm(instr, 0x0);
            assert_eq!(format!("undef{cond}"), dis.mnemonic().to_string());
            assert_eq!(format!("0x{instr:08x}"), dis.arguments().to_string());
            assert_eq!("", dis.comment(0, None).to_string());
        }
    }

    macro_rules! make_test {
        ($name:ident, $source:literal, $mnemonic:literal, $arguments:literal) => {
            #[test]
            fn $name() {
                let asm = assemble_one($source).unwrap();
                let dis = disasm(asm, 0x0);
                assert_eq!($mnemonic, dis.mnemonic().to_string());
                assert_eq!($arguments, dis.arguments().to_string());
            }
        };

        ($name:ident, $source:literal, $mnemonic:literal, $arguments:literal, $comment:literal) => {
            #[test]
            fn $name() {
                let asm = assemble_one($source).unwrap();
                let dis = disasm(asm, 0x0);
                assert_eq!($mnemonic, dis.mnemonic().to_string());
                assert_eq!($arguments, dis.arguments().to_string());
                assert_eq!($comment, dis.comment(0, None).to_string());
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
        [disasm_and_imm, "and r0, r1, #0x4", "and", "r0, r1, #0x4", "rhs = 4"],
        [disasm_ands_imm, "ands r0, r1, #0x4", "ands", "r0, r1, #0x4"],
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
        [disasm_eor_imm, "eor r0, r1, #0x4", "eor", "r0, r1, #0x4"],
        [disasm_eors_imm, "eors r0, r1, #0x4", "eors", "r0, r1, #0x4"],
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
        [disasm_sub_imm, "sub r0, r1, #0x4", "sub", "r0, r1, #0x4"],
        [disasm_subs_imm, "subs r0, r1, #0x4", "subs", "r0, r1, #0x4"],
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
        [disasm_rsb_imm, "rsb r0, r1, #0x4", "rsb", "r0, r1, #0x4"],
        [disasm_rsbs_imm, "rsbs r0, r1, #0x4", "rsbs", "r0, r1, #0x4"],
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
        [disasm_add_imm, "add r0, r1, #0x4", "add", "r0, r1, #0x4"],
        [disasm_adds_imm, "adds r0, r1, #0x4", "adds", "r0, r1, #0x4"],
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
        [disasm_adc_imm, "adc r0, r1, #0x4", "adc", "r0, r1, #0x4"],
        [disasm_adcs_imm, "adcs r0, r1, #0x4", "adcs", "r0, r1, #0x4"],
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
        [disasm_sbc_imm, "sbc r0, r1, #0x4", "sbc", "r0, r1, #0x4"],
        [disasm_sbcs_imm, "sbcs r0, r1, #0x4", "sbcs", "r0, r1, #0x4"],
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
        [disasm_rsc_imm, "rsc r0, r1, #0x4", "rsc", "r0, r1, #0x4"],
        [disasm_rscs_imm, "rscs r0, r1, #0x4", "rscs", "r0, r1, #0x4"],
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
        [disasm_orr_imm, "orr r0, r1, #0x4", "orr", "r0, r1, #0x4"],
        [disasm_orrs_imm, "orrs r0, r1, #0x4", "orrs", "r0, r1, #0x4"],
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
        [disasm_bic_imm, "bic r0, r1, #0x4", "bic", "r0, r1, #0x4"],
        [disasm_bics_imm, "bics r0, r1, #0x4", "bics", "r0, r1, #0x4"],
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
        [disasm_tst_imm, "tst r1, #0x4", "tst", "r1, #0x4"],
        [disasm_tsts_imm, "tsts r1, #0x4", "tst", "r1, #0x4"],
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
        [disasm_teq_imm, "teq r1, #0x4", "teq", "r1, #0x4"],
        [disasm_teqs_imm, "teqs r1, #0x4", "teq", "r1, #0x4"],
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
        [disasm_cmp_imm, "cmp r1, #0x4", "cmp", "r1, #0x4"],
        [disasm_cmps_imm, "cmps r1, #0x4", "cmp", "r1, #0x4"],
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
        [disasm_cmn_imm, "cmn r1, #0x4", "cmn", "r1, #0x4"],
        [disasm_cmns_imm, "cmns r1, #0x4", "cmn", "r1, #0x4"],
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
        [disasm_mov_imm, "mov r1, #0x4", "mov", "r1, #0x4"],
        [disasm_movs_imm, "movs r1, #0x4", "movs", "r1, #0x4"],
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
        [disasm_mvn_imm, "mvn r1, #0x4", "mvn", "r1, #0x4"],
        [disasm_mvns_imm, "mvns r1, #0x4", "mvns", "r1, #0x4"],
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
        [disasm_and_eq, "andeq r0, r1, #0x4", "andeq", "r0, r1, #0x4"],
        [disasm_ands_eq, "andeqs r0, r1, #0x4", "andeqs", "r0, r1, #0x4"],
        [disasm_and_ne, "andne r0, r1, #0x4", "andne", "r0, r1, #0x4"],
        [disasm_ands_ne, "andnes r0, r1, #0x4", "andnes", "r0, r1, #0x4"],
        [disasm_and_cs, "andcs r0, r1, #0x4", "andcs", "r0, r1, #0x4"],
        [disasm_ands_cs, "andcss r0, r1, #0x4", "andcss", "r0, r1, #0x4"],
        [disasm_and_cc, "andcc r0, r1, #0x4", "andcc", "r0, r1, #0x4"],
        [disasm_ands_cc, "andccs r0, r1, #0x4", "andccs", "r0, r1, #0x4"],
        [disasm_and_mi, "andmi r0, r1, #0x4", "andmi", "r0, r1, #0x4"],
        [disasm_ands_mi, "andmis r0, r1, #0x4", "andmis", "r0, r1, #0x4"],
        [disasm_and_pl, "andpl r0, r1, #0x4", "andpl", "r0, r1, #0x4"],
        [disasm_ands_pl, "andpls r0, r1, #0x4", "andpls", "r0, r1, #0x4"],
        [disasm_and_vs, "andvs r0, r1, #0x4", "andvs", "r0, r1, #0x4"],
        [disasm_ands_vs, "andvss r0, r1, #0x4", "andvss", "r0, r1, #0x4"],
        [disasm_and_vc, "andvc r0, r1, #0x4", "andvc", "r0, r1, #0x4"],
        [disasm_ands_vc, "andvcs r0, r1, #0x4", "andvcs", "r0, r1, #0x4"],
        [disasm_and_hi, "andhi r0, r1, #0x4", "andhi", "r0, r1, #0x4"],
        [disasm_ands_hi, "andhis r0, r1, #0x4", "andhis", "r0, r1, #0x4"],
        [disasm_and_ls, "andls r0, r1, #0x4", "andls", "r0, r1, #0x4"],
        [disasm_ands_ls, "andlss r0, r1, #0x4", "andlss", "r0, r1, #0x4"],
        [disasm_and_ge, "andge r0, r1, #0x4", "andge", "r0, r1, #0x4"],
        [disasm_ands_ge, "andges r0, r1, #0x4", "andges", "r0, r1, #0x4"],
        [disasm_and_lt, "andlt r0, r1, #0x4", "andlt", "r0, r1, #0x4"],
        [disasm_ands_lt, "andlts r0, r1, #0x4", "andlts", "r0, r1, #0x4"],
        [disasm_and_gt, "andgt r0, r1, #0x4", "andgt", "r0, r1, #0x4"],
        [disasm_ands_gt, "andgts r0, r1, #0x4", "andgts", "r0, r1, #0x4"],
        [disasm_and_le, "andle r0, r1, #0x4", "andle", "r0, r1, #0x4"],
        [disasm_ands_le, "andles r0, r1, #0x4", "andles", "r0, r1, #0x4"],

        // TST / TSTS
        [disasm_tst_eq, "tsteq r1, #0x4", "tsteq", "r1, #0x4"],
        [disasm_tsts_eq, "tsteqs r1, #0x4", "tsteq", "r1, #0x4"],
        [disasm_tst_ne, "tstne r1, #0x4", "tstne", "r1, #0x4"],
        [disasm_tsts_ne, "tstnes r1, #0x4", "tstne", "r1, #0x4"],
        [disasm_tst_cs, "tstcs r1, #0x4", "tstcs", "r1, #0x4"],
        [disasm_tsts_cs, "tstcss r1, #0x4", "tstcs", "r1, #0x4"],
        [disasm_tst_cc, "tstcc r1, #0x4", "tstcc", "r1, #0x4"],
        [disasm_tsts_cc, "tstccs r1, #0x4", "tstcc", "r1, #0x4"],
        [disasm_tst_mi, "tstmi r1, #0x4", "tstmi", "r1, #0x4"],
        [disasm_tsts_mi, "tstmis r1, #0x4", "tstmi", "r1, #0x4"],
        [disasm_tst_pl, "tstpl r1, #0x4", "tstpl", "r1, #0x4"],
        [disasm_tsts_pl, "tstpls r1, #0x4", "tstpl", "r1, #0x4"],
        [disasm_tst_vs, "tstvs r1, #0x4", "tstvs", "r1, #0x4"],
        [disasm_tsts_vs, "tstvss r1, #0x4", "tstvs", "r1, #0x4"],
        [disasm_tst_vc, "tstvc r1, #0x4", "tstvc", "r1, #0x4"],
        [disasm_tsts_vc, "tstvcs r1, #0x4", "tstvc", "r1, #0x4"],
        [disasm_tst_hi, "tsthi r1, #0x4", "tsthi", "r1, #0x4"],
        [disasm_tsts_hi, "tsthis r1, #0x4", "tsthi", "r1, #0x4"],
        [disasm_tst_ls, "tstls r1, #0x4", "tstls", "r1, #0x4"],
        [disasm_tsts_ls, "tstlss r1, #0x4", "tstls", "r1, #0x4"],
        [disasm_tst_ge, "tstge r1, #0x4", "tstge", "r1, #0x4"],
        [disasm_tsts_ge, "tstges r1, #0x4", "tstge", "r1, #0x4"],
        [disasm_tst_lt, "tstlt r1, #0x4", "tstlt", "r1, #0x4"],
        [disasm_tsts_lt, "tstlts r1, #0x4", "tstlt", "r1, #0x4"],
        [disasm_tst_gt, "tstgt r1, #0x4", "tstgt", "r1, #0x4"],
        [disasm_tsts_gt, "tstgts r1, #0x4", "tstgt", "r1, #0x4"],
        [disasm_tst_le, "tstle r1, #0x4", "tstle", "r1, #0x4"],
        [disasm_tsts_le, "tstles r1, #0x4", "tstle", "r1, #0x4"],
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

    // BRANCHES
    #[rustfmt::skip]
    make_tests! {
        [disasm_bx, "bx r2", "bx", "r2"],
        [disasm_b, "b 0x00081234", "b", "0x00081234"],
    }

    // PSR transfer
    #[rustfmt::skip]
    make_tests! {
        [disasm_mrs, "mrs r8, cpsr_all", "mrs", "r8, cpsr_all"],
        [disasm_msr_cpsr_all, "msr cpsr_all, r9", "msr", "cpsr_all, r9"],
        [disasm_msr_spsr_all, "msr spsr_all, r9", "msr", "spsr_all, r9"],
        [disasm_msr_cpsr_flg_reg, "msr cpsr_flg, r9", "msr", "cpsr_flg, r9"],
        [disasm_msr_spsr_flg_reg, "msr spsr_flg, r9", "msr", "spsr_flg, r9"],
        [disasm_msr_cpsr_flg_imm, "msr cpsr_flg, #0x10", "msr", "cpsr_flg, #0x10"],
        [disasm_msr_spsr_flg_imm, "msr spsr_flg, #0x10", "msr", "spsr_flg, #0x10"],
    }

    // Multiply
    #[rustfmt::skip]
    make_tests! {
        [disasm_mul, "mul r0, r1, r2", "mul", "r0, r1, r2"],
        [disasm_muls, "muls r0, r1, r2", "muls", "r0, r1, r2"],
        [disasm_mla, "mla r0, r1, r2, r3", "mla", "r0, r1, r2, r3"],
        [disasm_mlas, "mlas r0, r1, r2, r3", "mlas", "r0, r1, r2, r3"],
    }

    // Multiply Long
    #[rustfmt::skip]
    make_tests! {
        [disasm_umull, "umull r0, r1, r2, r3", "umull", "r0, r1, r2, r3"],
        [disasm_umlal, "umlal r0, r1, r2, r3", "umlal", "r0, r1, r2, r3"],
        [disasm_smull, "smull r0, r1, r2, r3", "smull", "r0, r1, r2, r3"],
        [disasm_smlal, "smlal r0, r1, r2, r3", "smlal", "r0, r1, r2, r3"],
    }

    // Single Data Transfer
    #[rustfmt::skip]
    make_tests! {
        // LDR
        [disasm_ldr_imm_pre, "ldr r0, [r1, #0x4]", "ldr", "r0, [r1, #0x4]"],
        [disasm_ldr_imm_pre_writeback, "ldr r0, [r1, #0x4]!", "ldr", "r0, [r1, #0x4]!"],
        [disasm_ldr_reg_pre, "ldr r0, [r1, r2]", "ldr", "r0, [r1, r2]"],
        [disasm_ldr_reg_pre_writeback, "ldr r0, [r1, r2]!", "ldr", "r0, [r1, r2]!"],
        [disasm_ldr_reg_pre_lsl, "ldr r0, [r1, r2, lsl #4]", "ldr", "r0, [r1, r2, lsl #4]"],
        [disasm_ldr_reg_pre_lsl_writeback, "ldr r0, [r1, r2, lsl #4]!", "ldr", "r0, [r1, r2, lsl #4]!"],
        [disasm_ldr_reg_pre_lsr, "ldr r0, [r1, r2, lsr #4]", "ldr", "r0, [r1, r2, lsr #4]"],
        [disasm_ldr_reg_pre_lsr_writeback, "ldr r0, [r1, r2, lsr #4]!", "ldr", "r0, [r1, r2, lsr #4]!"],
        [disasm_ldr_reg_pre_asr, "ldr r0, [r1, r2, asr #4]", "ldr", "r0, [r1, r2, asr #4]"],
        [disasm_ldr_reg_pre_asr_writeback, "ldr r0, [r1, r2, asr #4]!", "ldr", "r0, [r1, r2, asr #4]!"],
        [disasm_ldr_reg_pre_ror, "ldr r0, [r1, r2, ror #4]", "ldr", "r0, [r1, r2, ror #4]"],
        [disasm_ldr_reg_pre_ror_writeback, "ldr r0, [r1, r2, ror #4]!", "ldr", "r0, [r1, r2, ror #4]!"],
        [disasm_ldr_reg_pre_rrx, "ldr r0, [r1, r2, rrx]", "ldr", "r0, [r1, r2, rrx]"],
        [disasm_ldr_reg_pre_rrx_writeback, "ldr r0, [r1, r2, rrx]!", "ldr", "r0, [r1, r2, rrx]!"],
        [disasm_ldr_imm_post, "ldr r0, [r1], #0x4", "ldr", "r0, [r1], #0x4"],
        [disasm_ldr_reg_post, "ldr r0, [r1], r2", "ldr", "r0, [r1], r2"],
        [disasm_ldr_reg_post_lsl, "ldr r0, [r1], r2, lsl #4", "ldr", "r0, [r1], r2, lsl #4"],
        [disasm_ldr_reg_post_lsr, "ldr r0, [r1], r2, lsr #4", "ldr", "r0, [r1], r2, lsr #4"],
        [disasm_ldr_reg_post_asr, "ldr r0, [r1], r2, asr #4", "ldr", "r0, [r1], r2, asr #4"],
        [disasm_ldr_reg_post_ror, "ldr r0, [r1], r2, ror #4", "ldr", "r0, [r1], r2, ror #4"],
        [disasm_ldr_reg_post_rrx, "ldr r0, [r1], r2, rrx", "ldr", "r0, [r1], r2, rrx"],
        [disasm_ldrt_imm_post, "ldrt r0, [r1], #0x4", "ldrt", "r0, [r1], #0x4"],
        [disasm_ldrt_reg_post, "ldrt r0, [r1], r2", "ldrt", "r0, [r1], r2"],
        [disasm_ldrt_reg_post_lsl, "ldrt r0, [r1], r2, lsl #4", "ldrt", "r0, [r1], r2, lsl #4"],
        [disasm_ldrt_reg_post_lsr, "ldrt r0, [r1], r2, lsr #4", "ldrt", "r0, [r1], r2, lsr #4"],
        [disasm_ldrt_reg_post_asr, "ldrt r0, [r1], r2, asr #4", "ldrt", "r0, [r1], r2, asr #4"],
        [disasm_ldrt_reg_post_ror, "ldrt r0, [r1], r2, ror #4", "ldrt", "r0, [r1], r2, ror #4"],
        [disasm_ldrt_reg_post_rrx, "ldrt r0, [r1], r2, rrx", "ldrt", "r0, [r1], r2, rrx"],

        // STR
        [disasm_str_imm_pre, "str r0, [r1, #0x4]", "str", "r0, [r1, #0x4]"],
        [disasm_str_imm_pre_writeback, "str r0, [r1, #0x4]!", "str", "r0, [r1, #0x4]!"],
        [disasm_str_reg_pre, "str r0, [r1, r2]", "str", "r0, [r1, r2]"],
        [disasm_str_reg_pre_writeback, "str r0, [r1, r2]!", "str", "r0, [r1, r2]!"],
        [disasm_str_reg_pre_lsl, "str r0, [r1, r2, lsl #4]", "str", "r0, [r1, r2, lsl #4]"],
        [disasm_str_reg_pre_lsl_writeback, "str r0, [r1, r2, lsl #4]!", "str", "r0, [r1, r2, lsl #4]!"],
        [disasm_str_reg_pre_lsr, "str r0, [r1, r2, lsr #4]", "str", "r0, [r1, r2, lsr #4]"],
        [disasm_str_reg_pre_lsr_writeback, "str r0, [r1, r2, lsr #4]!", "str", "r0, [r1, r2, lsr #4]!"],
        [disasm_str_reg_pre_asr, "str r0, [r1, r2, asr #4]", "str", "r0, [r1, r2, asr #4]"],
        [disasm_str_reg_pre_asr_writeback, "str r0, [r1, r2, asr #4]!", "str", "r0, [r1, r2, asr #4]!"],
        [disasm_str_reg_pre_ror, "str r0, [r1, r2, ror #4]", "str", "r0, [r1, r2, ror #4]"],
        [disasm_str_reg_pre_ror_writeback, "str r0, [r1, r2, ror #4]!", "str", "r0, [r1, r2, ror #4]!"],
        [disasm_str_reg_pre_rrx, "str r0, [r1, r2, rrx]", "str", "r0, [r1, r2, rrx]"],
        [disasm_str_reg_pre_rrx_writeback, "str r0, [r1, r2, rrx]!", "str", "r0, [r1, r2, rrx]!"],
        [disasm_str_imm_post, "str r0, [r1], #0x4", "str", "r0, [r1], #0x4"],
        [disasm_str_reg_post, "str r0, [r1], r2", "str", "r0, [r1], r2"],
        [disasm_str_reg_post_lsl, "str r0, [r1], r2, lsl #4", "str", "r0, [r1], r2, lsl #4"],
        [disasm_str_reg_post_lsr, "str r0, [r1], r2, lsr #4", "str", "r0, [r1], r2, lsr #4"],
        [disasm_str_reg_post_asr, "str r0, [r1], r2, asr #4", "str", "r0, [r1], r2, asr #4"],
        [disasm_str_reg_post_ror, "str r0, [r1], r2, ror #4", "str", "r0, [r1], r2, ror #4"],
        [disasm_str_reg_post_rrx, "str r0, [r1], r2, rrx", "str", "r0, [r1], r2, rrx"],
        [disasm_strt_imm_post, "strt r0, [r1], #0x4", "strt", "r0, [r1], #0x4"],
        [disasm_strt_reg_post, "strt r0, [r1], r2", "strt", "r0, [r1], r2"],
        [disasm_strt_reg_post_lsl, "strt r0, [r1], r2, lsl #4", "strt", "r0, [r1], r2, lsl #4"],
        [disasm_strt_reg_post_lsr, "strt r0, [r1], r2, lsr #4", "strt", "r0, [r1], r2, lsr #4"],
        [disasm_strt_reg_post_asr, "strt r0, [r1], r2, asr #4", "strt", "r0, [r1], r2, asr #4"],
        [disasm_strt_reg_post_ror, "strt r0, [r1], r2, ror #4", "strt", "r0, [r1], r2, ror #4"],
        [disasm_strt_reg_post_rrx, "strt r0, [r1], r2, rrx", "strt", "r0, [r1], r2, rrx"],

        // LDRB
        [disasm_ldrb_imm_pre, "ldrb r0, [r1, #0x4]", "ldrb", "r0, [r1, #0x4]"],
        [disasm_ldrb_imm_pre_writeback, "ldrb r0, [r1, #0x4]!", "ldrb", "r0, [r1, #0x4]!"],
        [disasm_ldrb_reg_pre, "ldrb r0, [r1, r2]", "ldrb", "r0, [r1, r2]"],
        [disasm_ldrb_reg_pre_writeback, "ldrb r0, [r1, r2]!", "ldrb", "r0, [r1, r2]!"],
        [disasm_ldrb_reg_pre_lsl, "ldrb r0, [r1, r2, lsl #4]", "ldrb", "r0, [r1, r2, lsl #4]"],
        [disasm_ldrb_reg_pre_lsl_writeback, "ldrb r0, [r1, r2, lsl #4]!", "ldrb", "r0, [r1, r2, lsl #4]!"],
        [disasm_ldrb_reg_pre_lsr, "ldrb r0, [r1, r2, lsr #4]", "ldrb", "r0, [r1, r2, lsr #4]"],
        [disasm_ldrb_reg_pre_lsr_writeback, "ldrb r0, [r1, r2, lsr #4]!", "ldrb", "r0, [r1, r2, lsr #4]!"],
        [disasm_ldrb_reg_pre_asr, "ldrb r0, [r1, r2, asr #4]", "ldrb", "r0, [r1, r2, asr #4]"],
        [disasm_ldrb_reg_pre_asr_writeback, "ldrb r0, [r1, r2, asr #4]!", "ldrb", "r0, [r1, r2, asr #4]!"],
        [disasm_ldrb_reg_pre_ror, "ldrb r0, [r1, r2, ror #4]", "ldrb", "r0, [r1, r2, ror #4]"],
        [disasm_ldrb_reg_pre_ror_writeback, "ldrb r0, [r1, r2, ror #4]!", "ldrb", "r0, [r1, r2, ror #4]!"],
        [disasm_ldrb_reg_pre_rrx, "ldrb r0, [r1, r2, rrx]", "ldrb", "r0, [r1, r2, rrx]"],
        [disasm_ldrb_reg_pre_rrx_writeback, "ldrb r0, [r1, r2, rrx]!", "ldrb", "r0, [r1, r2, rrx]!"],
        [disasm_ldrb_imm_post, "ldrb r0, [r1], #0x4", "ldrb", "r0, [r1], #0x4"],
        [disasm_ldrb_reg_post, "ldrb r0, [r1], r2", "ldrb", "r0, [r1], r2"],
        [disasm_ldrb_reg_post_lsl, "ldrb r0, [r1], r2, lsl #4", "ldrb", "r0, [r1], r2, lsl #4"],
        [disasm_ldrb_reg_post_lsr, "ldrb r0, [r1], r2, lsr #4", "ldrb", "r0, [r1], r2, lsr #4"],
        [disasm_ldrb_reg_post_asr, "ldrb r0, [r1], r2, asr #4", "ldrb", "r0, [r1], r2, asr #4"],
        [disasm_ldrb_reg_post_ror, "ldrb r0, [r1], r2, ror #4", "ldrb", "r0, [r1], r2, ror #4"],
        [disasm_ldrb_reg_post_rrx, "ldrb r0, [r1], r2, rrx", "ldrb", "r0, [r1], r2, rrx"],
        [disasm_ldrbt_imm_post, "ldrbt r0, [r1], #0x4", "ldrbt", "r0, [r1], #0x4"],
        [disasm_ldrbt_reg_post, "ldrbt r0, [r1], r2", "ldrbt", "r0, [r1], r2"],
        [disasm_ldrbt_reg_post_lsl, "ldrbt r0, [r1], r2, lsl #4", "ldrbt", "r0, [r1], r2, lsl #4"],
        [disasm_ldrbt_reg_post_lsr, "ldrbt r0, [r1], r2, lsr #4", "ldrbt", "r0, [r1], r2, lsr #4"],
        [disasm_ldrbt_reg_post_asr, "ldrbt r0, [r1], r2, asr #4", "ldrbt", "r0, [r1], r2, asr #4"],
        [disasm_ldrbt_reg_post_ror, "ldrbt r0, [r1], r2, ror #4", "ldrbt", "r0, [r1], r2, ror #4"],
        [disasm_ldrbt_reg_post_rrx, "ldrbt r0, [r1], r2, rrx", "ldrbt", "r0, [r1], r2, rrx"],

        // STRB
        [disasm_strb_imm_pre, "strb r0, [r1, #0x4]", "strb", "r0, [r1, #0x4]"],
        [disasm_strb_imm_pre_writeback, "strb r0, [r1, #0x4]!", "strb", "r0, [r1, #0x4]!"],
        [disasm_strb_reg_pre, "strb r0, [r1, r2]", "strb", "r0, [r1, r2]"],
        [disasm_strb_reg_pre_writeback, "strb r0, [r1, r2]!", "strb", "r0, [r1, r2]!"],
        [disasm_strb_reg_pre_lsl, "strb r0, [r1, r2, lsl #4]", "strb", "r0, [r1, r2, lsl #4]"],
        [disasm_strb_reg_pre_lsl_writeback, "strb r0, [r1, r2, lsl #4]!", "strb", "r0, [r1, r2, lsl #4]!"],
        [disasm_strb_reg_pre_lsr, "strb r0, [r1, r2, lsr #4]", "strb", "r0, [r1, r2, lsr #4]"],
        [disasm_strb_reg_pre_lsr_writeback, "strb r0, [r1, r2, lsr #4]!", "strb", "r0, [r1, r2, lsr #4]!"],
        [disasm_strb_reg_pre_asr, "strb r0, [r1, r2, asr #4]", "strb", "r0, [r1, r2, asr #4]"],
        [disasm_strb_reg_pre_asr_writeback, "strb r0, [r1, r2, asr #4]!", "strb", "r0, [r1, r2, asr #4]!"],
        [disasm_strb_reg_pre_ror, "strb r0, [r1, r2, ror #4]", "strb", "r0, [r1, r2, ror #4]"],
        [disasm_strb_reg_pre_ror_writeback, "strb r0, [r1, r2, ror #4]!", "strb", "r0, [r1, r2, ror #4]!"],
        [disasm_strb_reg_pre_rrx, "strb r0, [r1, r2, rrx]", "strb", "r0, [r1, r2, rrx]"],
        [disasm_strb_reg_pre_rrx_writeback, "strb r0, [r1, r2, rrx]!", "strb", "r0, [r1, r2, rrx]!"],
        [disasm_strb_imm_post, "strb r0, [r1], #0x4", "strb", "r0, [r1], #0x4"],
        [disasm_strb_reg_post, "strb r0, [r1], r2", "strb", "r0, [r1], r2"],
        [disasm_strb_reg_post_lsl, "strb r0, [r1], r2, lsl #4", "strb", "r0, [r1], r2, lsl #4"],
        [disasm_strb_reg_post_lsr, "strb r0, [r1], r2, lsr #4", "strb", "r0, [r1], r2, lsr #4"],
        [disasm_strb_reg_post_asr, "strb r0, [r1], r2, asr #4", "strb", "r0, [r1], r2, asr #4"],
        [disasm_strb_reg_post_ror, "strb r0, [r1], r2, ror #4", "strb", "r0, [r1], r2, ror #4"],
        [disasm_strb_reg_post_rrx, "strb r0, [r1], r2, rrx", "strb", "r0, [r1], r2, rrx"],
        [disasm_strbt_imm_post, "strbt r0, [r1], #0x4", "strbt", "r0, [r1], #0x4"],
        [disasm_strbt_reg_post, "strbt r0, [r1], r2", "strbt", "r0, [r1], r2"],
        [disasm_strbt_reg_post_lsl, "strbt r0, [r1], r2, lsl #4", "strbt", "r0, [r1], r2, lsl #4"],
        [disasm_strbt_reg_post_lsr, "strbt r0, [r1], r2, lsr #4", "strbt", "r0, [r1], r2, lsr #4"],
        [disasm_strbt_reg_post_asr, "strbt r0, [r1], r2, asr #4", "strbt", "r0, [r1], r2, asr #4"],
        [disasm_strbt_reg_post_ror, "strbt r0, [r1], r2, ror #4", "strbt", "r0, [r1], r2, ror #4"],
        [disasm_strbt_reg_post_rrx, "strbt r0, [r1], r2, rrx", "strbt", "r0, [r1], r2, rrx"],

        // LDRH
        [disasm_ldrh_reg_pre, "ldrh r0, [r1, r2]", "ldrh", "r0, [r1, r2]"],
        [disasm_ldrh_reg_pre_writeback, "ldrh r0, [r1, r2]!", "ldrh", "r0, [r1, r2]!"],
        [disasm_ldrh_reg_post, "ldrh r0, [r1], r2", "ldrh", "r0, [r1], r2"],

        // STRH
        [disasm_strh_reg_pre, "strh r0, [r1, r2]", "strh", "r0, [r1, r2]"],
        [disasm_strh_reg_pre_writeback, "strh r0, [r1, r2]!", "strh", "r0, [r1, r2]!"],
        [disasm_strh_reg_post, "strh r0, [r1], r2", "strh", "r0, [r1], r2"],

        // LDRSB
        [disasm_ldrsb_reg_pre, "ldrsb r0, [r1, r2]", "ldrsb", "r0, [r1, r2]"],
        [disasm_ldrsb_reg_pre_writeback, "ldrsb r0, [r1, r2]!", "ldrsb", "r0, [r1, r2]!"],
        [disasm_ldrsb_reg_post, "ldrsb r0, [r1], r2", "ldrsb", "r0, [r1], r2"],

        // LDRSH
        [disasm_ldrsh_reg_pre, "ldrsb r0, [r1, r2]", "ldrsb", "r0, [r1, r2]"],
        [disasm_ldrsh_reg_pre_writeback, "ldrsb r0, [r1, r2]!", "ldrsb", "r0, [r1, r2]!"],
        [disasm_ldrsh_reg_post, "ldrsb r0, [r1], r2", "ldrsb", "r0, [r1], r2"],
    }

    // Block Data Transfer
    #[rustfmt::skip]
    make_tests! {
        // LDM
        [disasm_ldmib, "ldmib r0, {r1,r3-r4,r6-r10,lr}", "ldmib", "r0, {r1,r3-r4,r6-r10,lr}"],
        [disasm_ldmia, "ldmia r0, {r1,r3-r4,r6-r10,lr}", "ldmia", "r0, {r1,r3-r4,r6-r10,lr}"],
        [disasm_ldmdb, "ldmdb r0, {r1,r3-r4,r6-r10,lr}", "ldmdb", "r0, {r1,r3-r4,r6-r10,lr}"],
        [disasm_ldmda, "ldmda r0, {r1,r3-r4,r6-r10,lr}", "ldmda", "r0, {r1,r3-r4,r6-r10,lr}"],
        [disasm_ldmib_writeback, "ldmib r0!, {r1,r3-r4,r6-r10,lr}", "ldmib", "r0!, {r1,r3-r4,r6-r10,lr}"],
        [disasm_ldmia_writeback, "ldmia r0!, {r1,r3-r4,r6-r10,lr}", "ldmia", "r0!, {r1,r3-r4,r6-r10,lr}"],
        [disasm_ldmdb_writeback, "ldmdb r0!, {r1,r3-r4,r6-r10,lr}", "ldmdb", "r0!, {r1,r3-r4,r6-r10,lr}"],
        [disasm_ldmda_writeback, "ldmda r0!, {r1,r3-r4,r6-r10,lr}", "ldmda", "r0!, {r1,r3-r4,r6-r10,lr}"],
        [disasm_ldmib_s, "ldmib r0, {r1,r3-r4,r6-r10,lr}^", "ldmib", "r0, {r1,r3-r4,r6-r10,lr}^"],
        [disasm_ldmia_s, "ldmia r0, {r1,r3-r4,r6-r10,lr}^", "ldmia", "r0, {r1,r3-r4,r6-r10,lr}^"],
        [disasm_ldmdb_s, "ldmdb r0, {r1,r3-r4,r6-r10,lr}^", "ldmdb", "r0, {r1,r3-r4,r6-r10,lr}^"],
        [disasm_ldmda_s, "ldmda r0, {r1,r3-r4,r6-r10,lr}^", "ldmda", "r0, {r1,r3-r4,r6-r10,lr}^"],
        [disasm_ldmib_s_writeback, "ldmib r0!, {r1,r3-r4,r6-r10,lr}^", "ldmib", "r0!, {r1,r3-r4,r6-r10,lr}^"],
        [disasm_ldmia_s_writeback, "ldmia r0!, {r1,r3-r4,r6-r10,lr}^", "ldmia", "r0!, {r1,r3-r4,r6-r10,lr}^"],
        [disasm_ldmdb_s_writeback, "ldmdb r0!, {r1,r3-r4,r6-r10,lr}^", "ldmdb", "r0!, {r1,r3-r4,r6-r10,lr}^"],
        [disasm_ldmda_s_writeback, "ldmda r0!, {r1,r3-r4,r6-r10,lr}^", "ldmda", "r0!, {r1,r3-r4,r6-r10,lr}^"],

        // STM
        [disasm_stmib, "stmib r0, {r1,r3-r4,r6-r10,lr}", "stmib", "r0, {r1,r3-r4,r6-r10,lr}"],
        [disasm_stmia, "stmia r0, {r1,r3-r4,r6-r10,lr}", "stmia", "r0, {r1,r3-r4,r6-r10,lr}"],
        [disasm_stmdb, "stmdb r0, {r1,r3-r4,r6-r10,lr}", "stmdb", "r0, {r1,r3-r4,r6-r10,lr}"],
        [disasm_stmda, "stmda r0, {r1,r3-r4,r6-r10,lr}", "stmda", "r0, {r1,r3-r4,r6-r10,lr}"],
        [disasm_stmib_writeback, "stmib r0!, {r1,r3-r4,r6-r10,lr}", "stmib", "r0!, {r1,r3-r4,r6-r10,lr}"],
        [disasm_stmia_writeback, "stmia r0!, {r1,r3-r4,r6-r10,lr}", "stmia", "r0!, {r1,r3-r4,r6-r10,lr}"],
        [disasm_stmdb_writeback, "stmdb r0!, {r1,r3-r4,r6-r10,lr}", "stmdb", "r0!, {r1,r3-r4,r6-r10,lr}"],
        [disasm_stmda_writeback, "stmda r0!, {r1,r3-r4,r6-r10,lr}", "stmda", "r0!, {r1,r3-r4,r6-r10,lr}"],
        [disasm_stmib_s, "stmib r0, {r1,r3-r4,r6-r10,lr}^", "stmib", "r0, {r1,r3-r4,r6-r10,lr}^"],
        [disasm_stmia_s, "stmia r0, {r1,r3-r4,r6-r10,lr}^", "stmia", "r0, {r1,r3-r4,r6-r10,lr}^"],
        [disasm_stmdb_s, "stmdb r0, {r1,r3-r4,r6-r10,lr}^", "stmdb", "r0, {r1,r3-r4,r6-r10,lr}^"],
        [disasm_stmda_s, "stmda r0, {r1,r3-r4,r6-r10,lr}^", "stmda", "r0, {r1,r3-r4,r6-r10,lr}^"],
        [disasm_stmib_s_writeback, "stmib r0!, {r1,r3-r4,r6-r10,lr}^", "stmib", "r0!, {r1,r3-r4,r6-r10,lr}^"],
        [disasm_stmia_s_writeback, "stmia r0!, {r1,r3-r4,r6-r10,lr}^", "stmia", "r0!, {r1,r3-r4,r6-r10,lr}^"],
        [disasm_stmdb_s_writeback, "stmdb r0!, {r1,r3-r4,r6-r10,lr}^", "stmdb", "r0!, {r1,r3-r4,r6-r10,lr}^"],
        [disasm_stmda_s_writeback, "stmda r0!, {r1,r3-r4,r6-r10,lr}^", "stmda", "r0!, {r1,r3-r4,r6-r10,lr}^"],
    }

    // Single Data Swap
    #[rustfmt::skip]
    make_tests! {
        [disasm_swp, "swp r0, r1, [r2]", "swp", "r0, r1, [r2]"],
        [disasm_swpb, "swpb r0, r1, [r2]", "swpb", "r0, r1, [r2]"],
    }

    // Software Interrupt
    #[rustfmt::skip]
    make_tests! {
        [disasm_swi, "swi #0x123456", "swi", "#0x123456"],
    }

    // Load PC-relative
    #[rustfmt::skip]
    make_tests! {
        [disasm_ldr_pc_relative, "ldr r0, [pc, #0x4]", "ldr", "r0, [pc, #0x4]", "r0 = [0x0000000c]"],
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
