use util::bits::BitOps;

use crate::{CpsrFlag, Cycles, Registers};

// Binary Ops:
pub struct AdcOp;
pub struct AddOp;
pub struct AndOp;
pub struct BicOp;
pub struct CmnOp;
pub struct CmpOp;
pub struct EorOp;
pub struct MovOp;
pub struct MvnOp;
pub struct OrrOp;
pub struct RsbOp;
pub struct RscOp;
pub struct SbcOp;
pub struct SubOp;
pub struct TeqOp;
pub struct TstOp;

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

impl BinaryOp for AddOp {
    const HAS_RESULT: bool = true;

    fn execute<const S: bool>(registers: &mut Registers, lhs: u32, rhs: u32) -> u32 {
        let result = lhs.wrapping_add(rhs);
        Self::set_flags_if::<S>(registers, lhs, rhs, result);
        result
    }

    fn set_flags(registers: &mut Registers, lhs: u32, rhs: u32, result: u32) {
        registers.put_flag(CpsrFlag::N, (result >> 31) & 1);
        registers.put_flag(CpsrFlag::Z, result == 0);

        let (_, carry) = lhs.overflowing_add(rhs);
        let (_, overflow) = (lhs as i32).overflowing_add(rhs as i32);

        registers.put_flag(CpsrFlag::C, carry);
        registers.put_flag(CpsrFlag::V, overflow);
    }
}

impl BinaryOp for AdcOp {
    const HAS_RESULT: bool = true;

    fn execute<const S: bool>(registers: &mut Registers, lhs: u32, rhs: u32) -> u32 {
        let carry = registers.get_flag(CpsrFlag::C);
        let result = lhs.wrapping_add(rhs).wrapping_add(carry as u32);
        Self::set_flags_if::<S>(registers, lhs, rhs, result);
        result
    }

    fn set_flags(registers: &mut Registers, lhs: u32, rhs: u32, result: u32) {
        registers.put_flag(CpsrFlag::N, (result >> 31) & 1);
        registers.put_flag(CpsrFlag::Z, result == 0);

        let carry = registers.get_flag(CpsrFlag::C);

        let (res_0, carry_0) = lhs.overflowing_add(rhs);
        let (_, overflow_0) = (lhs as i32).overflowing_add(rhs as i32);

        let (_, carry_1) = res_0.overflowing_add(carry as u32);
        let (_, overflow_1) = (res_0 as i32).overflowing_add(carry as i32);

        registers.put_flag(CpsrFlag::C, carry_0 | carry_1);
        registers.put_flag(CpsrFlag::V, overflow_0 | overflow_1);
    }
}

impl BinaryOp for SubOp {
    const HAS_RESULT: bool = true;

    fn execute<const S: bool>(registers: &mut Registers, lhs: u32, rhs: u32) -> u32 {
        let result = lhs.wrapping_sub(rhs);
        Self::set_flags_if::<S>(registers, lhs, rhs, result);
        result
    }

    fn set_flags(registers: &mut Registers, lhs: u32, rhs: u32, result: u32) {
        registers.put_flag(CpsrFlag::N, (result >> 31) & 1);
        registers.put_flag(CpsrFlag::Z, result == 0);

        let (_, overflow) = (lhs as i32).overflowing_sub(rhs as i32);

        // #NOTE The concept of a borrow is not the same in ARM as it is in x86.
        //       while in x86 the borrow flag is set if lhs < rhs, in ARM
        //       if is set if lhs >= rhs (when the result of a subtraction is positive).
        registers.put_flag(CpsrFlag::C, lhs >= rhs);
        registers.put_flag(CpsrFlag::V, overflow);
    }
}

impl BinaryOp for SbcOp {
    const HAS_RESULT: bool = true;

    fn execute<const S: bool>(registers: &mut Registers, lhs: u32, rhs: u32) -> u32 {
        let carry = registers.get_flag(CpsrFlag::C);
        let result = lhs.wrapping_sub(rhs).wrapping_sub((!carry) as u32);
        Self::set_flags_if::<S>(registers, lhs, rhs, result);
        result
    }

    fn set_flags(registers: &mut Registers, lhs: u32, rhs: u32, result: u32) {
        registers.put_flag(CpsrFlag::N, (result >> 31) & 1);
        registers.put_flag(CpsrFlag::Z, result == 0);

        let carry = registers.get_flag(CpsrFlag::C);

        // #NOTE The concept of a borrow is not the same in ARM as it is in x86.
        //       while in x86 the borrow flag is set if lhs < rhs, in ARM
        //       if is set if lhs >= rhs (when the result of a subtraction is positive).
        registers.put_flag(CpsrFlag::C, (lhs as u64) >= (rhs as u64 + (!carry) as u64));
        registers.put_flag(
            CpsrFlag::V,
            (((lhs >> 31) ^ rhs) & ((lhs >> 31) ^ result)) != 0,
        );
    }
}

impl BinaryOp for RsbOp {
    const HAS_RESULT: bool = true;

    fn execute<const S: bool>(registers: &mut Registers, lhs: u32, rhs: u32) -> u32 {
        // same as SubOp but we swap the order of lhs and rhs
        SubOp::execute::<S>(registers, rhs, lhs)
    }

    fn set_flags(registers: &mut Registers, lhs: u32, rhs: u32, result: u32) {
        // same as SubOp but we swap the order of lhs and rhs
        SubOp::set_flags(registers, rhs, lhs, result)
    }
}

impl BinaryOp for RscOp {
    const HAS_RESULT: bool = true;

    fn execute<const S: bool>(registers: &mut Registers, lhs: u32, rhs: u32) -> u32 {
        // same as SbcOp but we swap the order of lhs and rhs
        SbcOp::execute::<S>(registers, rhs, lhs)
    }

    fn set_flags(registers: &mut Registers, lhs: u32, rhs: u32, result: u32) {
        // same as SbcOp but we swap the order of lhs and rhs
        SbcOp::set_flags(registers, rhs, lhs, result)
    }
}

impl BinaryOp for AndOp {
    const HAS_RESULT: bool = true;

    fn execute<const S: bool>(registers: &mut Registers, lhs: u32, rhs: u32) -> u32 {
        let result = lhs & rhs;
        Self::set_flags_if::<S>(registers, lhs, rhs, result);
        result
    }
}

impl BinaryOp for BicOp {
    const HAS_RESULT: bool = true;

    fn execute<const S: bool>(registers: &mut Registers, lhs: u32, rhs: u32) -> u32 {
        let result = lhs & !rhs;
        Self::set_flags_if::<S>(registers, lhs, rhs, result);
        result
    }
}

impl BinaryOp for CmnOp {
    const HAS_RESULT: bool = false;

    fn execute<const S: bool>(registers: &mut Registers, lhs: u32, rhs: u32) -> u32 {
        AddOp::execute::<S>(registers, lhs, rhs)
    }

    fn set_flags(registers: &mut Registers, lhs: u32, rhs: u32, result: u32) {
        AddOp::set_flags(registers, lhs, rhs, result)
    }
}

impl BinaryOp for CmpOp {
    const HAS_RESULT: bool = false;

    fn execute<const S: bool>(registers: &mut Registers, lhs: u32, rhs: u32) -> u32 {
        SubOp::execute::<S>(registers, lhs, rhs)
    }

    fn set_flags(registers: &mut Registers, lhs: u32, rhs: u32, result: u32) {
        SubOp::set_flags(registers, lhs, rhs, result)
    }
}

impl BinaryOp for EorOp {
    const HAS_RESULT: bool = true;

    fn execute<const S: bool>(registers: &mut Registers, lhs: u32, rhs: u32) -> u32 {
        let result = lhs ^ rhs;
        Self::set_flags_if::<S>(registers, lhs, rhs, result);
        result
    }
}

impl BinaryOp for OrrOp {
    const HAS_RESULT: bool = true;

    fn execute<const S: bool>(registers: &mut Registers, lhs: u32, rhs: u32) -> u32 {
        let result = lhs | rhs;
        Self::set_flags_if::<S>(registers, lhs, rhs, result);
        result
    }
}

impl BinaryOp for TeqOp {
    const HAS_RESULT: bool = false;

    fn execute<const S: bool>(registers: &mut Registers, lhs: u32, rhs: u32) -> u32 {
        EorOp::execute::<S>(registers, lhs, rhs)
    }

    fn set_flags(registers: &mut Registers, lhs: u32, rhs: u32, result: u32) {
        EorOp::set_flags(registers, lhs, rhs, result)
    }
}

impl BinaryOp for TstOp {
    const HAS_RESULT: bool = false;

    fn execute<const S: bool>(registers: &mut Registers, lhs: u32, rhs: u32) -> u32 {
        AndOp::execute::<S>(registers, lhs, rhs)
    }

    fn set_flags(registers: &mut Registers, lhs: u32, rhs: u32, result: u32) {
        AndOp::set_flags(registers, lhs, rhs, result)
    }
}

impl BinaryOp for MovOp {
    const HAS_RESULT: bool = true;

    fn execute<const S: bool>(registers: &mut Registers, lhs: u32, rhs: u32) -> u32 {
        let result = rhs;
        Self::set_flags_if::<S>(registers, lhs, rhs, result);
        result
    }
}

impl BinaryOp for MvnOp {
    const HAS_RESULT: bool = true;

    fn execute<const S: bool>(registers: &mut Registers, lhs: u32, rhs: u32) -> u32 {
        let result = !rhs;
        Self::set_flags_if::<S>(registers, lhs, rhs, result);
        result
    }
}

impl ExtractOp2 for ImmOp2 {
    const IS_REGISTER_SHIFT: bool = false;

    fn extract<const S: bool>(instr: u32, registers: &mut Registers) -> u32 {
        let (imm, rot) = Self::get_operands(instr, registers);
        Self::set_flags_if::<S>(registers, imm, rot);
        imm.rotate_right(rot)
    }

    fn set_flags(_registers: &mut Registers, _lhs: u32, _rhs: u32) {
        // NOP
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
        let (lhs, rhs) = Self::get_operands(instr, registers);
        Self::set_flags_if::<S>(registers, lhs, rhs);
        lhs << rhs
    }

    fn set_flags(registers: &mut Registers, lhs: u32, rhs: u32) {
        // LSL #0 is a special case, where the shifter carry out is the old value of the CPSR C
        // flag. The contents of Rm are used directly as the second operand.
        if rhs != 0 {
            registers.put_flag(CpsrFlag::C, (lhs >> (32 - rhs)) & 1)
        }
    }
}

impl ExtractOp2 for LlrOp2 {
    const IS_REGISTER_SHIFT: bool = true;

    fn extract<const S: bool>(instr: u32, registers: &mut Registers) -> u32 {
        let (lhs, rhs) = Self::get_operands(instr, registers);
        Self::set_flags_if::<S>(registers, lhs, rhs);
        if rhs < 32 {
            lhs << rhs
        } else {
            0
        }
    }

    fn set_flags(registers: &mut Registers, lhs: u32, rhs: u32) {
        match rhs.cmp(&32) {
            // If this byte is zero, the unchanged contents of Rm will be used as the second operand,
            // and the old value of the CPSR C flag will be passed on as the shifter carry output.
            std::cmp::Ordering::Less => {
                if rhs != 0 {
                    registers.put_flag(CpsrFlag::C, (lhs >> (32 - rhs)) & 1)
                }
            }

            // LSL by 32 has result zero, carry out equal to bit 0 of Rm.
            std::cmp::Ordering::Equal => registers.put_flag(CpsrFlag::C, lhs & 1),

            // LSL by more than 32 has result zero, carry out zero.
            std::cmp::Ordering::Greater => registers.clear_flag(CpsrFlag::C),
        }
    }
}

impl ExtractOp2 for LriOp2 {
    const IS_REGISTER_SHIFT: bool = false;

    fn extract<const S: bool>(instr: u32, registers: &mut Registers) -> u32 {
        let (lhs, rhs) = Self::get_operands(instr, registers);
        Self::set_flags_if::<S>(registers, lhs, rhs);

        // The form of the shift field which might be expected to correspond to LSR #0 is used to encode LSR #32,
        // which has a zero result with bit 31 of Rm as the carry output.
        if rhs == 0 {
            0
        } else {
            lhs.logical_shr(rhs)
        }
    }

    fn set_flags(registers: &mut Registers, lhs: u32, rhs: u32) {
        // The form of the shift field which might be expected to correspond to LSR #0 is used to encode LSR #32,
        // which has a zero result with bit 31 of Rm as the carry output.
        if rhs == 0 {
            registers.put_flag(CpsrFlag::C, lhs.get_bit(31))
        } else {
            registers.put_flag(CpsrFlag::C, (lhs >> (rhs - 1)) & 1);
        }
    }
}

impl ExtractOp2 for LrrOp2 {
    const IS_REGISTER_SHIFT: bool = true;

    fn extract<const S: bool>(instr: u32, registers: &mut Registers) -> u32 {
        let (lhs, rhs) = Self::get_operands(instr, registers);
        Self::set_flags_if::<S>(registers, lhs, rhs);

        if rhs >= 32 {
            0
        } else {
            lhs.logical_shr(rhs)
        }
    }

    fn set_flags(registers: &mut Registers, lhs: u32, rhs: u32) {
        match rhs.cmp(&32) {
            std::cmp::Ordering::Less => {
                // If this byte is zero, the unchanged contents of Rm will be used as the second operand,
                // and the old value of the CPSR C flag will be passed on as the shifter carry output.
                if rhs > 0 {
                    registers.put_flag(CpsrFlag::C, (lhs >> (rhs - 1)) & 1);
                }
            }
            // LSR by 32 has result zero, carry out equal to bit 31 of Rm.
            std::cmp::Ordering::Equal => {
                registers.put_flag(CpsrFlag::C, lhs.get_bit(31));
            }

            // LSR by more than 32 has result zero, carry out zero.
            std::cmp::Ordering::Greater => {
                registers.clear_flag(CpsrFlag::C);
            }
        }
    }
}

impl ExtractOp2 for AriOp2 {
    const IS_REGISTER_SHIFT: bool = false;

    fn extract<const S: bool>(instr: u32, registers: &mut Registers) -> u32 {
        let (lhs, rhs) = Self::get_operands(instr, registers);
        Self::set_flags_if::<S>(registers, lhs, rhs);

        // The form of the shift field which might be expected to give ASR #0 is used to encode ASR #32.
        // Bit 31 of Rm is again used as the carry output, and each bit of operand 2 is also equal to bit 31 of Rm.
        // The result is therefore all ones or all zeros, according to the value of bit 31 of Rm.
        if rhs == 0 {
            ((lhs as i32) >> 31) as u32
        } else {
            lhs.arithmetic_shr(rhs)
        }
    }

    fn set_flags(registers: &mut Registers, lhs: u32, rhs: u32) {
        // The form of the shift field which might be expected to give ASR #0 is used to encode ASR #32.
        // Bit 31 of Rm is again used as the carry output, and each bit of operand 2 is also equal to bit 31 of Rm.
        // The result is therefore all ones or all zeros, according to the value of bit 31 of Rm.
        if rhs == 0 {
            registers.put_flag(CpsrFlag::C, lhs.get_bit(31));
        } else {
            registers.put_flag(CpsrFlag::C, (lhs >> (rhs - 1)) & 1);
        }
    }
}

impl ExtractOp2 for ArrOp2 {
    const IS_REGISTER_SHIFT: bool = true;

    fn extract<const S: bool>(instr: u32, registers: &mut Registers) -> u32 {
        let (lhs, rhs) = Self::get_operands(instr, registers);
        Self::set_flags_if::<S>(registers, lhs, rhs);

        // ASR by 32 or more has result filled with and carry out equal to bit 31 of Rm.
        if rhs >= 32 {
            ((lhs as i32) >> 31) as u32
        } else {
            lhs.arithmetic_shr(rhs)
        }
    }

    fn set_flags(registers: &mut Registers, lhs: u32, rhs: u32) {
        // If this byte is zero, the unchanged contents of Rm will be used as the second operand,
        // and the old value of the CPSR C flag will be passed on as the shifter carry output.
        if rhs == 0 {
            return;
        }

        // ASR by 32 or more has result filled with and carry out equal to bit 31 of Rm.
        if rhs >= 32 {
            registers.put_flag(CpsrFlag::C, lhs.get_bit(31));
        } else {
            registers.put_flag(CpsrFlag::C, (lhs >> (rhs - 1)) & 1);
        }
    }
}

impl ExtractOp2 for RriOp2 {
    const IS_REGISTER_SHIFT: bool = false;

    fn extract<const S: bool>(instr: u32, registers: &mut Registers) -> u32 {
        let (lhs, rhs) = Self::get_operands(instr, registers);
        let carry = registers.get_flag(CpsrFlag::C); // have to get this before it's modified
        Self::set_flags_if::<S>(registers, lhs, rhs);

        // The form of the shift field which might be expected to give ROR #0
        // is used to encode a special function of the barrel shifter, rotate right extended (RRX)
        if rhs == 0 {
            lhs.rotate_right_extended(carry)
        } else {
            lhs.rotate_right(rhs)
        }
    }

    fn set_flags(registers: &mut Registers, lhs: u32, rhs: u32) {
        if rhs == 0 {
            // The form of the shift field which might be expected to give ROR #0
            // is used to encode a special function of the barrel shifter, rotate right extended (RRX)
            registers.put_flag(CpsrFlag::C, lhs & 1);
        } else {
            registers.put_flag(CpsrFlag::C, (lhs >> (rhs - 1)) & 1);
        }
    }
}

impl ExtractOp2 for RrrOp2 {
    const IS_REGISTER_SHIFT: bool = true;

    fn extract<const S: bool>(instr: u32, registers: &mut Registers) -> u32 {
        let (lhs, mut rhs) = Self::get_operands(instr, registers);

        // ROR by n where n is greater than 32 will give the same result and carry out as ROR by n-32;
        // therefore repeatedly subtract 32 from n until the amount is in the range 1 to 32.
        while rhs > 32 {
            rhs -= 32;
        }

        Self::set_flags_if::<S>(registers, lhs, rhs);

        // If this byte is zero, the unchanged contents of Rm will be used as the second operand,
        // and the old value of the CPSR C flag will be passed on as the shifter carry output.
        if rhs == 0 {
            lhs
        } else {
            // ROR by 32 has result equal to Rm (so same as rotate_right(0))
            lhs.rotate_right(rhs & 31)
        }
    }

    #[inline(always)]
    fn set_flags(registers: &mut Registers, lhs: u32, mut rhs: u32) {
        // ROR by n where n is greater than 32 will give the same result and carry out as ROR by n-32;
        // therefore repeatedly subtract 32 from n until the amount is in the range 1 to 32.
        while rhs > 32 {
            rhs -= 32;
        }

        // ROR by 32 has result equal to Rm, carry out equal to bit 31 of Rm.
        if rhs == 32 {
            registers.put_flag(CpsrFlag::C, lhs.get_bit(31))
        } else if rhs != 0 {
            registers.put_flag(CpsrFlag::C, (lhs >> (rhs - 1)) & 1);
        }
    }
}

pub trait BinaryOp {
    const HAS_RESULT: bool;

    /// Executes this binary operation and updates CPU flags
    /// if `S` (S flag used by ARM dataprocessing operations) is true.
    #[must_use]
    fn execute<const S: bool>(registers: &mut Registers, lhs: u32, rhs: u32) -> u32;

    /// Just a shorthand for `if S { Self::set_flags() }`
    fn set_flags_if<const S: bool>(registers: &mut Registers, lhs: u32, rhs: u32, result: u32) {
        if S {
            Self::set_flags(registers, lhs, rhs, result)
        }
    }

    fn set_flags(registers: &mut Registers, _lhs: u32, _rhs: u32, result: u32) {
        registers.put_flag(CpsrFlag::N, (result >> 31) & 1);
        registers.put_flag(CpsrFlag::Z, result == 0);
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

    /// Just a shorthand for `if S { Self::set_flags() }`
    #[inline(always)]
    fn set_flags_if<const S: bool>(registers: &mut Registers, lhs: u32, rhs: u32) {
        if S {
            Self::set_flags(registers, lhs, rhs)
        }
    }

    fn set_flags(registers: &mut Registers, _lhs: u32, _rhs: u32);

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

impl RotateRightExtended for u32 {
    type Output = Self;

    #[inline]
    fn rotate_right_extended(self, carry: bool) -> Self::Output {
        let carry = carry as u32;
        (self >> 1) | (carry << 31)
    }
}

impl ArithmeticShr for u32 {
    type Output = Self;

    #[inline(always)]
    fn arithmetic_shr(self, shift: Self) -> Self::Output {
        ((self as i32) >> shift) as u32
    }
}

impl LogicalShr for u32 {
    type Output = Self;

    #[inline(always)]
    fn logical_shr(self, shift: Self) -> Self::Output {
        self >> shift
    }
}

pub trait RotateRightExtended {
    type Output;
    fn rotate_right_extended(self, carry: bool) -> Self::Output;
}

pub trait ArithmeticShr<Rhs = Self> {
    type Output;
    fn arithmetic_shr(self, shift: Rhs) -> Self::Output;
}

pub trait LogicalShr<Rhs = Self> {
    type Output;
    fn logical_shr(self, shift: Rhs) -> Self::Output;
}
