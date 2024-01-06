use std::fmt::Write;
use util::bits::BitOps as _;

use crate::{
    common::{
        Condition, DataProc, DataTransferDirection, DataTransferIndexing, DataTransferOp, Register,
        RegisterList, RegisterOrImmediate, SDTDataType, ShiftType,
    },
    MemoryView,
};

pub fn disasm(instr: u16, address: u32) -> ThumbInstr {
    let opcode_row = instr.get_bit_range(12..=15);
    let opcode_col = instr.get_bit_range(8..=11);
    let opcode_idx = (opcode_row * 16) + opcode_col;
    match opcode_idx as u8 {
        0x00..=0x17 => disasm_move_shifted_register(instr),
        0x18..=0x1B => disasm_add_subtract_reg3(instr),
        0x1C..=0x1F => disasm_add_subtract_imm3(instr),
        0x20..=0x3F => disasm_mov_cmp_add_sub_imm8(instr),
        0x40..=0x43 => disasm_alu_op(instr),
        0x44..=0x47 => disasm_hi_reg_op(instr),
        0x48..=0x4F => disasm_ldr_pc_relative_imm10(instr),
        0x50..=0x51 => disasm_ldr_and_str_reg(instr),
        0x52..=0x53 => disasm_ldrh_and_strsb_reg(instr),
        0x54..=0x55 => disasm_ldr_and_str_reg(instr),
        0x56..=0x57 => disasm_ldrh_and_strsb_reg(instr),
        0x58..=0x59 => disasm_ldr_and_str_reg(instr),
        0x5A..=0x5B => disasm_ldrh_and_strsb_reg(instr),
        0x5C..=0x5D => disasm_ldr_and_str_reg(instr),
        0x5E..=0x5F => disasm_ldrh_and_strsb_reg(instr),
        0x60..=0x6F => disasm_ldr_and_str_imm7(instr),
        0x70..=0x7F => disasm_ldrb_and_strb_imm5(instr),
        0x80..=0x8F => disasm_ldrh_and_strh_imm6(instr),
        0x90..=0x9F => disasm_ldr_and_str_sp_relative_imm10(instr),
        0xA0..=0xAF => disasm_load_address(instr),
        0xB0..=0xB0 => disasm_add_sp(instr),
        0xB1..=0xB3 => ThumbInstr::Undefined(instr),
        0xB4..=0xB5 => disasm_push_pop_registers(instr),
        0xB6..=0xBB => ThumbInstr::Undefined(instr),
        0xBC..=0xBD => disasm_push_pop_registers(instr),
        0xBE..=0xBE => disasm_bkpt(instr),
        0xBF..=0xBF => ThumbInstr::Undefined(instr),
        0xC0..=0xCF => disasm_block_data_transfer(instr),
        0xD0..=0xDD => disasm_conditional_branch(instr, address),
        0xDE..=0xDE => ThumbInstr::Undefined(instr),
        0xDF..=0xDF => disasm_swi(instr),
        0xE0..=0xE7 => disasm_unconditional_branch(instr, address),
        0xE8..=0xEF => disasm_blx(instr),
        0xF0..=0xF7 => disasm_bl_setup(instr, address),
        0xF8..=0xFF => disasm_bl_complete(instr),
    }
}

fn disasm_move_shifted_register(instr: u16) -> ThumbInstr {
    ThumbInstr::MoveShiftedRegister {
        shift: match instr.get_bit_range(11..=12) {
            0 => ShiftType::Lsl,
            1 => ShiftType::Lsr,
            2 => ShiftType::Asr,
            _ => unreachable!("invalid shift type"),
        },
        lhs: Some(Register::from(instr.get_bit_range(3..=5))),
        dst: Register::from(instr.get_bit_range(0..=2)),
        rhs: RegisterOrImmediate::Immediate(instr.get_bit_range(6..=10) as u32),
    }
}

fn disasm_add_subtract_imm3(instr: u16) -> ThumbInstr {
    ThumbInstr::DataProc {
        op: if instr.get_bit(9) {
            DataProc::Sub
        } else {
            DataProc::Add
        },
        dst: Register::from(instr.get_bit_range(0..=2)),
        lhs: Some(Register::from(instr.get_bit_range(3..=5))),
        rhs: RegisterOrImmediate::Immediate(instr.get_bit_range(6..=8) as u32),
    }
}

fn disasm_add_subtract_reg3(instr: u16) -> ThumbInstr {
    ThumbInstr::DataProc {
        op: if instr.get_bit(9) {
            DataProc::Sub
        } else {
            DataProc::Add
        },
        dst: Register::from(instr.get_bit_range(0..=2)),
        lhs: Some(Register::from(instr.get_bit_range(3..=5))),
        rhs: RegisterOrImmediate::Register(Register::from(instr.get_bit_range(6..=8))), // Rn
    }
}

fn disasm_mov_cmp_add_sub_imm8(instr: u16) -> ThumbInstr {
    ThumbInstr::DataProc {
        op: match instr.get_bit_range(11..=12) {
            0 => DataProc::Mov,
            1 => DataProc::Cmp,
            2 => DataProc::Add,
            3 => DataProc::Sub,
            _ => unreachable!("invalid opcode"),
        },
        dst: Register::from(instr.get_bit_range(8..=10)),
        lhs: None,
        rhs: RegisterOrImmediate::Immediate(instr.get_bit_range(0..=7) as u32),
    }
}

fn disasm_alu_op(instr: u16) -> ThumbInstr {
    let dst = Register::from(instr.get_bit_range(0..=2));
    let rhs = Register::from(instr.get_bit_range(3..=5));

    let make_shift = |shift: ShiftType| -> ThumbInstr {
        ThumbInstr::MoveShiftedRegister {
            shift,
            lhs: None,
            dst,
            rhs: RegisterOrImmediate::Register(rhs),
        }
    };

    let op = match instr.get_bit_range(6..=9) {
        0b0000 => DataProc::And,
        0b0001 => DataProc::Eor,
        0b0010 => return make_shift(ShiftType::Lsl),
        0b0011 => return make_shift(ShiftType::Lsr),
        0b0100 => return make_shift(ShiftType::Asr),
        0b0101 => DataProc::Adc,
        0b0110 => DataProc::Sbc,
        0b0111 => return make_shift(ShiftType::Ror),
        0b1000 => DataProc::Tst,
        0b1001 => {
            return ThumbInstr::DataProc {
                op: DataProc::Rsb,
                dst,
                lhs: Some(rhs),
                rhs: RegisterOrImmediate::Immediate(0),
            }
        }
        0b1010 => DataProc::Cmp,
        0b1011 => DataProc::Cmn,
        0b1100 => DataProc::Orr,
        0b1101 => return ThumbInstr::Multiply { dst, rhs },
        0b1110 => DataProc::Bic,
        0b1111 => DataProc::Mvn,
        _ => unreachable!("invalid opcode"),
    };

    ThumbInstr::DataProc {
        op,
        dst,
        lhs: None,
        rhs: RegisterOrImmediate::Register(rhs),
    }
}

fn disasm_hi_reg_op(instr: u16) -> ThumbInstr {
    let h1 = instr.get_bit_int(7) << 3;
    let h2 = instr.get_bit_int(6) << 3;

    let dst = Register::from(instr.get_bit_range(0..=2) + h1);
    let rhs = Register::from(instr.get_bit_range(3..=5) + h2);
    let op = match instr.get_bit_range(8..=9) {
        0b00 => DataProc::Add,
        0b01 => DataProc::Cmp,
        0b10 => DataProc::Mov,
        0b11 => return ThumbInstr::BranchAndExchange { rs: rhs },
        _ => unreachable!("invalid opcode"),
    };

    // The action of H1= 0, H2 = 0 for Op = 00 (ADD), Op =01 (CMP) and Op = 10 (MOV) is
    // undefined, and should not be used.
    if h1 == 0 && h2 == 0 {
        return ThumbInstr::Undefined(instr);
    }

    ThumbInstr::DataProc {
        op,
        dst,
        lhs: None,
        rhs: RegisterOrImmediate::Register(rhs),
    }
}

fn disasm_ldr_pc_relative_imm10(instr: u16) -> ThumbInstr {
    let offset = (instr.get_bit_range(0..=7) as u32) << 2;
    ThumbInstr::SingleDataTransfer {
        op: DataTransferOp::Load,
        data_type: SDTDataType::Word,
        dst: Register::from(instr.get_bit_range(8..=10)),
        src: Register::R15,
        off: RegisterOrImmediate::Immediate(offset),
    }
}

fn disasm_ldr_and_str_reg(instr: u16) -> ThumbInstr {
    ThumbInstr::SingleDataTransfer {
        op: if instr.get_bit(11) {
            DataTransferOp::Load
        } else {
            DataTransferOp::Store
        },
        data_type: if instr.get_bit(10) {
            SDTDataType::Byte
        } else {
            SDTDataType::Word
        },
        dst: Register::from(instr.get_bit_range(0..=2)),
        src: Register::from(instr.get_bit_range(3..=5)),
        off: RegisterOrImmediate::Register(Register::from(instr.get_bit_range(6..=8))),
    }
}

fn disasm_ldrh_and_strsb_reg(instr: u16) -> ThumbInstr {
    let (op, data_type) = match instr.get_bit_range(10..=11) {
        0b00 => (DataTransferOp::Store, SDTDataType::Halfword),
        0b01 => (DataTransferOp::Load, SDTDataType::SignedByte),
        0b10 => (DataTransferOp::Load, SDTDataType::Halfword),
        0b11 => (DataTransferOp::Load, SDTDataType::SignedHalfword),
        _ => unreachable!("invalid opcode"),
    };

    ThumbInstr::SingleDataTransfer {
        op,
        data_type,
        dst: Register::from(instr.get_bit_range(0..=2)),
        src: Register::from(instr.get_bit_range(3..=5)),
        off: RegisterOrImmediate::Register(Register::from(instr.get_bit_range(6..=8))),
    }
}

fn disasm_ldr_and_str_imm7(instr: u16) -> ThumbInstr {
    ThumbInstr::SingleDataTransfer {
        op: if instr.get_bit(11) {
            DataTransferOp::Load
        } else {
            DataTransferOp::Store
        },
        data_type: SDTDataType::Word,
        dst: Register::from(instr.get_bit_range(0..=2)),
        src: Register::from(instr.get_bit_range(3..=5)),
        off: RegisterOrImmediate::Immediate((instr.get_bit_range(6..=10) as u32) << 2),
    }
}

fn disasm_ldrb_and_strb_imm5(instr: u16) -> ThumbInstr {
    ThumbInstr::SingleDataTransfer {
        op: if instr.get_bit(11) {
            DataTransferOp::Load
        } else {
            DataTransferOp::Store
        },
        data_type: SDTDataType::Byte,
        dst: Register::from(instr.get_bit_range(0..=2)),
        src: Register::from(instr.get_bit_range(3..=5)),
        off: RegisterOrImmediate::Immediate(instr.get_bit_range(6..=10) as u32),
    }
}

fn disasm_ldrh_and_strh_imm6(instr: u16) -> ThumbInstr {
    ThumbInstr::SingleDataTransfer {
        op: if instr.get_bit(11) {
            DataTransferOp::Load
        } else {
            DataTransferOp::Store
        },
        data_type: SDTDataType::Halfword,
        dst: Register::from(instr.get_bit_range(0..=2)),
        src: Register::from(instr.get_bit_range(3..=5)),
        off: RegisterOrImmediate::Immediate((instr.get_bit_range(6..=10) as u32) << 1),
    }
}

fn disasm_ldr_and_str_sp_relative_imm10(instr: u16) -> ThumbInstr {
    ThumbInstr::SingleDataTransfer {
        op: if instr.get_bit(11) {
            DataTransferOp::Load
        } else {
            DataTransferOp::Store
        },
        data_type: SDTDataType::Word,
        dst: Register::from(instr.get_bit_range(8..=10)),
        src: Register::R13,
        off: RegisterOrImmediate::Immediate((instr.get_bit_range(0..=7) as u32) << 2),
    }
}

fn disasm_load_address(instr: u16) -> ThumbInstr {
    let offset = (instr.get_bit_range(0..=7) as u32) << 2;
    ThumbInstr::DataProc {
        op: DataProc::Add,
        dst: Register::from(instr.get_bit_range(8..=10)),
        lhs: Some(if instr.get_bit(11) {
            Register::R13
        } else {
            Register::R15
        }),
        rhs: RegisterOrImmediate::Immediate(offset),
    }
}

fn disasm_add_sp(instr: u16) -> ThumbInstr {
    let offset = (instr.get_bit_range(0..=6) as u32) << 2;
    ThumbInstr::DataProc {
        op: if instr.get_bit(7) {
            DataProc::Sub
        } else {
            DataProc::Add
        },
        dst: Register::R13,
        lhs: None,
        rhs: RegisterOrImmediate::Immediate(offset),
    }
}

fn disasm_push_pop_registers(instr: u16) -> ThumbInstr {
    let (op, direction, indexing, extra) = match (instr.get_bit(11), instr.get_bit(8)) {
        (false, false) => (
            DataTransferOp::Store,
            DataTransferDirection::Down,
            DataTransferIndexing::Pre,
            None,
        ),
        (false, true) => (
            DataTransferOp::Store,
            DataTransferDirection::Down,
            DataTransferIndexing::Pre,
            Some(Register::R14),
        ),
        (true, false) => (
            DataTransferOp::Load,
            DataTransferDirection::Up,
            DataTransferIndexing::Post,
            None,
        ),
        (true, true) => (
            DataTransferOp::Load,
            DataTransferDirection::Up,
            DataTransferIndexing::Post,
            Some(Register::R15),
        ),
    };
    let extra: Option<Register> = extra;
    let mut register_list = RegisterList::from(instr.get_bit_range(0..=7));

    match extra {
        Some(Register::R13) => register_list.set(Register::R13),
        Some(Register::R14) => register_list.set(Register::R14),
        Some(Register::R15) => register_list.set(Register::R15),
        None => (),
        _ => unreachable!("invalid extra register"),
    }

    ThumbInstr::BlockDataTransfer {
        op,
        direction,
        indexing,
        rn: Register::R13,
        registers: register_list,
    }
}

fn disasm_block_data_transfer(instr: u16) -> ThumbInstr {
    ThumbInstr::BlockDataTransfer {
        op: if instr.get_bit(11) {
            DataTransferOp::Load
        } else {
            DataTransferOp::Store
        },
        direction: DataTransferDirection::Up,
        indexing: DataTransferIndexing::Post,
        rn: Register::from(instr.get_bit_range(8..=10)),
        registers: RegisterList::from(instr.get_bit_range(0..=7)),
    }
}

fn disasm_conditional_branch(instr: u16, address: u32) -> ThumbInstr {
    let condition = Condition::from(instr.get_bit_range(8..=11) as u32);
    if matches!(condition, Condition::Al | Condition::Nv) {
        return ThumbInstr::Undefined(instr);
    }
    let offset = (((instr as u32) & 0xFF) << 1).sign_extend(9);
    let pc = address.wrapping_add(4);
    let dest = pc.wrapping_add(offset) & 0xFFFFFFFE;
    ThumbInstr::Branch { condition, dest }
}

fn disasm_unconditional_branch(instr: u16, address: u32) -> ThumbInstr {
    let offset = (((instr as u32) & 0x7FF) << 1).sign_extend(12);
    let pc = address.wrapping_add(4);
    let dest = pc.wrapping_add(offset) & 0xFFFFFFFE;
    ThumbInstr::Branch {
        condition: Condition::Al,
        dest,
    }
}

fn disasm_bl_setup(instr: u16, address: u32) -> ThumbInstr {
    let pc = address.wrapping_add(4);
    let off = (((instr as u32) & 0x7FF) << 12).sign_extend(23);
    let setup = pc.wrapping_add(off);
    ThumbInstr::BrandAndLinkSetup(setup)
}

fn disasm_bl_complete(instr: u16) -> ThumbInstr {
    let off = ((instr as u32) & 0x7FF) << 1;
    ThumbInstr::BranchAndLink(off)
}

fn disasm_bkpt(instr: u16) -> ThumbInstr {
    ThumbInstr::Undefined(instr)
}

fn disasm_blx(instr: u16) -> ThumbInstr {
    ThumbInstr::Undefined(instr)
}

fn disasm_swi(instr: u16) -> ThumbInstr {
    ThumbInstr::SoftwareInterrupt {
        comment: instr.get_bit_range(0..=7) as u8,
    }
}

#[derive(Debug)]
pub enum ThumbInstr {
    Undefined(u16),

    SingleDataTransfer {
        op: DataTransferOp,
        data_type: SDTDataType,
        dst: Register,
        src: Register,
        off: RegisterOrImmediate,
    },

    MoveShiftedRegister {
        dst: Register,
        lhs: Option<Register>,
        rhs: RegisterOrImmediate,
        shift: ShiftType,
    },

    DataProc {
        op: DataProc,
        dst: Register,
        lhs: Option<Register>, // dst doubles as lhs if None
        rhs: RegisterOrImmediate,
    },

    Multiply {
        dst: Register,
        rhs: Register,
    },

    BranchAndExchange {
        rs: Register,
    },

    BlockDataTransfer {
        op: DataTransferOp,
        direction: DataTransferDirection,
        indexing: DataTransferIndexing,
        rn: Register,
        registers: RegisterList,
    },

    SoftwareInterrupt {
        comment: u8,
    },

    Branch {
        condition: Condition,
        dest: u32,
    },

    BrandAndLinkSetup(u32),
    BranchAndLink(u32),
}

impl ThumbInstr {
    pub(crate) fn write_mnemonic<W: Write>(&self, mut f: W) -> std::fmt::Result {
        match self {
            ThumbInstr::Undefined(_) => write!(f, "undef"),
            ThumbInstr::SoftwareInterrupt { .. } => write!(f, "swi"),
            ThumbInstr::MoveShiftedRegister { shift, .. } => write!(f, "{shift}"),
            ThumbInstr::DataProc { op, lhs, rhs, .. } => {
                if *op == DataProc::Rsb
                    && lhs.is_some()
                    && *rhs == RegisterOrImmediate::Immediate(0)
                {
                    write!(f, "neg")
                } else {
                    write!(f, "{op}")
                }
            }
            ThumbInstr::Multiply { .. } => write!(f, "mul"),
            ThumbInstr::BranchAndExchange { .. } => write!(f, "bx"),
            ThumbInstr::SingleDataTransfer { op, data_type, .. } => {
                let op = match op {
                    DataTransferOp::Load => "ldr",
                    DataTransferOp::Store => "str",
                };

                let dt = match data_type {
                    SDTDataType::Word => "",
                    SDTDataType::Byte => "b",
                    SDTDataType::Halfword => "h",
                    SDTDataType::SignedHalfword => "sh",
                    SDTDataType::SignedByte => "sb",
                };

                write!(f, "{op}{dt}")
            }
            ThumbInstr::BlockDataTransfer {
                op,
                direction,
                indexing,
                rn,
                ..
            } => {
                let proc = match (op, direction, indexing) {
                    (
                        DataTransferOp::Store,
                        DataTransferDirection::Down,
                        DataTransferIndexing::Pre,
                    ) if *rn == Register::R13 => "push",
                    (
                        DataTransferOp::Load,
                        DataTransferDirection::Up,
                        DataTransferIndexing::Post,
                    ) if *rn == Register::R13 => "pop",
                    (
                        DataTransferOp::Load,
                        DataTransferDirection::Up,
                        DataTransferIndexing::Post,
                    ) => "ldmia",
                    (
                        DataTransferOp::Store,
                        DataTransferDirection::Up,
                        DataTransferIndexing::Post,
                    ) => "stmia",
                    _ => unreachable!("invalid block data transfer"),
                };
                write!(f, "{proc}")
            }
            ThumbInstr::Branch { condition, .. } => {
                write!(f, "b{condition}")
            }
            ThumbInstr::BrandAndLinkSetup(..) => write!(f, "bl_setup"),
            ThumbInstr::BranchAndLink(..) => write!(f, "bl"),
        }
    }

    pub(crate) fn write_arguments<W: Write>(
        &self,
        mut f: W,
        addr: u32,
        memory: Option<&dyn MemoryView>,
    ) -> std::fmt::Result {
        match self {
            ThumbInstr::Undefined(instr) => write!(f, "0x{:04x}", instr),
            ThumbInstr::SoftwareInterrupt { comment } => write!(f, "#0x{:02x}", comment),
            ThumbInstr::MoveShiftedRegister {
                rhs,
                dst,
                lhs: maybe_lhs,
                ..
            } => {
                if let Some(lhs) = maybe_lhs {
                    write!(f, "{dst}, {lhs}, {rhs}")
                } else {
                    write!(f, "{dst}, {rhs}")
                }
            }
            ThumbInstr::DataProc {
                op,
                dst,
                lhs: maybe_lhs,
                rhs,
                ..
            } => {
                let lhs = maybe_lhs.unwrap_or(*dst);
                match op {
                    DataProc::Mov | DataProc::Mvn => write!(f, "{dst}, {rhs:x}"),
                    DataProc::Tst | DataProc::Teq | DataProc::Cmp | DataProc::Cmn => {
                        write!(f, "{lhs}, {rhs:x}")
                    }
                    DataProc::Rsb
                        if *op == DataProc::Rsb
                            && maybe_lhs.is_some()
                            && *rhs == RegisterOrImmediate::Immediate(0) =>
                    {
                        write!(f, "{dst}, {lhs}")
                    }
                    _ if maybe_lhs.is_none() => write!(f, "{dst}, {rhs:x}"),
                    _ => write!(f, "{dst}, {lhs}, {rhs:x}"),
                }
            }
            ThumbInstr::Multiply { dst, rhs } => write!(f, "{dst}, {rhs}"),
            ThumbInstr::BranchAndExchange { rs } => write!(f, "{rs}"),
            ThumbInstr::SingleDataTransfer { dst, src, off, .. } => {
                write!(f, "{dst}, [{src}, {off:x}]")
            }
            &ThumbInstr::BlockDataTransfer {
                op,
                direction,
                indexing,
                rn,
                registers,
            } => {
                let is_push = op == DataTransferOp::Store
                    && direction == DataTransferDirection::Down
                    && indexing == DataTransferIndexing::Pre
                    && rn == Register::R13;
                let is_pop = op == DataTransferOp::Load
                    && direction == DataTransferDirection::Up
                    && indexing == DataTransferIndexing::Post
                    && rn == Register::R13;
                if is_push || is_pop {
                    write!(f, "{registers}")
                } else {
                    write!(f, "{rn}!, {registers}")
                }
            }
            ThumbInstr::Branch { dest, .. } => write!(f, "0x{dest:08x}"),

            ThumbInstr::BrandAndLinkSetup(..) => Ok(()),
            &ThumbInstr::BranchAndLink(offset) => {
                if let Some(memory) = memory {
                    let setup_instr_bytes = memory.view16(addr.wrapping_sub(2));
                    let setup_instr = disasm(setup_instr_bytes, addr.wrapping_sub(2));

                    if let ThumbInstr::BrandAndLinkSetup(lr) = setup_instr {
                        let dest = lr.wrapping_add(offset) & 0xFFFFFFFE;
                        write!(f, "0x{dest:08x}")
                    } else {
                        write!(f, "<invalid>")
                    }
                } else {
                    write!(f, "<unknown>")
                }
            }
        }
    }

    pub(crate) fn write_comment<W: Write>(
        &self,
        mut f: W,
        addr: u32,
        m: Option<&dyn MemoryView>,
    ) -> std::fmt::Result {
        match *self {
            ThumbInstr::SingleDataTransfer {
                op: DataTransferOp::Load,
                data_type,
                dst,
                src: Register::R15,
                off: RegisterOrImmediate::Immediate(off),
            } => {
                let pc = addr.wrapping_add(4);
                let data_addr = pc.wrapping_add(off);
                if let Some(m) = m {
                    match data_type {
                        SDTDataType::Word => {
                            let data = m
                                .view32(data_addr & !0x03)
                                .rotate_right(8 * (data_addr % 4));
                            write!(f, "{dst} = 0x{data:08x}")
                        }
                        SDTDataType::Byte => {
                            let data = m.view8(data_addr);
                            write!(f, "{dst} = 0x{data:02x}")
                        }
                        SDTDataType::Halfword => {
                            let data = m.view16(data_addr & !0x1);
                            write!(f, "{dst} = 0x{data:04x}")
                        }
                        SDTDataType::SignedHalfword => {
                            let data = m.view16(data_addr & !0x1) as i16;
                            write!(f, "{dst} = 0x{data:04x}")
                        }
                        SDTDataType::SignedByte => {
                            let data = m.view8(data_addr) as i8;
                            write!(f, "{dst} = 0x{data:02x}")
                        }
                    }
                } else {
                    write!(f, "{dst} = [0x{data_addr:08x}]")
                }
            }

            ThumbInstr::BrandAndLinkSetup(setup) => {
                write!(f, "lr = 0x{:08x}", setup)
            }

            _ => Ok(()),
        }
    }

    pub fn mnemonic(&self) -> crate::Mnemonic<'_, Self> {
        crate::Mnemonic(self)
    }

    pub fn arguments<'s, 'm>(
        &'s self,
        addr: u32,
        memory: Option<&'m dyn MemoryView>,
    ) -> crate::Arguments<'s, 'm, Self> {
        crate::Arguments(self, addr, memory)
    }

    pub fn comment<'s>(
        &'s self,
        addr: u32,
        m: Option<&'s dyn MemoryView>,
    ) -> crate::Comment<'s, 's, Self> {
        crate::Comment(self, addr, m)
    }
}

#[cfg(test)]
mod tests {
    use super::disasm;
    use arm_devkit::LinkerScriptWeakRef;
    use std::sync::RwLock;

    #[test]
    fn disasm_undef() {
        // instructions are undefined for these values of the top 8 bits
        const UNDEFINED_INSTRUCTION_BITS: [u16; 20] = [
            0xB1, 0xB2, 0xB3, 0xB6, 0xB7, 0xB8, 0xB9, 0xBA, 0xBB, 0xBF, 0xDE,
            0xBE, // #FIXME: This is BKPT
            0xE8, 0xE9, 0xEA, 0xEB, 0xEC, 0xED, 0xEE, 0xEF, // #FIXME: This is BLX
        ];

        for bits in 0..=0xFF {
            let bits = bits as u16;
            for ubits in UNDEFINED_INSTRUCTION_BITS.iter() {
                let instr = (bits & 0x00FF) | (ubits << 8);
                let dis = disasm(instr, 0x0);
                assert_eq!("undef", dis.mnemonic().to_string());
                assert_eq!(format!("0x{instr:04x}"), dis.arguments(0, None).to_string());
                assert_eq!("", dis.comment(0, None).to_string());
            }
        }
    }

    #[test]
    fn disasm_bl() {
        let (setup, bl) = assemble_two("bl 0x1234").unwrap();
        let setup_bytes = setup.to_le_bytes();
        let bl_bytes = bl.to_le_bytes();
        let memory = [setup_bytes[0], setup_bytes[1], bl_bytes[0], bl_bytes[1]];
        let dis = disasm(bl, 0x2);

        assert_eq!("bl", dis.mnemonic().to_string());
        assert_eq!(
            "0x00001234",
            dis.arguments(0x2, Some(&&memory[..])).to_string()
        );
        assert_eq!("", dis.comment(0, None).to_string());

        let dis = disasm(setup, 0x0);
        assert_eq!("bl_setup", dis.mnemonic().to_string());
        assert_eq!("", dis.arguments(0, None).to_string());
        assert_eq!("lr = 0x00001004", dis.comment(0, None).to_string());
    }

    macro_rules! make_test {
        ($name:ident, $source:literal, $mnemonic:literal, $arguments:literal) => {
            #[test]
            fn $name() {
                let asm = assemble_one($source).unwrap();
                let dis = disasm(asm, 0x0);
                assert_eq!($mnemonic, dis.mnemonic().to_string());
                assert_eq!($arguments, dis.arguments(0, None).to_string());
            }
        };

        ($name:ident, $source:literal, $mnemonic:literal, $arguments:literal, $comment:literal) => {
            #[test]
            fn $name() {
                let asm = assemble_one($source).unwrap();
                let dis = disasm(asm, 0x0);
                assert_eq!($mnemonic, dis.mnemonic().to_string());
                assert_eq!($arguments, dis.arguments(0, None).to_string());
                assert_eq!($comment, dis.comment(0, None).to_string());
            }
        };
    }

    macro_rules! make_tests {
        ($([$name:ident, $source:literal, $mnemonic:literal, $arguments:literal $(, $comment:literal)?]),+ $(,)?) => {
            $(make_test!($name, $source, $mnemonic, $arguments $(, $comment)?);)+
        };
    }

    // Move shifted register (imm5)
    #[rustfmt::skip]
    make_tests! {
        [disasm_lsl_imm5, "lsl r0, r1, #12", "lsl", "r0, r1, #12"],
        [disasm_lsr_imm5, "lsr r0, r1, #12", "lsr", "r0, r1, #12"],
        [disasm_asr_imm5, "asr r0, r1, #12", "asr", "r0, r1, #12"],
    }

    // ADD/SUB (imm3)
    #[rustfmt::skip]
    make_tests! {
        [disasm_add_imm3, "add r0, r1, #0x6", "add", "r0, r1, #0x6"],
        [disasm_sub_imm3, "sub r0, r1, #0x6", "sub", "r0, r1, #0x6"],
        [disasm_add_reg3, "add r0, r1, r2", "add", "r0, r1, r2"],
        [disasm_sub_reg3, "sub r0, r1, r2", "sub", "r0, r1, r2"],
    }

    // MOVE/COMPARE/ADD/SUBTRACT (imm8)
    #[rustfmt::skip]
    make_tests! {
        [disasm_mov_imm8, "mov r5, #0xab", "mov", "r5, #0xab"],
        [disasm_cmp_imm8, "cmp r5, #0xab", "cmp", "r5, #0xab"],
        [disasm_add_imm8, "add r5, #0xab", "add", "r5, #0xab"],
        [disasm_sub_imm8, "sub r5, #0xab", "sub", "r5, #0xab"],
    }

    // Data processing
    #[rustfmt::skip]
    make_tests! {
        [disasm_and_alu, "and r1, r2", "and", "r1, r2"],
        [disasm_eor_alu, "eor r1, r2", "eor", "r1, r2"],
        [disasm_lsl_alu, "lsl r1, r2", "lsl", "r1, r2"],
        [disasm_lsr_alu, "lsr r1, r2", "lsr", "r1, r2"],
        [disasm_asr_alu, "asr r1, r2", "asr", "r1, r2"],
        [disasm_adc_alu, "adc r1, r2", "adc", "r1, r2"],
        [disasm_sbc_alu, "sbc r1, r2", "sbc", "r1, r2"],
        [disasm_ror_alu, "ror r1, r2", "ror", "r1, r2"],
        [disasm_tst_alu, "tst r1, r2", "tst", "r1, r2"],
        [disasm_neg_alu, "neg r1, r2", "neg", "r1, r2"],
        [disasm_cmp_alu, "cmp r1, r2", "cmp", "r1, r2"],
        [disasm_cmn_alu, "cmn r1, r2", "cmn", "r1, r2"],
        [disasm_orr_alu, "orr r1, r2", "orr", "r1, r2"],
        [disasm_mul_alu, "mul r1, r2", "mul", "r1, r2"],
        [disasm_bic_alu, "bic r1, r2", "bic", "r1, r2"],
        [disasm_mvn_alu, "mvn r1, r2", "mvn", "r1, r2"],
    }

    // Hi register operations/branch exchange
    #[rustfmt::skip]
    make_tests! {
        [disasm_add_lo_hi_reg, "add r1, r10", "add", "r1, r10"],
        [disasm_add_hi_lo_reg, "add r10, r1", "add", "r10, r1"],
        [disasm_cmp_lo_hi_reg, "cmp r1, r10", "cmp", "r1, r10"],
        [disasm_cmp_hi_lo_reg, "cmp r10, r1", "cmp", "r10, r1"],
        [disasm_mov_lo_hi_reg, "mov r1, r10", "mov", "r1, r10"],
        [disasm_mov_hi_lo_reg, "mov r10, r1", "mov", "r10, r1"],
        [disasm_bx_lo, "bx r1", "bx", "r1"],
        [disasm_bx_hi, "bx r10", "bx", "r10"],
    }

    // PC-relative load
    #[rustfmt::skip]
    make_tests! {
        [disasm_ldr_pc_relative, "ldr r0, [pc, #0x4]", "ldr", "r0, [pc, #0x4]", "r0 = [0x00000008]"],
    }

    // SP-relative load/store
    #[rustfmt::skip]
    make_tests! {
        [disasm_ldr_sp_relative, "ldr r0, [sp, #0x4]", "ldr", "r0, [sp, #0x4]"],
        [disasm_str_sp_relative, "str r0, [sp, #0x4]", "str", "r0, [sp, #0x4]"],
    }

    // LDR/STR (register)
    #[rustfmt::skip]
    make_tests! {
        [disasm_ldr_reg, "ldr r0, [r1, r2]", "ldr", "r0, [r1, r2]"],
        [disasm_str_reg, "str r0, [r1, r2]", "str", "r0, [r1, r2]"],
        [disasm_ldrb_reg, "ldrb r0, [r1, r2]", "ldrb", "r0, [r1, r2]"],
        [disasm_strb_reg, "strb r0, [r1, r2]", "strb", "r0, [r1, r2]"],

        [disasm_ldr_imm7, "ldr r0, [r1, #0x4]", "ldr", "r0, [r1, #0x4]"],
        [disasm_str_imm7, "str r0, [r1, #0x4]", "str", "r0, [r1, #0x4]"],
        [disasm_ldrb_imm5, "ldrb r0, [r1, #0x4]", "ldrb", "r0, [r1, #0x4]"],
        [disasm_strb_imm5, "strb r0, [r1, #0x3]", "strb", "r0, [r1, #0x3]"],

        [disasm_ldrh_reg, "ldrh r0, [r1, r2]", "ldrh", "r0, [r1, r2]"],
        [disasm_ldrsb_reg, "ldrsb r0, [r1, r2]", "ldrsb", "r0, [r1, r2]"],
        [disasm_ldrsh_reg, "ldrsh r0, [r1, r2]", "ldrsh", "r0, [r1, r2]"],
        [disasm_strh_reg, "strh r0, [r1, r2]", "strh", "r0, [r1, r2]"],

        [disasm_ldrh_imm6, "ldrh r0, [r1, #0x4]", "ldrh", "r0, [r1, #0x4]"],
        [disasm_strh_imm6, "strh r0, [r1, #0x4]", "strh", "r0, [r1, #0x4]"],
    }

    // load address
    #[rustfmt::skip]
    make_tests! {
        [disasm_load_address_pc, "add r0, pc, #0x4", "add", "r0, pc, #0x4"],
        [disasm_load_address_sp, "add r0, sp, #0x4", "add", "r0, sp, #0x4"],
    }

    // add offset to stack pointer
    #[rustfmt::skip]
    make_tests! {
        [disasm_add_sp, "add sp, #0x4", "add", "sp, #0x4"],
        [disasm_sub_sp, "sub sp, #0x4", "sub", "sp, #0x4"],
    }

    // push/pop
    #[rustfmt::skip]
    make_tests! {
        [disasm_push, "push {r0,r2-r4,r6}", "push", "{r0,r2-r4,r6}"],
        [disasm_push_lr, "push {r0,r4-r5,lr}", "push", "{r0,r4-r5,lr}"],
        [disasm_pop, "pop {r0,r2-r4,r6}", "pop", "{r0,r2-r4,r6}"],
        [disasm_pop_pc, "pop {r0,r4-r5,pc}", "pop", "{r0,r4-r5,pc}"],
    }

    // ldmia/stmia
    #[rustfmt::skip]
    make_tests! {
        [disasm_ldmia, "ldmia r1!, {r0,r2-r4,r6}", "ldmia", "r1!, {r0,r2-r4,r6}"],
        [disasm_stmia, "stmia r1!, {r0,r2-r4,r6}", "stmia", "r1!, {r0,r2-r4,r6}"],
    }

    // branch
    #[rustfmt::skip]
    make_tests! {
        [disasm_b, "b 0x000000b8", "b", "0x000000b8"],

        [disasm_beq, "beq 0x000000b8", "beq", "0x000000b8"],
        [disasm_bne, "bne 0x000000b8", "bne", "0x000000b8"],
        [disasm_bcs, "bcs 0x000000b8", "bcs", "0x000000b8"],
        [disasm_bcc, "bcc 0x000000b8", "bcc", "0x000000b8"],
        [disasm_bmi, "bmi 0x000000b8", "bmi", "0x000000b8"],
        [disasm_bpl, "bpl 0x000000b8", "bpl", "0x000000b8"],
        [disasm_bvs, "bvs 0x000000b8", "bvs", "0x000000b8"],
        [disasm_bvc, "bvc 0x000000b8", "bvc", "0x000000b8"],
        [disasm_bhi, "bhi 0x000000b8", "bhi", "0x000000b8"],
        [disasm_bls, "bls 0x000000b8", "bls", "0x000000b8"],
        [disasm_bge, "bge 0x000000b8", "bge", "0x000000b8"],
        [disasm_blt, "blt 0x000000b8", "blt", "0x000000b8"],
        [disasm_bgt, "bgt 0x000000b8", "bgt", "0x000000b8"],
        [disasm_ble, "ble 0x000000b8", "ble", "0x000000b8"],
    }

    #[rustfmt::skip]
    make_tests! {
        [disasm_swi, "swi #0x56", "swi", "#0x56"],
    }

    fn assemble(source: &str) -> std::io::Result<Vec<u8>> {
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

        arm_devkit::thumb::assemble(source, linker_script)
    }

    fn assemble_one(source: &str) -> std::io::Result<u16> {
        let assembled = assemble(source)?;
        assert!(assembled.len() >= 2);
        let instr = (assembled[0] as u16) | ((assembled[1] as u16) << 8);
        Ok(instr)
    }

    fn assemble_two(source: &str) -> std::io::Result<(u16, u16)> {
        let assembled = assemble(source)?;
        assert!(assembled.len() >= 4);
        let instr1 = (assembled[0] as u16) | ((assembled[1] as u16) << 8);
        let instr2 = (assembled[2] as u16) | ((assembled[3] as u16) << 8);
        Ok((instr1, instr2))
    }
}
