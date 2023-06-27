use arm::CpsrFlag;
use proptest::prelude::*;

#[macro_use]
pub mod common;

#[test]
pub fn test_b() {
    let (cpu, _mem) = arm! {"
        mov     r0, #5
        b       _exit
        mov     r0, #2
    "};
    assert_eq!(cpu.registers.read(0), 5);
}

#[test]
pub fn test_bl() {
    let (cpu, _mem) = arm! {"
        mov     r0, #5
        bl      _exit
        mov     r0, #2
    "};
    assert_eq!(cpu.registers.read(0), 5);
    assert_eq!(cpu.registers.read(14), 8);
}

#[test]
pub fn test_dataproc_imm_operand() {
    let (cpu, _mem) = arm!("mov r0, #0x12000000");
    assert_eq!(cpu.registers.read(0), 0x12000000);
}

#[test]
pub fn test_dataproc_r15_operand() {
    // should only be 8 bytes ahead since this is not a register shift:
    let (cpu, _mem) = arm!("mov r0, r15");
    assert_eq!(cpu.registers.read(0), 8);

    // should be 12 bytes ahead for a register shift:
    let (cpu, _mem) = arm!("mov r0, r15, lsl r1");
    assert_eq!(cpu.registers.read(0), 12);
}

#[test]
pub fn test_lsl() {
    let (cpu, _mem) = arm! {"
        mov r1, #1
        mov r0, r1, LSL #1
    "};
    assert_eq!(cpu.registers.read(0), 2);
    assert!(!cpu.registers.get_flag(CpsrFlag::C));
}

#[test]
pub fn test_lsls_imm() {
    // The least significant **discarded** bit becomes the shifter carry output which may
    // be latched into the C bit of the CPSR.
    let (cpu, _mem) = arm! {"
        mov     r1, #0x82000000
        movs    r0, r1, LSL #1
    "};
    assert_eq!(cpu.registers.read(0), 0x04000000);
    assert!(cpu.registers.get_flag(CpsrFlag::C));
}

#[test]
pub fn test_lsls_reg() {
    // The least significant **discarded** bit becomes the shifter carry output which may
    // be latched into the C bit of the CPSR.
    let (cpu, _mem) = arm! {"
        mov     r1, #0x82000000
        mov     r2, #1
        movs    r0, r1, LSL r2
    "};
    assert_eq!(cpu.registers.read(0), 0x04000000);
    assert!(cpu.registers.get_flag(CpsrFlag::C));
}

#[test]
pub fn test_lsls_imm_by_zero() {
    // LSL #0 is a special case, where the shifter carry out is the old value of the CPSR C
    // flag. The contents of Rm are used directly as the second operand.
    let (cpu, _mem) = arm! {"
        mov     r1, #0x82000000
        movs    r0, r1, LSL #1
        mov     r1, #0x80000000
        movs    r0, r1, LSL #0
    "};
    assert_eq!(cpu.registers.read(0), 0x80000000);
    assert!(cpu.registers.get_flag(CpsrFlag::C));
}

#[test]
pub fn test_lsls_reg_by_zero() {
    // LSL #0 is a special case, where the shifter carry out is the old value of the CPSR C
    // flag. The contents of Rm are used directly as the second operand.
    let (cpu, _mem) = arm! {"
        mov     r1, #0x82000000
        movs    r0, r1, LSL #1
        mov     r1, #0x80000000
        mov     r2, #0
        movs    r0, r1, LSL r2
    "};
    assert_eq!(cpu.registers.read(0), 0x80000000);
    assert!(cpu.registers.get_flag(CpsrFlag::C));
}

#[test]
pub fn test_lsls_reg_by_32() {
    // LSL by 32 has result zero, carry out equal to bit 0 of Rm.
    let (cpu, _mem) = arm! {"
        mov     r1, #0x00000001
        mov     r2, #32
        movs    r0, r1, LSL r2
    "};
    assert_eq!(cpu.registers.read(0), 0);
    assert!(cpu.registers.get_flag(CpsrFlag::C));
}

#[test]
pub fn test_lsls_reg_by_more_than_32() {
    // LSL by more than 32 has result zero, carry out zero.
    let (cpu, _mem) = arm! {"
        mov     r1, #0x00000001
        mov     r2, #33
        movs    r0, r1, LSL r2
    "};
    assert_eq!(cpu.registers.read(0), 0);
    assert!(!cpu.registers.get_flag(CpsrFlag::C));
}

#[test]
pub fn test_lsr() {
    let (cpu, _mem) = arm! {"
        mov r1, #4
        mov r0, r1, LSR #1
    "};
    assert_eq!(cpu.registers.read(0), 2);
    assert!(!cpu.registers.get_flag(CpsrFlag::C));
}

#[test]
pub fn test_lsrs_imm() {
    let (cpu, _mem) = arm! {"
        mov     r1, #5
        movs    r0, r1, LSR #1
    "};
    assert_eq!(cpu.registers.read(0), 2);
    assert!(cpu.registers.get_flag(CpsrFlag::C));
}

#[test]
pub fn test_lsrs_reg() {
    let (cpu, _mem) = arm! {"
        mov     r1, #5
        mov     r2, #1
        movs    r0, r1, LSR r2
    "};
    assert_eq!(cpu.registers.read(0), 2);
    assert!(cpu.registers.get_flag(CpsrFlag::C));
}

#[test]
pub fn test_lsrs_imm_by_zero() {
    // Logical shift right zero is redundant as it is the same as logical shift left zero,
    // so the assembler will convert LSR #0 (and ASR #0 and ROR #0) into LSL #0, and allow LSR #32 to be specified.
    let (cpu, _mem) = arm! {"
        mov     r1, #0x82000000
        movs    r0, r1, LSL #1
        mov     r1, #0x00000001
        movs    r0, r1, LSR #0
    "};
    assert_eq!(cpu.registers.read(0), 0x00000001);
    assert!(cpu.registers.get_flag(CpsrFlag::C));
}

#[test]
pub fn test_lsrs_reg_by_zero() {
    // Logical shift right zero is redundant as it is the same as logical shift left zero,
    // so the assembler will convert LSR #0 (and ASR #0 and ROR #0) into LSL #0, and allow LSR #32 to be specified.
    let (cpu, _mem) = arm! {"
        mov     r1, #0x82000000
        movs    r0, r1, LSL #1
        mov     r1, #0x00000001
        mov     r2, #0
        movs    r0, r1, LSR r2
    "};
    assert_eq!(cpu.registers.read(0), 0x00000001);
    assert!(cpu.registers.get_flag(CpsrFlag::C));
}

#[test]
pub fn test_lsrs_imm_by_32() {
    // The form of the shift field which might be expected to correspond to LSR #0 is used to encode LSR #32,
    // which has a zero result with bit 31 of Rm as the carry output.
    let (cpu, _mem) = arm! {"
        mov     r0, #42
        mov     r1, #0x80000000
        movs    r0, r1, LSR #32
    "};
    assert_eq!(cpu.registers.read(0), 0);
    assert!(cpu.registers.get_flag(CpsrFlag::C));
}

#[test]
pub fn test_lsrs_reg_by_32() {
    // The form of the shift field which might be expected to correspond to LSR #0 is used to encode LSR #32,
    // which has a zero result with bit 31 of Rm as the carry output.
    let (cpu, _mem) = arm! {"
        mov     r0, #42
        mov     r1, #0x80000000
        mov     r2, #32
        movs    r0, r1, LSR r2
    "};
    assert_eq!(cpu.registers.read(0), 0);
    assert!(cpu.registers.get_flag(CpsrFlag::C));
}

#[test]
pub fn test_lsrs_reg_by_more_than_32() {
    // LSR by more than 32 has result zero, carry out zero.
    let (cpu, _mem) = arm! {"
        mov     r0, #42
        mov     r1, #0x80000000
        mov     r2, #33
        movs    r0, r1, LSR r2
    "};
    assert_eq!(cpu.registers.read(0), 0);
    assert!(!cpu.registers.get_flag(CpsrFlag::C));
}

#[test]
pub fn test_asr_positive() {
    let (cpu, _mem) = arm! {"
        mov r1, #0x60000000
        mov r0, r1, ASR #1
    "};
    assert_eq!(cpu.registers.read(0), 0x30000000);
    assert!(!cpu.registers.get_flag(CpsrFlag::C));
}

#[test]
pub fn test_asr_negative() {
    let (cpu, _mem) = arm! {"
        mov r1, #0xE0000000
        mov r0, r1, ASR #1
    "};
    assert_eq!(cpu.registers.read(0), 0xF0000000);
    assert!(!cpu.registers.get_flag(CpsrFlag::C));
}

#[test]
pub fn test_asrs_imm() {
    let (cpu, _mem) = arm! {"
        mov     r1, #0xE0000001
        movs    r0, r1, ASR #1
    "};
    assert_eq!(cpu.registers.read(0), 0xF0000000);
    assert!(cpu.registers.get_flag(CpsrFlag::C));
}

#[test]
pub fn test_asrs_reg() {
    let (cpu, _mem) = arm! {"
        mov     r1, #0xE0000001
        mov     r2, #1
        movs    r0, r1, ASR r2
    "};
    assert_eq!(cpu.registers.read(0), 0xF0000000);
    assert!(cpu.registers.get_flag(CpsrFlag::C));
}

#[test]
pub fn test_asrs_imm_by_zero() {
    // Logical shift right zero is redundant as it is the same as logical shift left zero,
    // so the assembler will convert LSR #0 (and ASR #0 and ROR #0) into LSL #0, and allow LSR #32 to be specified.
    let (cpu, _mem) = arm! {"
        mov     r1, #0x82000000
        movs    r0, r1, LSL #1
        mov     r1, #0x00000001
        movs    r0, r1, ASR #0
    "};
    assert_eq!(cpu.registers.read(0), 0x00000001);
    assert!(cpu.registers.get_flag(CpsrFlag::C));
}

#[test]
pub fn test_asrs_reg_by_zero() {
    // Logical shift right zero is redundant as it is the same as logical shift left zero,
    // so the assembler will convert LSR #0 (and ASR #0 and ROR #0) into LSL #0, and allow
    // LSR #32 to be specified.
    let (cpu, _mem) = arm! {"
        mov     r1, #0x82000000
        movs    r0, r1, LSL #1
        mov     r1, #0x00000001
        mov     r2, #0
        movs    r0, r1, ASR r2
    "};
    assert_eq!(cpu.registers.read(0), 0x00000001);
    assert!(cpu.registers.get_flag(CpsrFlag::C));
}

#[test]
pub fn test_asrs_imm_by_32() {
    // The form of the shift field which might be expected to give ASR #0 is used to encode
    // ASR #32. Bit 31 of Rm is again used as the carry output, and each bit of operand 2 is
    // also equal to bit 31 of Rm. The result is therefore all ones or all zeros, according to the
    // value of bit 31 of Rm
    let (cpu, _mem) = arm! {"
        mov     r1, #0x80000000 
        movs    r0, r1, ASR #32
    "};
    assert_eq!(cpu.registers.read(0), 0xFFFFFFFF);
    assert!(cpu.registers.get_flag(CpsrFlag::C));

    let (cpu, _mem) = arm! {"
        mov     r1, #0x70000000 
        movs    r0, r1, ASR #32
    "};
    assert_eq!(cpu.registers.read(0), 0x00000000);
    assert!(!cpu.registers.get_flag(CpsrFlag::C));
}

#[test]
pub fn test_asrs_reg_by_32() {
    // ASR by 32 or more has result filled with and carry out equal to bit 31 of Rm.
    let (cpu, _mem) = arm! {"
        mov     r1, #0x80000000 
        mov     r2, #32
        movs    r0, r1, ASR r2
    "};
    assert_eq!(cpu.registers.read(0), 0xFFFFFFFF);
    assert!(cpu.registers.get_flag(CpsrFlag::C));

    let (cpu, _mem) = arm! {"
        mov     r1, #0x70000000 
        mov     r2, #32
        movs    r0, r1, ASR r2
    "};
    assert_eq!(cpu.registers.read(0), 0x00000000);
    assert!(!cpu.registers.get_flag(CpsrFlag::C));
}

#[test]
pub fn test_asrs_reg_by_more_than_32() {
    // ASR by 32 or more has result filled with and carry out equal to bit 31 of Rm.
    let (cpu, _mem) = arm! {"
        mov     r1, #0x80000000 
        mov     r2, #33
        movs    r0, r1, ASR r2
    "};
    assert_eq!(cpu.registers.read(0), 0xFFFFFFFF);
    assert!(cpu.registers.get_flag(CpsrFlag::C));

    let (cpu, _mem) = arm! {"
        mov     r1, #0x70000000
        mov     r2, #33
        movs    r0, r1, ASR r2
    "};
    assert_eq!(cpu.registers.read(0), 0x00000000);
    assert!(!cpu.registers.get_flag(CpsrFlag::C));
}

#[test]
pub fn test_ror() {
    let (cpu, _mem) = arm! {"
        mov r1, #0x0000000F
        mov r0, r1, ROR #4
    "};
    assert_eq!(cpu.registers.read(0), 0xF0000000);
    assert!(!cpu.registers.get_flag(CpsrFlag::C));
}

#[test]
pub fn test_rors_imm() {
    let (cpu, _mem) = arm! {"
        mov     r1, #0x0000000F
        movs    r0, r1, ROR #4
    "};
    assert_eq!(cpu.registers.read(0), 0xF0000000);
    assert!(cpu.registers.get_flag(CpsrFlag::C));
}

#[test]
pub fn test_rors_reg() {
    let (cpu, _mem) = arm! {"
        mov     r1, #0x0000000F
        mov     r2, #4
        movs    r0, r1, ROR r2
    "};
    assert_eq!(cpu.registers.read(0), 0xF0000000);
    assert!(cpu.registers.get_flag(CpsrFlag::C));
}

#[test]
pub fn test_rors_imm_by_zero() {
    // Logical shift right zero is redundant as it is the same as logical shift left zero,
    // so the assembler will convert LSR #0 (and ASR #0 and ROR #0) into LSL #0, and allow LSR #32 to be specified.
    let (cpu, _mem) = arm! {"
        mov     r1, #0x82000000
        movs    r0, r1, LSL #1
        mov     r1, #0x00000001
        movs    r0, r1, ROR #0
    "};
    assert_eq!(cpu.registers.read(0), 0x00000001);
    assert!(cpu.registers.get_flag(CpsrFlag::C));
}

#[test]
pub fn test_rors_reg_by_zero() {
    // The unchanged contents of Rm will be used as the second operand,
    // and the old value of the CPSR C flag will be passed on as the shifter carry output.
    let (cpu, _mem) = arm! {"
        mov     r1, #0x82000000
        movs    r0, r1, LSL #1
        mov     r1, #0x00000001
        mov     r2, #0
        movs    r0, r1, ROR r2
    "};
    assert_eq!(cpu.registers.read(0), 0x00000001);
    assert!(cpu.registers.get_flag(CpsrFlag::C));
}

#[test]
pub fn test_rors_reg_by_32() {
    // ROR by 32 has result equal to Rm, carry out equal to bit 31 of Rm.
    let (cpu, _mem) = arm! {"
        mov     r1, #0x80000000
        movs    r0, r1, LSL #1
        mov     r1, #0x80000000
        mov     r2, #32
        movs    r0, r1, ROR r2
    "};
    assert_eq!(cpu.registers.read(0), 0x80000000);
    assert!(cpu.registers.get_flag(CpsrFlag::C));

    // ROR by 32 has result equal to Rm, carry out equal to bit 31 of Rm.
    let (cpu, _mem) = arm! {"
        mov     r1, #0x80000000
        movs    r0, r1, LSL #1
        mov     r1, #0x70000000
        mov     r2, #32
        movs    r0, r1, ROR r2
    "};
    assert_eq!(cpu.registers.read(0), 0x70000000);
    assert!(!cpu.registers.get_flag(CpsrFlag::C));
}

#[test]
pub fn test_rors_reg_by_more_than_32() {
    // ROR by 32 has result equal to Rm, carry out equal to bit 31 of Rm.
    let (cpu, _mem) = arm! {"
        mov     r1, #0x80000000
        movs    r0, r1, LSL #1
        mov     r1, #0x00000008
        mov     r2, #36
        movs    r0, r1, ROR r2
    "};
    assert_eq!(cpu.registers.read(0), 0x80000000);
    assert!(cpu.registers.get_flag(CpsrFlag::C));

    // ROR by 32 has result equal to Rm, carry out equal to bit 31 of Rm.
    let (cpu, _mem) = arm! {"
        mov     r1, #0x80000000
        movs    r0, r1, LSL #1
        mov     r1, #0x00000007
        mov     r2, #36
        movs    r0, r1, ROR r2
    "};
    assert_eq!(cpu.registers.read(0), 0x70000000);
    assert!(!cpu.registers.get_flag(CpsrFlag::C));
}

#[test]
pub fn test_rrx_carry_clear() {
    let (cpu, _mem) = arm! {"
        mov r1, #0x10000001
        mov r0, r1, RRX
    "};
    assert_eq!(cpu.registers.read(0), 0x08000000);
    assert!(!cpu.registers.get_flag(CpsrFlag::C));
}

#[test]
pub fn test_rrx_carry_set() {
    let (cpu, _mem) = arm! {"
        mov     r1, #0x80000000
        movs    r0, r1, LSL #1
        mov     r1, #0x10000001
        mov     r0, r1, RRX
    "};
    assert_eq!(cpu.registers.read(0), 0x88000000);
    assert!(cpu.registers.get_flag(CpsrFlag::C));
}

#[test]
pub fn test_rrxs_carry_clear() {
    let (cpu, _mem) = arm! {"
        mov     r1, #0x10000001
        movs    r0, r1, RRX
    "};
    assert_eq!(cpu.registers.read(0), 0x08000000);
    assert!(cpu.registers.get_flag(CpsrFlag::C));

    let (cpu, _mem) = arm! {"
        mov     r1, #0x10000000
        movs    r0, r1, RRX
    "};
    assert_eq!(cpu.registers.read(0), 0x08000000);
    assert!(!cpu.registers.get_flag(CpsrFlag::C));
}

#[test]
pub fn test_rrxs_carry_set() {
    let (cpu, _mem) = arm! {"
        mov     r1, #0x80000000
        movs    r0, r1, LSL #1
        mov     r1, #0x10000001
        movs    r0, r1, RRX
    "};
    assert_eq!(cpu.registers.read(0), 0x88000000);
    assert!(cpu.registers.get_flag(CpsrFlag::C));

    let (cpu, _mem) = arm! {"
        mov     r1, #0x80000000
        movs    r0, r1, LSL #1
        mov     r1, #0x10000000
        movs    r0, r1, RRX
    "};
    assert_eq!(cpu.registers.read(0), 0x88000000);
    assert!(!cpu.registers.get_flag(CpsrFlag::C));
}

#[test]
pub fn test_mov() {
    let (cpu, _mem) = arm! {"
        mov     r1, #42
    "};
    assert_eq!(cpu.registers.read(1), 42);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn test_load_value_into_register(value: u32) {
        let (cpu, _mem) = arm! {"ldr r0, =#{value}"};
        prop_assert_eq!(cpu.registers.read(0), value);
    }

    #[test]
    fn test_adds(lhs in operand(), rhs in operand()) {
        let (cpu, _mem) = arm! {"
            ldr     r1, =#{lhs}
            ldr     r2, =#{rhs}
            adds    r0, r1, r2
        "};

        let expected_result = lhs.wrapping_add(rhs);
        let expected_n = (expected_result as i32) < 0;
        let expected_z = expected_result == 0;
        let expected_c = lhs.overflowing_add(rhs).1;
        let expected_v = (lhs as i32).overflowing_add(rhs as i32).1;

        prop_assert_eq!(cpu.registers.read(0), expected_result);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::N), expected_n);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::Z), expected_z);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::C), expected_c);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::V), expected_v);
    }

    #[test]
    fn test_adcs(lhs in operand(), rhs in operand(), initial_carry: bool) {
        let (cpu, _mem) = if initial_carry {
            arm! {"
                ldr     r0, =0x80000000
                movs    r0, r0, lsl #1  @ set carry

                ldr     r1, =#{lhs}
                ldr     r2, =#{rhs}
                adcs    r0, r1, r2
            "}
        } else {
            arm! {"
                ldr     r1, =#{lhs}
                ldr     r2, =#{rhs}
                adcs    r0, r1, r2
            "}
        };

        let expected_result = lhs.wrapping_add(rhs).wrapping_add(initial_carry as u32);
        let expected_n = (expected_result as i32) < 0;
        let expected_z = expected_result == 0;
        let expected_c = lhs.overflowing_add(rhs).1 |
            lhs.wrapping_add(rhs).overflowing_add(initial_carry as u32).1;
        let expected_v = (lhs as i32).overflowing_add(rhs as i32).1 |
            (lhs.wrapping_add(rhs) as i32).overflowing_add(initial_carry as i32).1;

        prop_assert_eq!(cpu.registers.read(0), expected_result);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::N), expected_n);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::Z), expected_z);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::C), expected_c);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::V), expected_v);
    }

    #[test]
    fn test_subs(lhs in operand(), rhs in operand()) {
        let (cpu, _mem) = arm! {"
            ldr     r1, =#{lhs}
            ldr     r2, =#{rhs}
            subs    r0, r1, r2
        "};

        let expected_result = lhs.wrapping_sub(rhs);
        let expected_n = (expected_result as i32) < 0;
        let expected_z = expected_result == 0;
        let expected_c = lhs >= rhs;
        let expected_v = (lhs as i32).overflowing_sub(rhs as i32).1;

        prop_assert_eq!(cpu.registers.read(0), expected_result);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::N), expected_n);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::Z), expected_z);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::C), expected_c);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::V), expected_v);
    }

    #[test]
    fn test_sbcs(lhs in operand(), rhs in operand(), initial_carry: bool) {
        let (cpu, _mem) = if initial_carry {
            arm! {"
                ldr     r0, =0x80000000
                movs    r0, r0, lsl #1  @ set carry

                ldr     r1, =#{lhs}
                ldr     r2, =#{rhs}
                sbcs    r0, r1, r2
            "}
        } else {
            arm! {"
                ldr     r1, =#{lhs}
                ldr     r2, =#{rhs}
                sbcs    r0, r1, r2
            "}
        };

        let expected_result = lhs.wrapping_sub(rhs).wrapping_sub(!initial_carry as u32);
        let expected_n = (expected_result as i32) < 0;
        let expected_z = expected_result == 0;
        let expected_c = (lhs as u64) >= (rhs as u64 + (!initial_carry) as u64);
        let expected_v = (((lhs >> 31) ^ rhs) & ((lhs >> 31) ^ expected_result)) != 0;

        prop_assert_eq!(cpu.registers.read(0), expected_result);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::N), expected_n);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::Z), expected_z);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::C), expected_c);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::V), expected_v);
    }

    #[test]
    fn test_rsbs(lhs in operand(), rhs in operand()) {
        let (cpu, _mem) = arm! {"
            ldr     r1, =#{lhs}
            ldr     r2, =#{rhs}
            rsbs    r0, r1, r2
        "};

        let expected_result = rhs.wrapping_sub(lhs);
        let expected_n = (expected_result as i32) < 0;
        let expected_z = expected_result == 0;
        let expected_c = rhs >= lhs;
        let expected_v = (rhs as i32).overflowing_sub(lhs as i32).1;

        prop_assert_eq!(cpu.registers.read(0), expected_result);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::N), expected_n);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::Z), expected_z);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::C), expected_c);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::V), expected_v);
    }

    #[test]
    fn test_rscs(lhs in operand(), rhs in operand(), initial_carry: bool) {
        let (cpu, _mem) = if initial_carry {
            arm! {"
                ldr     r0, =0x80000000
                movs    r0, r0, lsl #1  @ set carry

                ldr     r1, =#{lhs}
                ldr     r2, =#{rhs}
                rscs    r0, r1, r2
            "}
        } else {
            arm! {"
                ldr     r1, =#{lhs}
                ldr     r2, =#{rhs}
                rscs    r0, r1, r2
            "}
        };

        let expected_result = rhs.wrapping_sub(lhs).wrapping_sub(!initial_carry as u32);
        let expected_n = (expected_result as i32) < 0;
        let expected_z = expected_result == 0;
        let expected_c = (rhs as u64) >= (lhs as u64 + (!initial_carry) as u64);
        let expected_v = (((rhs >> 31) ^ lhs) & ((rhs >> 31) ^ expected_result)) != 0;

        prop_assert_eq!(cpu.registers.read(0), expected_result);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::N), expected_n);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::Z), expected_z);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::C), expected_c);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::V), expected_v);
    }

    #[test]
    fn test_movs(lhs in operand()) {
        let (cpu, _mem) = arm! {"
            ldr     r1, =#{lhs}
            movs    r0, r1
        "};

        let expected_result = lhs;
        let expected_n = (expected_result as i32) < 0;
        let expected_z = expected_result == 0;

        prop_assert_eq!(cpu.registers.read(0), expected_result);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::N), expected_n);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::Z), expected_z);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::C), false);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::V), false);
    }

    #[test]
    fn test_mvns(lhs in operand()) {
        let (cpu, _mem) = arm! {"
            ldr     r1, =#{lhs}
            mvns    r0, r1
        "};

        let expected_result = !lhs;
        let expected_n = (expected_result as i32) < 0;
        let expected_z = expected_result == 0;

        prop_assert_eq!(cpu.registers.read(0), expected_result);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::N), expected_n);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::Z), expected_z);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::C), false);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::V), false);
    }

    #[test]
    fn test_ands(lhs in operand(), rhs in operand()) {
        let (cpu, _mem) = arm! {"
            ldr     r1, =#{lhs}
            ldr     r2, =#{rhs}
            ands    r0, r1, r2
        "};

        let expected_result = lhs & rhs;
        let expected_n = (expected_result as i32) < 0;
        let expected_z = expected_result == 0;

        prop_assert_eq!(cpu.registers.read(0), expected_result);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::N), expected_n);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::Z), expected_z);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::C), false);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::V), false);
    }

    #[test]
    fn test_bics(lhs in operand(), rhs in operand()) {
        let (cpu, _mem) = arm! {"
            ldr     r1, =#{lhs}
            ldr     r2, =#{rhs}
            bics    r0, r1, r2
        "};

        let expected_result = lhs & !rhs;
        let expected_n = (expected_result as i32) < 0;
        let expected_z = expected_result == 0;

        prop_assert_eq!(cpu.registers.read(0), expected_result);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::N), expected_n);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::Z), expected_z);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::C), false);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::V), false);
    }

    #[test]
    fn test_orrs(lhs in operand(), rhs in operand()) {
        let (cpu, _mem) = arm! {"
            ldr     r1, =#{lhs}
            ldr     r2, =#{rhs}
            orrs    r0, r1, r2
        "};

        let expected_result = lhs | rhs;
        let expected_n = (expected_result as i32) < 0;
        let expected_z = expected_result == 0;

        prop_assert_eq!(cpu.registers.read(0), expected_result);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::N), expected_n);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::Z), expected_z);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::C), false);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::V), false);
    }

    #[test]
    fn test_eors(lhs in operand(), rhs in operand()) {
        let (cpu, _mem) = arm! {"
            ldr     r1, =#{lhs}
            ldr     r2, =#{rhs}
            eors    r0, r1, r2
        "};

        let expected_result = lhs ^ rhs;
        let expected_n = (expected_result as i32) < 0;
        let expected_z = expected_result == 0;

        prop_assert_eq!(cpu.registers.read(0), expected_result);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::N), expected_n);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::Z), expected_z);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::C), false);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::V), false);
    }

    #[test]
    fn test_cmp(lhs in operand(), rhs in operand()) {
        let (cpu, _mem) = arm! {"
            ldr     r1, =#{lhs}
            ldr     r2, =#{rhs}
            cmp     r1, r2
        "};

        let expected_result = lhs.wrapping_sub(rhs);
        let expected_n = (expected_result as i32) < 0;
        let expected_z = expected_result == 0;
        let expected_c = lhs >= rhs;
        let expected_v = (lhs as i32).overflowing_sub(rhs as i32).1;

        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::N), expected_n);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::Z), expected_z);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::C), expected_c);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::V), expected_v);
    }

    #[test]
    fn test_cmn(lhs in operand(), rhs in operand()) {
        let (cpu, _mem) = arm! {"
            ldr     r1, =#{lhs}
            ldr     r2, =#{rhs}
            cmn     r1, r2
        "};

        let expected_result = lhs.wrapping_add(rhs);
        let expected_n = (expected_result as i32) < 0;
        let expected_z = expected_result == 0;
        let expected_c = lhs.overflowing_add(rhs).1;
        let expected_v = (lhs as i32).overflowing_add(rhs as i32).1;

        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::N), expected_n);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::Z), expected_z);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::C), expected_c);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::V), expected_v);
    }

    #[test]
    fn test_tsts(lhs in operand(), rhs in operand()) {
        let (cpu, _mem) = arm! {"
            ldr     r1, =#{lhs}
            ldr     r2, =#{rhs}
            tsts    r1, r2
        "};

        let expected_result = lhs & rhs;
        let expected_n = (expected_result as i32) < 0;
        let expected_z = expected_result == 0;

        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::N), expected_n);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::Z), expected_z);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::C), false);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::V), false);
    }

    #[test]
    fn test_teqs(lhs in operand(), rhs in operand()) {
        let (cpu, _mem) = arm! {"
            ldr     r1, =#{lhs}
            ldr     r2, =#{rhs}
            teqs    r1, r2
        "};

        let expected_result = lhs ^ rhs;
        let expected_n = (expected_result as i32) < 0;
        let expected_z = expected_result == 0;

        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::N), expected_n);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::Z), expected_z);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::C), false);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::V), false);
    }
}

#[test]
pub fn test_ldr() {
    let (cpu, _mem) = arm! {"
        ldr r1, =deadbeef
        mov r2, r1
        ldr r0, [r1]
    .data
    deadbeef:
        .word 0xDEADBEEF
    "};
    assert_eq!(cpu.registers.read(1), cpu.registers.read(2));
    assert_eq!(cpu.registers.read(0), 0xDEADBEEF);
}

#[test]
pub fn test_ldr_post_index() {
    let (cpu, _mem) = arm! {"
        ldr r1, =deadbeef
        mov r2, r1
        ldr r0, [r1], #4
    .data
    deadbeef:
        .word 0xDEADBEEF
        .word 0xAABBCCDD
    "};
    assert_eq!(cpu.registers.read(1), cpu.registers.read(2).wrapping_add(4));
    assert_eq!(cpu.registers.read(0), 0xDEADBEEF);
}

#[test]
pub fn test_ldr_pre_increment() {
    let (cpu, _mem) = arm! {"
        ldr r1, =deadbeef
        mov r2, r1
        ldr r0, [r1, #4]!
    .data
    deadbeef:
        .word 0xDEADBEEF
        .word 0xAABBCCDD
    "};
    assert_eq!(cpu.registers.read(1), cpu.registers.read(2).wrapping_add(4));
    assert_eq!(cpu.registers.read(0), 0xAABBCCDD);
}

#[test]
pub fn test_ldr_pre_decrement() {
    let (cpu, _mem) = arm! {"
        ldr r1, =deadbeef
        mov r2, r1
        ldr r0, [r1, #-4]!
    .data
        .word 0xDEADBEEF
    deadbeef:
        .word 0xAABBCCDD
    "};
    assert_eq!(cpu.registers.read(1), cpu.registers.read(2).wrapping_sub(4));
    assert_eq!(cpu.registers.read(0), 0xDEADBEEF);
}

#[test]
pub fn test_ldr_lsl() {
    let (cpu, _mem) = arm! {"
        mov r1, #0
        ldr r2, =0xDEADBEEF
        ldr r0, [r1, r2, lsl #4]!
    "};
    assert_eq!(cpu.registers.read(1), 0xEADBEEF0);
}

#[test]
pub fn test_ldr_lsr() {
    let (cpu, _mem) = arm! {"
        mov r1, #0
        ldr r2, =0xDEADBEEF
        ldr r0, [r1, r2, lsr #4]!
    "};
    assert_eq!(cpu.registers.read(1), 0x0DEADBEE);
}

#[test]
pub fn test_ldr_asr() {
    let (cpu, _mem) = arm! {"
        mov r1, #0
        ldr r2, =0xDEADBEEF
        ldr r0, [r1, r2, asr #4]!
    "};
    assert_eq!(cpu.registers.read(1), 0xFDEADBEE);
}

#[test]
pub fn test_ldr_ror() {
    let (cpu, _mem) = arm! {"
        mov r1, #0
        ldr r2, =0xDEADBEEF
        ldr r0, [r1, r2, ror #4]!
    "};
    assert_eq!(cpu.registers.read(1), 0xFDEADBEE);
}

fn operand() -> impl Strategy<Value = u32> {
    const VALUES: &[u32] = &[
        0, 1, 2, 0x00BEEF00, 0x7FFFFFFF, 0xFFFFFFFC, 0xFFFFFFFE, 0xFFFFFFFF,
    ];

    proptest::sample::select(VALUES)
}
