#[macro_use]
pub mod common;

use arm::CpsrFlag;

use crate::common::operands::{imm3, imm32};

#[test]
pub fn test_lsl_imm() {
    // The least significant **discarded** bit becomes the shifter carry output which may
    // be latched into the C bit of the CPSR.
    let (cpu, _mem) = thumb! {"
        ldr r1, =#0x82000000
        lsl r0, r1, #1
    "};
    assert_eq!(cpu.registers.read(0), 0x04000000);
    assert!(cpu.registers.get_flag(CpsrFlag::C));
}

#[test]
pub fn test_lsl_imm_by_zero() {
    // LSL #0 is a special case, where the shifter carry out is the old value of the CPSR C
    // flag. The contents of Rm are used directly as the second operand.
    let (cpu, _mem) = thumb! {"
        ldr r1, =#0x82000000
        lsl r0, r1, #1
        ldr r1, =#0x80000000
        lsl r0, r1, #0
    "};
    assert_eq!(cpu.registers.read(0), 0x80000000);
    assert!(cpu.registers.get_flag(CpsrFlag::C));
}

#[test]
pub fn test_lsr_imm() {
    let (cpu, _mem) = thumb! {"
        mov r1, #5
        lsr r0, r1, #1
    "};
    assert_eq!(cpu.registers.read(0), 2);
    assert!(cpu.registers.get_flag(CpsrFlag::C));
}

#[test]
pub fn test_lsr_imm_by_zero() {
    // Logical shift right zero is redundant as it is the same as logical shift left zero,
    // so the assembler will convert LSR #0 (and ASR #0 and ROR #0) into LSL #0, and allow LSR #32 to be specified.
    let (cpu, _mem) = thumb! {"
        ldr r1, =#0x82000000
        lsl r0, r1, #1
        ldr r1, =#0x00000001
        lsr r0, r1, #0
    "};
    assert_eq!(cpu.registers.read(0), 0x00000001);
    assert!(cpu.registers.get_flag(CpsrFlag::C));
}

#[test]
pub fn test_lsr_imm_by_32() {
    // The form of the shift field which might be expected to correspond to LSR #0 is used to encode LSR #32,
    // which has a zero result with bit 31 of Rm as the carry output.
    let (cpu, _mem) = thumb! {"
        mov r0, #42
        ldr r1, =#0x80000000
        lsr r0, r1, #32
    "};
    assert_eq!(cpu.registers.read(0), 0);
    assert!(cpu.registers.get_flag(CpsrFlag::C));
}

#[test]
pub fn test_asr_positive() {
    let (cpu, _mem) = thumb! {"
        ldr r1, =#0x60000000
        asr r0, r1, #1
    "};
    assert_eq!(cpu.registers.read(0), 0x30000000);
    assert!(!cpu.registers.get_flag(CpsrFlag::C));
}

#[test]
pub fn test_asr_negative() {
    let (cpu, _mem) = thumb! {"
        ldr r1, =#0xE0000000
        asr r0, r1, #1
    "};
    assert_eq!(cpu.registers.read(0), 0xF0000000);
    assert!(!cpu.registers.get_flag(CpsrFlag::C));
}

#[test]
pub fn test_asr_imm() {
    let (cpu, _mem) = thumb! {"
        ldr r1, =#0xE0000001
        asr r0, r1, #1
    "};
    assert_eq!(cpu.registers.read(0), 0xF0000000);
    assert!(cpu.registers.get_flag(CpsrFlag::C));
}

#[test]
pub fn test_asr_imm_by_zero() {
    // Logical shift right zero is redundant as it is the same as logical shift left zero,
    // so the assembler will convert LSR #0 (and ASR #0 and ROR #0) into LSL #0, and allow LSR #32 to be specified.
    let (cpu, _mem) = thumb! {"
        ldr r1, =#0x82000000
        lsl r0, r1, #1
        ldr r1, =#0x00000001
        asr r0, r1, #0
    "};
    assert_eq!(cpu.registers.read(0), 0x00000001);
    assert!(cpu.registers.get_flag(CpsrFlag::C));
}

#[test]
pub fn test_asr_imm_by_32() {
    // The form of the shift field which might be expected to give ASR #0 is used to encode
    // ASR #32. Bit 31 of Rm is again used as the carry output, and each bit of operand 2 is
    // also equal to bit 31 of Rm. The result is therefore all ones or all zeros, according to the
    // value of bit 31 of Rm
    let (cpu, _mem) = thumb! {"
        ldr r1, =#0x80000000 
        asr r0, r1, #32
    "};
    assert_eq!(cpu.registers.read(0), 0xFFFFFFFF);
    assert!(cpu.registers.get_flag(CpsrFlag::C));

    let (cpu, _mem) = thumb! {"
        ldr r1, =#0x70000000 
        asr r0, r1, #32
    "};
    assert_eq!(cpu.registers.read(0), 0x00000000);
    assert!(!cpu.registers.get_flag(CpsrFlag::C));
}

test_combinations! {
    #[test]
    fn test_load_value_into_register(value in imm32()) {
        let (cpu, _mem) = arm! {"ldr r0, =#{value}"};
        assert_eq!(cpu.registers.read(0), value as u32);
    }

    #[test]
    fn test_add_imm3(lhs in imm32(), rhs in imm3()) {
        println!("lhs = {lhs}, rhs = {rhs}");
        let (cpu, _mem) = thumb! {"
                ldr r1, =#{lhs}
                add r0, r1, #{rhs}
            "};

        let expected_result = lhs.wrapping_add(rhs);
        let expected_n = expected_result < 0;
        let expected_z = expected_result == 0;
        let expected_c = (lhs as u32).overflowing_add(rhs as u32).1;
        let expected_v = lhs.overflowing_add(rhs).1;

        assert_eq!(cpu.registers.read(0), expected_result as u32);
        assert_eq!(cpu.registers.get_flag(CpsrFlag::N), expected_n);
        assert_eq!(cpu.registers.get_flag(CpsrFlag::Z), expected_z);
        assert_eq!(cpu.registers.get_flag(CpsrFlag::C), expected_c);
        assert_eq!(cpu.registers.get_flag(CpsrFlag::V), expected_v);
    }

    #[test]
    fn test_add_reg3(lhs in imm32(), rhs in imm32()) {
        println!("lhs = {lhs}, rhs = {rhs}");
        let (cpu, _mem) = thumb! {"
                ldr r1, =#{lhs}
                ldr r2, =#{rhs}
                add r0, r1, r2
            "};

        let expected_result = lhs.wrapping_add(rhs);
        let expected_n = expected_result < 0;
        let expected_z = expected_result == 0;
        let expected_c = (lhs as u32).overflowing_add(rhs as u32).1;
        let expected_v = lhs.overflowing_add(rhs).1;

        assert_eq!(cpu.registers.read(0), expected_result as u32);
        assert_eq!(cpu.registers.get_flag(CpsrFlag::N), expected_n);
        assert_eq!(cpu.registers.get_flag(CpsrFlag::Z), expected_z);
        assert_eq!(cpu.registers.get_flag(CpsrFlag::C), expected_c);
        assert_eq!(cpu.registers.get_flag(CpsrFlag::V), expected_v);
    }

    #[test]
    fn test_sub_imm3(lhs in imm32(), rhs in imm3()) {
        let (cpu, _mem) = thumb! {"
                ldr r1, =#{lhs}
                sub r0, r1, #{rhs}
            "};

        let expected_result = lhs.wrapping_sub(rhs);
        let expected_n = expected_result < 0;
        let expected_z = expected_result == 0;
        let expected_c = (lhs as u32) >= (rhs as u32);
        let expected_v = lhs.overflowing_sub(rhs).1;

        assert_eq!(cpu.registers.read(0), expected_result as u32);
        assert_eq!(cpu.registers.get_flag(CpsrFlag::N), expected_n);
        assert_eq!(cpu.registers.get_flag(CpsrFlag::Z), expected_z);
        assert_eq!(cpu.registers.get_flag(CpsrFlag::C), expected_c);
        assert_eq!(cpu.registers.get_flag(CpsrFlag::V), expected_v);
    }

    #[test]
    fn test_sub_reg3(lhs in imm32(), rhs in imm32()) {
        let (cpu, _mem) = thumb! {"
                ldr r1, =#{lhs}
                ldr r2, =#{rhs}
                sub r0, r1, r2
            "};

        let expected_result = lhs.wrapping_sub(rhs);
        let expected_n = expected_result < 0;
        let expected_z = expected_result == 0;
        let expected_c = (lhs as u32) >= (rhs as u32);
        let expected_v = lhs.overflowing_sub(rhs).1;

        assert_eq!(cpu.registers.read(0), expected_result as u32);
        assert_eq!(cpu.registers.get_flag(CpsrFlag::N), expected_n);
        assert_eq!(cpu.registers.get_flag(CpsrFlag::Z), expected_z);
        assert_eq!(cpu.registers.get_flag(CpsrFlag::C), expected_c);
        assert_eq!(cpu.registers.get_flag(CpsrFlag::V), expected_v);
    }
}
