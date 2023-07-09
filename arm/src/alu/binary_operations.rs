use util::bits::BitOps;

use crate::{ArithmeticShr, CpsrFlag, Registers, RotateRightExtended};

use super::LogicalShr;

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
pub struct MulOp;
pub struct NegOp;

pub struct LslOp;
pub struct LsrOp;
pub struct AsrOp;
pub struct RorOp;
pub struct RrxOp;

impl BinaryOp for AddOp {
    const HAS_RESULT: bool = true;

    fn execute(_registers: &Registers, lhs: u32, rhs: u32) -> u32 {
        lhs.wrapping_add(rhs)
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

    fn execute(registers: &Registers, lhs: u32, rhs: u32) -> u32 {
        let carry = registers.get_flag(CpsrFlag::C);
        lhs.wrapping_add(rhs).wrapping_add(carry as u32)
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

    fn execute(_registers: &Registers, lhs: u32, rhs: u32) -> u32 {
        lhs.wrapping_sub(rhs)
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

    fn execute(registers: &Registers, lhs: u32, rhs: u32) -> u32 {
        let carry = registers.get_flag(CpsrFlag::C);
        lhs.wrapping_sub(rhs).wrapping_sub((!carry) as u32)
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

    fn execute(_registers: &Registers, lhs: u32, rhs: u32) -> u32 {
        // same as SubOp but we swap the order of lhs and rhs
        SubOp::execute(_registers, rhs, lhs)
    }

    fn set_flags(registers: &mut Registers, lhs: u32, rhs: u32, result: u32) {
        // same as SubOp but we swap the order of lhs and rhs
        SubOp::set_flags(registers, rhs, lhs, result)
    }
}

impl BinaryOp for RscOp {
    const HAS_RESULT: bool = true;

    fn execute(_registers: &Registers, lhs: u32, rhs: u32) -> u32 {
        // same as SbcOp but we swap the order of lhs and rhs
        SbcOp::execute(_registers, rhs, lhs)
    }

    fn set_flags(registers: &mut Registers, lhs: u32, rhs: u32, result: u32) {
        // same as SbcOp but we swap the order of lhs and rhs
        SbcOp::set_flags(registers, rhs, lhs, result)
    }
}

impl BinaryOp for AndOp {
    const HAS_RESULT: bool = true;

    fn execute(_registers: &Registers, lhs: u32, rhs: u32) -> u32 {
        lhs & rhs
    }
}

impl BinaryOp for BicOp {
    const HAS_RESULT: bool = true;

    fn execute(_registers: &Registers, lhs: u32, rhs: u32) -> u32 {
        lhs & !rhs
    }
}

impl BinaryOp for CmnOp {
    const HAS_RESULT: bool = false;

    fn execute(_registers: &Registers, lhs: u32, rhs: u32) -> u32 {
        AddOp::execute(_registers, lhs, rhs)
    }

    fn set_flags(registers: &mut Registers, lhs: u32, rhs: u32, result: u32) {
        AddOp::set_flags(registers, lhs, rhs, result)
    }
}

impl BinaryOp for CmpOp {
    const HAS_RESULT: bool = false;

    fn execute(_registers: &Registers, lhs: u32, rhs: u32) -> u32 {
        SubOp::execute(_registers, lhs, rhs)
    }

    fn set_flags(registers: &mut Registers, lhs: u32, rhs: u32, result: u32) {
        SubOp::set_flags(registers, lhs, rhs, result)
    }
}

impl BinaryOp for EorOp {
    const HAS_RESULT: bool = true;

    fn execute(_registers: &Registers, lhs: u32, rhs: u32) -> u32 {
        lhs ^ rhs
    }
}

impl BinaryOp for OrrOp {
    const HAS_RESULT: bool = true;

    fn execute(_registers: &Registers, lhs: u32, rhs: u32) -> u32 {
        lhs | rhs
    }
}

impl BinaryOp for TeqOp {
    const HAS_RESULT: bool = false;

    fn execute(_registers: &Registers, lhs: u32, rhs: u32) -> u32 {
        EorOp::execute(_registers, lhs, rhs)
    }

    fn set_flags(registers: &mut Registers, lhs: u32, rhs: u32, result: u32) {
        EorOp::set_flags(registers, lhs, rhs, result)
    }
}

impl BinaryOp for TstOp {
    const HAS_RESULT: bool = false;

    fn execute(_registers: &Registers, lhs: u32, rhs: u32) -> u32 {
        AndOp::execute(_registers, lhs, rhs)
    }

    fn set_flags(registers: &mut Registers, lhs: u32, rhs: u32, result: u32) {
        AndOp::set_flags(registers, lhs, rhs, result)
    }
}

impl BinaryOp for MovOp {
    const HAS_RESULT: bool = true;

    fn execute(_registers: &Registers, _lhs: u32, rhs: u32) -> u32 {
        rhs
    }
}

impl BinaryOp for MvnOp {
    const HAS_RESULT: bool = true;

    fn execute(_registers: &Registers, _lhs: u32, rhs: u32) -> u32 {
        !rhs
    }
}

impl BinaryOp for LslOp {
    const HAS_RESULT: bool = true;

    fn execute(_registers: &Registers, lhs: u32, rhs: u32) -> u32 {
        if rhs < 32 {
            lhs << rhs
        } else {
            0
        }
    }

    fn get_carry_out(lhs: u32, rhs: u32) -> Option<bool> {
        match rhs.cmp(&32) {
            // If this byte is zero, the unchanged contents of Rm will be used as the second operand,
            // and the old value of the CPSR C flag will be passed on as the shifter carry output.
            std::cmp::Ordering::Less => {
                if rhs != 0 {
                    Some(lhs.get_bit(32 - rhs))
                } else {
                    None
                }
            }

            // LSL by 32 has result zero, carry out equal to bit 0 of Rm.
            std::cmp::Ordering::Equal => Some(lhs.get_bit(0)),

            // LSL by more than 32 has result zero, carry out zero.
            std::cmp::Ordering::Greater => Some(false),
        }
    }
}

impl BinaryOp for LsrOp {
    const HAS_RESULT: bool = true;

    fn execute(_registers: &Registers, lhs: u32, rhs: u32) -> u32 {
        if rhs >= 32 {
            0
        } else {
            lhs.logical_shr(rhs)
        }
    }

    fn get_carry_out(lhs: u32, rhs: u32) -> Option<bool> {
        match rhs.cmp(&32) {
            std::cmp::Ordering::Less => {
                // If this byte is zero, the unchanged contents of Rm will be used as the second operand,
                // and the old value of the CPSR C flag will be passed on as the shifter carry output.
                if rhs > 0 {
                    Some(lhs.get_bit(rhs - 1))
                } else {
                    None
                }
            }

            // LSR by 32 has result zero, carry out equal to bit 31 of Rm.
            std::cmp::Ordering::Equal => Some(lhs.get_bit(31)),

            // LSR by more than 32 has result zero, carry out zero.
            std::cmp::Ordering::Greater => Some(false),
        }
    }

    #[inline]
    fn transform_imm_rhs(rhs: u32) -> u32 {
        // The form of the shift field which might be expected to correspond to LSR #0 is used to encode LSR #32,
        // which has a zero result with bit 31 of Rm as the carry output.
        if rhs == 0 {
            32
        } else {
            rhs
        }
    }
}

impl BinaryOp for AsrOp {
    const HAS_RESULT: bool = true;

    fn execute(_registers: &Registers, lhs: u32, rhs: u32) -> u32 {
        // ASR by 32 or more has result filled with and carry out equal to bit 31 of Rm.
        if rhs >= 32 {
            ((lhs as i32) >> 31) as u32
        } else {
            lhs.arithmetic_shr(rhs)
        }
    }

    fn get_carry_out(lhs: u32, rhs: u32) -> Option<bool> {
        // If this byte is zero, the unchanged contents of Rm will be used as the second operand,
        // and the old value of the CPSR C flag will be passed on as the shifter carry output.
        if rhs == 0 {
            return None;
        }

        // ASR by 32 or more has result filled with and carry out equal to bit 31 of Rm.
        if rhs >= 32 {
            Some(lhs.get_bit(31))
        } else {
            Some(lhs.get_bit(rhs - 1))
        }
    }

    #[inline]
    fn transform_imm_rhs(rhs: u32) -> u32 {
        // The form of the shift field which might be expected to give ASR #0 is used to encode ASR #32.
        if rhs == 0 {
            32
        } else {
            rhs
        }
    }
}

impl BinaryOp for RorOp {
    const HAS_RESULT: bool = true;

    fn execute(_registers: &Registers, lhs: u32, mut rhs: u32) -> u32 {
        // ROR by n where n is greater than 32 will give the same result and carry out as ROR by n-32;
        // therefore repeatedly subtract 32 from n until the amount is in the range 1 to 32.
        while rhs > 32 {
            rhs -= 32;
        }

        // If this byte is zero, the unchanged contents of Rm will be used as the second operand,
        // and the old value of the CPSR C flag will be passed on as the shifter carry output.
        if rhs == 0 {
            lhs
        } else {
            // ROR by 32 has result equal to Rm (so same as rotate_right(0))
            lhs.rotate_right(rhs)
        }
    }

    fn get_carry_out(lhs: u32, mut rhs: u32) -> Option<bool> {
        // ROR by n where n is greater than 32 will give the same result and carry out as ROR by n-32;
        // therefore repeatedly subtract 32 from n until the amount is in the range 1 to 32.
        while rhs > 32 {
            rhs -= 32;
        }

        // ROR by 32 has result equal to Rm, carry out equal to bit 31 of Rm.
        if rhs == 32 {
            Some(lhs.get_bit(31))
        } else if rhs != 0 {
            Some(lhs.get_bit(rhs - 1))
        } else {
            // If this byte is zero, the unchanged contents of Rm will be used as the second operand,
            // and the old value of the CPSR C flag will be passed on as the shifter carry output.
            None
        }
    }
}

impl BinaryOp for RrxOp {
    const HAS_RESULT: bool = true;

    fn execute(registers: &Registers, lhs: u32, _rhs: u32) -> u32 {
        let carry = registers.get_flag(CpsrFlag::C); // have to get this before it's modified
        lhs.rotate_right_extended(carry)
    }

    fn get_carry_out(lhs: u32, _rhs: u32) -> Option<bool> {
        Some(lhs.get_bit(0))
    }
}

impl BinaryOp for MulOp {
    const HAS_RESULT: bool = true;

    fn execute(_registers: &Registers, lhs: u32, rhs: u32) -> u32 {
        lhs.wrapping_mul(rhs)
    }
}

impl BinaryOp for NegOp {
    const HAS_RESULT: bool = true;

    fn execute(_registers: &Registers, _lhs: u32, rhs: u32) -> u32 {
        RsbOp::execute(_registers, rhs, 0)
    }

    fn set_flags(registers: &mut Registers, _lhs: u32, rhs: u32, result: u32) {
        RsbOp::set_flags(registers, rhs, 0, result)
    }
}

pub trait BinaryOp {
    const HAS_RESULT: bool;

    #[must_use]
    fn execute(registers: &Registers, lhs: u32, rhs: u32) -> u32;

    /// Just a shorthand for `if S { Self::set_flags() }`
    fn set_flags_if<const S: bool>(registers: &mut Registers, lhs: u32, rhs: u32, result: u32) {
        if S {
            Self::set_flags(registers, lhs, rhs, result)
        }
    }

    #[inline(always)]
    fn get_carry_out(_lhs: u32, _rhs: u32) -> Option<bool> {
        None
    }

    fn set_flags(registers: &mut Registers, lhs: u32, rhs: u32, result: u32) {
        if let Some(carry) = Self::get_carry_out(lhs, rhs) {
            registers.put_flag(CpsrFlag::C, carry);
        }
        registers.put_flag(CpsrFlag::N, (result >> 31) & 1);
        registers.put_flag(CpsrFlag::Z, result == 0);
    }

    /// Some immediate forms of the shift operations use #0 to encode
    /// #32 (or RRX). This method is called to do this transformation.
    #[inline]
    fn transform_imm_rhs(rhs: u32) -> u32 {
        rhs
    }
}
