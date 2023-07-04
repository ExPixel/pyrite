use arm::{CpsrFlag, CpuMode};
use proptest::prelude::*;

#[macro_use]
pub mod common;

use common::proptest_util::operand;

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
pub fn test_bx_arm() {
    let (cpu, _mem) = arm! {"
    main:
        ldr r0, =arm_main
        bx  r0
        b   _exit

    .arm
    arm_main:
    "};
    assert!(!cpu.registers.get_flag(CpsrFlag::T));
}

#[test]
pub fn test_bx_thumb() {
    let (cpu, _mem) = arm! {"
    main:
        ldr r0, =thumb_main+1
        bx  r0
        b   _exit

    .thumb
    thumb_main:
    "};
    assert!(cpu.registers.get_flag(CpsrFlag::T));
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

#[test]
pub fn test_data_processing_with_r15() {
    // When using R15 as operand (Rm or Rn), the returned value
    // depends on the instruction: PC+12 if I=0,R=1 (shift by register),
    // otherwise PC+8 (shift by immediate).
    let (cpu, _mem) = arm! {"
        mov r0, r15, lsl r1
        mov r1, r15, lsl #0
    "};

    assert_eq!(cpu.registers.read(0), 12);
    assert_eq!(cpu.registers.read(1), 12);
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

    #[test]
    fn test_muls(lhs in operand(), rhs in operand()) {
        let (cpu, _mem) = arm! {"
            ldr     r1, =#{lhs}
            ldr     r2, =#{rhs}
            muls    r0, r1, r2
        "};

        let expected_result = lhs.wrapping_mul(rhs);
        let expected_n = (expected_result as i32) < 0;
        let expected_z = expected_result == 0;

        prop_assert_eq!(cpu.registers.read(0), expected_result);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::N), expected_n);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::Z), expected_z);
    }

    #[test]
    fn test_mlas(lhs in operand(), rhs in operand(), acc in operand()) {
        let (cpu, _mem) = arm! {"
            ldr     r1, =#{lhs}
            ldr     r2, =#{rhs}
            ldr     r3, =#{acc}
            mlas    r0, r1, r2, r3
        "};

        let expected_result = lhs.wrapping_mul(rhs).wrapping_add(acc);
        let expected_n = (expected_result as i32) < 0;
        let expected_z = expected_result == 0;

        prop_assert_eq!(cpu.registers.read(0), expected_result);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::N), expected_n);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::Z), expected_z);
    }

    #[test]
    fn test_umulls(lhs in operand(), rhs in operand()) {
        let (cpu, _mem) = arm! {"
            ldr     r2, =#{lhs}
            ldr     r3, =#{rhs}
            umulls  r0, r1, r2, r3
        "};

        let expected_result = (lhs as u64).wrapping_mul(rhs as u64);
        let expected_n = (expected_result as i64) < 0;
        let expected_z = expected_result == 0;
        let expected_result_lo = expected_result as u32;
        let expected_result_hi = (expected_result >> 32) as u32;

        prop_assert_eq!(cpu.registers.read(0), expected_result_lo);
        prop_assert_eq!(cpu.registers.read(1), expected_result_hi);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::N), expected_n);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::Z), expected_z);
    }

    #[test]
    fn test_umlals(lhs in operand(), rhs in operand(), acc: u64) {
        let acc_lo = acc as u32;
        let acc_hi = (acc >> 32) as u32;

        let (cpu, _mem) = arm! {"
            ldr     r0, =#{acc_lo}
            ldr     r1, =#{acc_hi}
            ldr     r2, =#{lhs}
            ldr     r3, =#{rhs}
            umlals  r0, r1, r2, r3
        "};

        let expected_result = (lhs as u64).wrapping_mul(rhs as u64).wrapping_add(acc);
        let expected_n = (expected_result as i64) < 0;
        let expected_z = expected_result == 0;
        let expected_result_lo = expected_result as u32;
        let expected_result_hi = (expected_result >> 32) as u32;

        prop_assert_eq!(cpu.registers.read(0), expected_result_lo);
        prop_assert_eq!(cpu.registers.read(1), expected_result_hi);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::N), expected_n);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::Z), expected_z);
    }

    #[test]
    fn test_smulls(lhs in operand(), rhs in operand()) {
        let (cpu, _mem) = arm! {"
            ldr     r2, =#{lhs}
            ldr     r3, =#{rhs}
            smulls  r0, r1, r2, r3
        "};

        let expected_result = (lhs as i32 as i64).wrapping_mul(rhs as i32 as i64);
        let expected_n = expected_result < 0;
        let expected_z = expected_result == 0;
        let expected_result_lo = expected_result as u32;
        let expected_result_hi = (expected_result >> 32) as u32;

        prop_assert_eq!(cpu.registers.read(0), expected_result_lo);
        prop_assert_eq!(cpu.registers.read(1), expected_result_hi);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::N), expected_n);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::Z), expected_z);
    }

    #[test]
    fn test_smlals(lhs in operand(), rhs in operand(), acc: u64) {
        let acc_lo = acc as u32;
        let acc_hi = (acc >> 32) as u32;

        let (cpu, _mem) = arm! {"
            ldr     r0, =#{acc_lo}
            ldr     r1, =#{acc_hi}
            ldr     r2, =#{lhs}
            ldr     r3, =#{rhs}
            smlals  r0, r1, r2, r3
        "};

        let expected_result =
            (lhs as i32 as i64).wrapping_mul(rhs as i32 as i64).wrapping_add(acc as i64);
        let expected_n = expected_result < 0;
        let expected_z = expected_result == 0;
        let expected_result_lo = expected_result as u32;
        let expected_result_hi = (expected_result >> 32) as u32;

        prop_assert_eq!(cpu.registers.read(0), expected_result_lo);
        prop_assert_eq!(cpu.registers.read(1), expected_result_hi);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::N), expected_n);
        prop_assert_eq!(cpu.registers.get_flag(CpsrFlag::Z), expected_z);
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
pub fn test_ldr_unaligned() {
    let (cpu, _mem) = arm! {"
        ldr r1, =deadbeef
        mov r2, r1
        ldr r0, [r1, #2]
    .data
    deadbeef:
        .word 0xDEADBEEF
    "};
    assert_eq!(cpu.registers.read(1), cpu.registers.read(2));
    assert_eq!(cpu.registers.read(0), 0xBEEFDEAD);
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

#[test]
pub fn test_str() {
    let (cpu, mem) = arm! {"
        ldr r2, =deadbeef
        ldr r0, =0xDEADBEEF
        ldr r1, [r2]
        str r0, [r2]
    .data
    deadbeef:
        .word 0xAABBCCDD
    "};
    assert_eq!(cpu.registers.read(0), 0xDEADBEEF);
    assert_eq!(cpu.registers.read(1), 0xAABBCCDD);
    assert_eq!(mem.view32(cpu.registers.read(2)), cpu.registers.read(0));
}

#[test]
pub fn test_ldrb() {
    let (cpu, _mem) = arm! {"
        ldr     r1, =deadbeef
        mov     r2, r1
        ldrb    r0, [r1]
    .data
    deadbeef:
        .word 0xDEADBEEF
    "};
    assert_eq!(cpu.registers.read(1), cpu.registers.read(2));
    assert_eq!(cpu.registers.read(0), 0xEF);
}

#[test]
pub fn test_strb() {
    let (cpu, mem) = arm! {"
        ldr     r2, =deadbeef
        ldr     r0, =0xDEADBEEF
        ldr     r1, [r2]
        strb    r0, [r2]
    .data
    deadbeef:
        .word 0xAABBCCDD
    "};
    assert_eq!(cpu.registers.read(0), 0xDEADBEEF);
    assert_eq!(cpu.registers.read(1), 0xAABBCCDD);
    assert_eq!(mem.view32(cpu.registers.read(2)), 0xAABBCCEF);
}

#[test]
pub fn test_ldrh() {
    let (cpu, _mem) = arm! {"
        ldr     r1, =deadbeef
        mov     r2, r1
        ldrh    r0, [r1]
    .data
    deadbeef:
        .word 0xDEADBEEF
    "};
    assert_eq!(cpu.registers.read(1), cpu.registers.read(2));
    assert_eq!(cpu.registers.read(0), 0xBEEF);
}

#[test]
pub fn test_ldrh_post_index() {
    let (cpu, _mem) = arm! {"
        ldr     r1, =deadbeef
        mov     r2, r1
        ldrh    r0, [r1], #4
    .data
    deadbeef:
        .word 0xDEADBEEF
        .word 0xAABBCCDD
    "};
    assert_eq!(cpu.registers.read(1), cpu.registers.read(2).wrapping_add(4));
    assert_eq!(cpu.registers.read(0), 0xBEEF);
}

#[test]
pub fn test_ldrh_pre_increment() {
    let (cpu, _mem) = arm! {"
        ldr     r1, =deadbeef
        mov     r2, r1
        ldrh    r0, [r1, #4]!
    .data
    deadbeef:
        .word 0xDEADBEEF
        .word 0xAABBCCDD
    "};
    assert_eq!(cpu.registers.read(1), cpu.registers.read(2).wrapping_add(4));
    assert_eq!(cpu.registers.read(0), 0xCCDD);
}

#[test]
pub fn test_ldrh_pre_decrement() {
    let (cpu, _mem) = arm! {"
        ldr     r1, =deadbeef
        mov     r2, r1
        ldrh    r0, [r1, #-4]!
    .data
        .word 0xDEADBEEF
    deadbeef:
        .word 0xAABBCCDD
    "};
    assert_eq!(cpu.registers.read(1), cpu.registers.read(2).wrapping_sub(4));
    assert_eq!(cpu.registers.read(0), 0xBEEF);
}

#[test]
pub fn test_strh() {
    let (cpu, mem) = arm! {"
        ldr     r2, =deadbeef
        ldr     r0, =0xDEADBEEF
        ldr     r1, [r2]
        strh    r0, [r2]
    .data
    deadbeef:
        .word 0xAABBCCDD
    "};
    assert_eq!(cpu.registers.read(0), 0xDEADBEEF);
    assert_eq!(cpu.registers.read(1), 0xAABBCCDD);
    assert_eq!(mem.view32(cpu.registers.read(2)), 0xAABBBEEF);
}

#[test]
pub fn test_ldrsh() {
    let (cpu, _mem) = arm! {"
        ldr     r1, =deadbeef
        mov     r2, r1
        ldrsh   r0, [r1]
    .data
    deadbeef:
        .word 0xDEAD8000
    "};
    assert_eq!(cpu.registers.read(1), cpu.registers.read(2));
    assert_eq!(cpu.registers.read(0), 0xFFFF8000);
}

#[test]
pub fn test_ldrsb() {
    let (cpu, _mem) = arm! {"
        ldr     r1, =deadbeef
        mov     r2, r1
        ldrsb   r0, [r1]
    .data
    deadbeef:
        .word 0xDEAD8080
    "};
    assert_eq!(cpu.registers.read(1), cpu.registers.read(2));
    assert_eq!(cpu.registers.read(0), 0xFFFFFF80);
}

#[test]
pub fn test_swi() {
    let (cpu, _mem) = arm! {"
        b   main        @ Reset
        b   _exit       @ Undefined
        b   swi_handler @ SWI

    main:
        ldr r1, =12
        swi 0
        b   _exit
    swi_handler:
        add     r1, #2
        movs    pc, lr  @ return from SWI
    "};

    assert_eq!(cpu.registers.read(1), 14);
    assert_eq!(cpu.registers.read_mode(), CpuMode::System);
}

#[test]
pub fn test_swi_check_mode() {
    let (cpu, _mem) = arm! {"
        b   main        @ Reset
        b   _exit       @ Undefined
        b   swi_handler @ SWI

    main:
        ldr r1, =12
        swi 0
    swi_handler:
        b   _exit
    "};

    assert_eq!(cpu.registers.read_mode(), CpuMode::Supervisor);
}

#[test]
pub fn test_msr_move_value_to_status_word() {
    let (cpu, _mem) = arm! {"
        ldr     r0, =0x60000010
        msr     cpsr_all, r0
    "};
    assert_eq!(cpu.registers.read_mode(), CpuMode::User);
    assert!(!cpu.registers.get_flag(CpsrFlag::N));
    assert!(cpu.registers.get_flag(CpsrFlag::Z));
    assert!(cpu.registers.get_flag(CpsrFlag::C));
    assert!(!cpu.registers.get_flag(CpsrFlag::V));
}

#[test]
pub fn test_msr_move_value_to_status_word_flags_only() {
    let (cpu, _mem) = arm! {"
        ldr     r0, =0x60000010
        msr     cpsr_flg, r0
    "};
    assert_eq!(cpu.registers.read_mode(), CpuMode::System);
    assert!(!cpu.registers.get_flag(CpsrFlag::N));
    assert!(cpu.registers.get_flag(CpsrFlag::Z));
    assert!(cpu.registers.get_flag(CpsrFlag::C));
    assert!(!cpu.registers.get_flag(CpsrFlag::V));
}

#[test]
pub fn test_msr_move_value_to_status_word_spsr() {
    let (mut cpu, _mem) = arm! {"
        b   main        @ Reset
        b   _exit       @ Undefined
        b   swi_handler @ SWI

    main:
        swi     0
    swi_handler:                    @ use SWI handler here to have to mode with banks
        ldr     r0, =0x60000010
        msr     spsr_all, r0
        b       _exit
    "};
    cpu.registers.write_cpsr(cpu.registers.read_spsr());
    assert_eq!(cpu.registers.read_mode(), CpuMode::User);
    assert!(!cpu.registers.get_flag(CpsrFlag::N));
    assert!(cpu.registers.get_flag(CpsrFlag::Z));
    assert!(cpu.registers.get_flag(CpsrFlag::C));
    assert!(!cpu.registers.get_flag(CpsrFlag::V));
}

#[test]
pub fn test_mrs_move_status_word_to_register() {
    let (cpu, _mem) = arm! {"
        ldr     r0, =0x80000000
        movs    r0, r0, lsl #1  @ set carry and zero flags
        mrs     r0, cpsr
    "};

    assert_eq!(cpu.registers.read(0), 0x60000000 | (CpuMode::System as u32));
}

#[test]
pub fn test_ldmia() {
    let (cpu, _mem) = arm! {"
        ldr     r0, =data
        ldr     r5, =end_of_data
        ldmia   r0!, {{r1-r4}}
    .data
    data:
        .word 0x00112233
        .word 0x44556677
        .word 0x8899AABB
        .word 0xCCDDEEFF
    end_of_data:
        .word 0xDEADBEEF
    "};

    assert_eq!(cpu.registers.read(1), 0x00112233);
    assert_eq!(cpu.registers.read(2), 0x44556677);
    assert_eq!(cpu.registers.read(3), 0x8899AABB);
    assert_eq!(cpu.registers.read(4), 0xCCDDEEFF);
    assert_eq!(cpu.registers.read(0), cpu.registers.read(5));
}

#[test]
pub fn test_ldmib() {
    let (cpu, _mem) = arm! {"
        ldr     r0, =data
        ldr     r1, =0x00112233
        ldr     r2, =0x44556677
        ldr     r3, =0x8899AABB
        ldr     r4, =0xCCDDEEFF
        ldr     r5, =end_of_data
        ldr     r6, =data
        ldmib   r0!, {{r1-r4}}
    .data
    data:
        .word 0x00112233
        .word 0x44556677
        .word 0x8899AABB
        .word 0xCCDDEEFF
    end_of_data:
        .word 0xDEADBEEF
    "};

    assert_eq!(cpu.registers.read(1), 0x44556677);
    assert_eq!(cpu.registers.read(2), 0x8899AABB);
    assert_eq!(cpu.registers.read(3), 0xCCDDEEFF);
    assert_eq!(cpu.registers.read(4), 0xDEADBEEF);
    assert_eq!(cpu.registers.read(0), cpu.registers.read(5));
}

#[test]
pub fn test_ldmda() {
    let (cpu, _mem) = arm! {"
        ldr     r0, =end_of_data
        ldr     r5, =data
        ldmda   r0!, {{r1-r4}}
    .data
    data:
        .word 0x00112233
        .word 0x44556677
        .word 0x8899AABB
        .word 0xCCDDEEFF
    end_of_data:
        .word 0xDEADBEEF
    "};

    assert_eq!(cpu.registers.read(1), 0x44556677);
    assert_eq!(cpu.registers.read(2), 0x8899AABB);
    assert_eq!(cpu.registers.read(3), 0xCCDDEEFF);
    assert_eq!(cpu.registers.read(4), 0xDEADBEEF);
    assert_eq!(cpu.registers.read(0), cpu.registers.read(5));
}

#[test]
pub fn test_ldmdb() {
    let (cpu, _mem) = arm! {"
        ldr     r0, =end_of_data
        ldr     r5, =data
        ldmdb   r0!, {{r1-r4}}
    .data
    data:
        .word 0x00112233
        .word 0x44556677
        .word 0x8899AABB
        .word 0xCCDDEEFF
    end_of_data:
        .word 0xDEADBEEF
    "};

    assert_eq!(cpu.registers.read(1), 0x00112233);
    assert_eq!(cpu.registers.read(2), 0x44556677);
    assert_eq!(cpu.registers.read(3), 0x8899AABB);
    assert_eq!(cpu.registers.read(4), 0xCCDDEEFF);
    assert_eq!(cpu.registers.read(0), cpu.registers.read(5));
}

#[test]
pub fn test_stmia() {
    let (cpu, mem) = arm! {"
        ldr     r0, =data
        ldr     r1, =0x00112233
        ldr     r2, =0x44556677
        ldr     r3, =0x8899AABB
        ldr     r4, =0xCCDDEEFF
        ldr     r5, =end_of_data
        stmia   r0!, {{r1-r4}}
    .data
    data:
        .word 0x00000000
        .word 0x00000000
        .word 0x00000000
        .word 0x00000000
    end_of_data:
        .word 0x00000000
    "};

    let expected_data = [
        mem.view32(cpu.registers.read(5).wrapping_sub(16)),
        mem.view32(cpu.registers.read(5).wrapping_sub(12)),
        mem.view32(cpu.registers.read(5).wrapping_sub(8)),
        mem.view32(cpu.registers.read(5).wrapping_sub(4)),
    ];

    assert_eq!(cpu.registers.read(1), expected_data[0]);
    assert_eq!(cpu.registers.read(2), expected_data[1]);
    assert_eq!(cpu.registers.read(3), expected_data[2]);
    assert_eq!(cpu.registers.read(4), expected_data[3]);
    assert_eq!(cpu.registers.read(0), cpu.registers.read(5));
}

#[test]
pub fn test_stmib() {
    let (cpu, mem) = arm! {"
        ldr     r0, =data
        ldr     r1, =0x00112233
        ldr     r2, =0x44556677
        ldr     r3, =0x8899AABB
        ldr     r4, =0xCCDDEEFF
        ldr     r5, =end_of_data
        stmib   r0!, {{r1-r4}}
    .data
    data:
        .word 0x00000000
        .word 0x00000000
        .word 0x00000000
        .word 0x00000000
    end_of_data:
        .word 0x00000000
    "};

    let expected_data = [
        mem.view32(cpu.registers.read(5).wrapping_sub(12)),
        mem.view32(cpu.registers.read(5).wrapping_sub(8)),
        mem.view32(cpu.registers.read(5).wrapping_sub(4)),
        mem.view32(cpu.registers.read(5)),
    ];

    assert_eq!(cpu.registers.read(1), expected_data[0]);
    assert_eq!(cpu.registers.read(2), expected_data[1]);
    assert_eq!(cpu.registers.read(3), expected_data[2]);
    assert_eq!(cpu.registers.read(4), expected_data[3]);
    assert_eq!(cpu.registers.read(0), cpu.registers.read(5));
}

#[test]
pub fn test_stmda() {
    let (cpu, mem) = arm! {"
        ldr     r0, =end_of_data
        ldr     r1, =0x00112233
        ldr     r2, =0x44556677
        ldr     r3, =0x8899AABB
        ldr     r4, =0xCCDDEEFF
        ldr     r5, =data
        stmda   r0!, {{r1-r4}}
    .data
    data:
        .word 0x00000000
        .word 0x00000000
        .word 0x00000000
        .word 0x00000000
    end_of_data:
        .word 0x00000000
    "};

    let expected_data = [
        mem.view32(cpu.registers.read(5).wrapping_add(4)),
        mem.view32(cpu.registers.read(5).wrapping_add(8)),
        mem.view32(cpu.registers.read(5).wrapping_add(12)),
        mem.view32(cpu.registers.read(5).wrapping_add(16)),
    ];

    assert_eq!(cpu.registers.read(1), expected_data[0]);
    assert_eq!(cpu.registers.read(2), expected_data[1]);
    assert_eq!(cpu.registers.read(3), expected_data[2]);
    assert_eq!(cpu.registers.read(4), expected_data[3]);
    assert_eq!(cpu.registers.read(0), cpu.registers.read(5));
}

#[test]
pub fn test_stmdb() {
    let (cpu, mem) = arm! {"
        ldr     r0, =end_of_data
        ldr     r1, =0x00112233
        ldr     r2, =0x44556677
        ldr     r3, =0x8899AABB
        ldr     r4, =0xCCDDEEFF
        ldr     r5, =data
        stmdb   r0!, {{r1-r4}}
    .data
    data:
        .word 0x00000000
        .word 0x00000000
        .word 0x00000000
        .word 0x00000000
    end_of_data:
        .word 0x00000000
    "};

    let expected_data = [
        mem.view32(cpu.registers.read(5)),
        mem.view32(cpu.registers.read(5).wrapping_add(4)),
        mem.view32(cpu.registers.read(5).wrapping_add(8)),
        mem.view32(cpu.registers.read(5).wrapping_add(12)),
    ];

    assert_eq!(cpu.registers.read(1), expected_data[0]);
    assert_eq!(cpu.registers.read(2), expected_data[1]);
    assert_eq!(cpu.registers.read(3), expected_data[2]);
    assert_eq!(cpu.registers.read(4), expected_data[3]);
    assert_eq!(cpu.registers.read(0), cpu.registers.read(5));
}

#[test]
pub fn test_ldm_load_spsr() {
    let (cpu, _mem) = arm! {"
        b   main        @ Reset
        b   _exit       @ Undefined
        b   swi_handler @ SWI

    main:
        ldr     r0, =jump_to_exit
        swi     0
    jump_to_exit:
        ldr     r9, =0xDEADBEEF
        b       _exit

    swi_handler:                    @ use SWI handler here to have to mode with banks
        ldr     r0, =0x60000010
        msr     spsr_all, r0        @ Load some value into SPSR
        mrs     r6, cpsr_all        @ Remember CPSR

        ldr     r0, =data
        ldr     r5, =end_of_data
        ldmia   r0!, {{r1-r4, r15}}^ @ Load SPSR and goto jump_to_exit
        b       _exit
    .data
    data:
        .word 0x00112233
        .word 0x44556677
        .word 0x8899AABB
        .word 0xCCDDEEFF
        .word jump_to_exit
    end_of_data:
    "};

    assert_eq!(cpu.registers.read(1), 0x00112233);
    assert_eq!(cpu.registers.read(2), 0x44556677);
    assert_eq!(cpu.registers.read(3), 0x8899AABB);
    assert_eq!(cpu.registers.read(4), 0xCCDDEEFF);
    assert_eq!(cpu.registers.read(9), 0xDEADBEEF);
    assert_eq!(cpu.registers.read_cpsr(), 0x60000010);
    assert_eq!(cpu.registers.read(0), cpu.registers.read(5));
}
