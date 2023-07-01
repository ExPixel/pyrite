mod binary_operations;
pub mod multiply;
mod psr;
mod shited_operands;

pub use binary_operations::*;
pub use psr::*;
pub use shited_operands::*;

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
