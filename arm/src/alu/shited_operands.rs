use util::bits::BitOps;

use crate::{alu::LslOp, CpsrFlag, Cycles, Registers};

use super::{AsrOp, BinaryOp, LsrOp, RorOp, RrxOp};

// Operand 2 Extractors for ARM dataprocessing instructions:
// Immediate operand (simple rotate)
pub struct ImmOp2;

/// Logical shift left by immdiate
pub struct LliOp2;
/// Logical shift left by register
pub struct LlrOp2;
/// Logical shift right by immediate
pub struct LriOp2;
/// Logical shift right by register
pub struct LrrOp2;
/// Arithmetic shift right by immediate
pub struct AriOp2;
/// Arithmetic shift right by register
pub struct ArrOp2;
/// Rotate right by immediate
pub struct RriOp2;
/// Rotate right by register
pub struct RrrOp2;

#[inline(always)]
fn get_op2_using_binop<B, const S: bool>(lhs: u32, rhs: u32, registers: &mut Registers) -> u32
where
    B: BinaryOp,
{
    // NOTE:    We have to make sure to execute the instruction before modifying
    //          the flags so that RRX works correctly.
    let result = B::execute(registers, lhs, rhs);
    if let (Some(carry), true) = (B::get_carry_out(lhs, rhs), S) {
        registers.put_flag(CpsrFlag::C, carry);
    }
    result
}

impl ExtractOp2 for ImmOp2 {
    const IS_REGISTER_SHIFT: bool = false;

    fn extract<const S: bool>(instr: u32, registers: &mut Registers) -> u32 {
        let (imm, rot) = Self::get_operands(instr, registers);
        imm.rotate_right(rot)
    }

    fn get_operands(instr: u32, _registers: &Registers) -> (u32, u32) {
        let imm = instr.get_bit_range(0..=7);
        let rot = instr.get_bit_range(8..=11) * 2;
        (imm, rot)
    }
}

impl ExtractOp2 for LliOp2 {
    const IS_REGISTER_SHIFT: bool = false;

    fn extract<const S: bool>(instr: u32, registers: &mut Registers) -> u32 {
        let (lhs, mut rhs) = Self::get_operands(instr, registers);
        rhs = LslOp::transform_imm_rhs(rhs);
        get_op2_using_binop::<LslOp, S>(lhs, rhs, registers)
    }
}

impl ExtractOp2 for LlrOp2 {
    const IS_REGISTER_SHIFT: bool = true;

    fn extract<const S: bool>(instr: u32, registers: &mut Registers) -> u32 {
        let (lhs, rhs) = Self::get_operands(instr, registers);
        get_op2_using_binop::<LslOp, S>(lhs, rhs, registers)
    }
}

impl ExtractOp2 for LriOp2 {
    const IS_REGISTER_SHIFT: bool = false;

    fn extract<const S: bool>(instr: u32, registers: &mut Registers) -> u32 {
        let (lhs, mut rhs) = Self::get_operands(instr, registers);
        rhs = LsrOp::transform_imm_rhs(rhs);
        get_op2_using_binop::<LsrOp, S>(lhs, rhs, registers)
    }
}

impl ExtractOp2 for LrrOp2 {
    const IS_REGISTER_SHIFT: bool = true;

    fn extract<const S: bool>(instr: u32, registers: &mut Registers) -> u32 {
        let (lhs, rhs) = Self::get_operands(instr, registers);
        get_op2_using_binop::<LsrOp, S>(lhs, rhs, registers)
    }
}

impl ExtractOp2 for AriOp2 {
    const IS_REGISTER_SHIFT: bool = false;

    fn extract<const S: bool>(instr: u32, registers: &mut Registers) -> u32 {
        let (lhs, mut rhs) = Self::get_operands(instr, registers);
        rhs = AsrOp::transform_imm_rhs(rhs);
        get_op2_using_binop::<AsrOp, S>(lhs, rhs, registers)
    }
}

impl ExtractOp2 for ArrOp2 {
    const IS_REGISTER_SHIFT: bool = true;

    fn extract<const S: bool>(instr: u32, registers: &mut Registers) -> u32 {
        let (lhs, rhs) = Self::get_operands(instr, registers);
        get_op2_using_binop::<AsrOp, S>(lhs, rhs, registers)
    }
}

impl ExtractOp2 for RriOp2 {
    const IS_REGISTER_SHIFT: bool = false;

    fn extract<const S: bool>(instr: u32, registers: &mut Registers) -> u32 {
        let (lhs, rhs) = Self::get_operands(instr, registers);
        if rhs == 0 {
            get_op2_using_binop::<RrxOp, S>(lhs, rhs, registers)
        } else {
            get_op2_using_binop::<RorOp, S>(lhs, rhs, registers)
        }
    }
}

impl ExtractOp2 for RrrOp2 {
    const IS_REGISTER_SHIFT: bool = true;

    fn extract<const S: bool>(instr: u32, registers: &mut Registers) -> u32 {
        let (lhs, rhs) = Self::get_operands(instr, registers);
        get_op2_using_binop::<RorOp, S>(lhs, rhs, registers)
    }
}

pub trait ExtractOp2 {
    const IS_REGISTER_SHIFT: bool;

    /// Extracts the second operand for ARM dataprocessing instructions
    /// from an opcode. Updates CPU flags if `S` (S flag used by ARM dataprocessing operations)
    /// is true.
    fn extract<const S: bool>(instr: u32, registers: &mut Registers) -> u32;

    fn stall() -> Cycles {
        if Self::IS_REGISTER_SHIFT {
            Cycles::one()
        } else {
            Cycles::zero()
        }
    }

    // FIXME This should probably be set to #[inline] or #[inline(always)]
    //       Looked at some of the assembly being generated for arm_dataproc and
    //       it is resulting in a call. Should also look into annotating the
    //       various extract and set_flags functions.
    fn get_operands(instr: u32, registers: &Registers) -> (u32, u32) {
        if Self::IS_REGISTER_SHIFT {
            let rm = instr.get_bit_range(0..=3);
            let rs = instr.get_bit_range(8..=11);

            // When using R15 as operand (Rm or Rn), the returned value
            // depends on the instruction: PC+12 if I=0,R=1 (shift by register),
            // otherwise PC+8 (shift by immediate).
            let lhs_offset = if rm == 15 { 4 } else { 0 };

            let lhs = registers.read(rm).wrapping_add(lhs_offset);
            let rhs = registers.read(rs) & 0xFF; // only the lower 8bits are used

            (lhs, rhs)
        } else {
            let lhs = registers.read(instr.get_bit_range(0..=3));
            let rhs = instr.get_bit_range(7..=11);

            (lhs, rhs)
        }
    }
}
