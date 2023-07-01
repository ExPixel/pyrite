use crate::{CpsrFlag, Cycles, Registers};

#[inline]
pub fn internal_multiply_cycles(rhs: u32) -> Cycles {
    let xor = rhs ^ ((rhs as i32) >> 31) as u32;

    if (xor & 0xFFFFFF00) == 0 {
        // m = 1, if bits [32:8] of the multiplier operand are all zero or all one
        1u32.into()
    } else if (xor & 0xFFFF0000) == 0 {
        // m = 2, if bits [32:16] of the multiplier operand are all zero or all one.
        2u32.into()
    } else if (xor & 0xFF000000) == 0 {
        // m = 3, if bits [32:24] of the multiplier operand are all zero or all one.
        3u32.into()
    } else {
        // m = 4, in all other cases
        4u32.into()
    }
}

#[inline]
pub fn set_multiply_flags(result: u32, registers: &mut Registers) {
    registers.put_flag(CpsrFlag::N, (result >> 31) & 1);
    registers.put_flag(CpsrFlag::Z, result == 0);
}
