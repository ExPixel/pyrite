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
pub fn test_lsl() {
    let (cpu, _mem) = arm! {"
        mov r1, #1
        mov r0, r1, LSL #1
    "};
    assert_eq!(cpu.registers.read(0), 2);
    assert!(!cpu.registers.get_flag(arm::CpsrFlag::C));
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
    assert!(cpu.registers.get_flag(arm::CpsrFlag::C));
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
    assert!(cpu.registers.get_flag(arm::CpsrFlag::C));
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
    assert!(cpu.registers.get_flag(arm::CpsrFlag::C));
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
    assert!(cpu.registers.get_flag(arm::CpsrFlag::C));
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
    assert!(cpu.registers.get_flag(arm::CpsrFlag::C));
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
    assert!(!cpu.registers.get_flag(arm::CpsrFlag::C));
}

#[test]
pub fn test_lsr() {
    let (cpu, _mem) = arm! {"
        mov r1, #4
        mov r0, r1, LSR #1
    "};
    assert_eq!(cpu.registers.read(0), 2);
    assert!(!cpu.registers.get_flag(arm::CpsrFlag::C));
}

#[test]
pub fn test_lsrs_imm() {
    let (cpu, _mem) = arm! {"
        mov     r1, #5
        movs    r0, r1, LSR #1
    "};
    assert_eq!(cpu.registers.read(0), 2);
    assert!(cpu.registers.get_flag(arm::CpsrFlag::C));
}

#[test]
pub fn test_lsrs_reg() {
    let (cpu, _mem) = arm! {"
        mov     r1, #5
        mov     r2, #1
        movs    r0, r1, LSR r2
    "};
    assert_eq!(cpu.registers.read(0), 2);
    assert!(cpu.registers.get_flag(arm::CpsrFlag::C));
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
    assert!(cpu.registers.get_flag(arm::CpsrFlag::C));
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
    assert!(cpu.registers.get_flag(arm::CpsrFlag::C));
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
    assert!(cpu.registers.get_flag(arm::CpsrFlag::C));
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
    assert!(cpu.registers.get_flag(arm::CpsrFlag::C));
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
    assert!(!cpu.registers.get_flag(arm::CpsrFlag::C));
}

#[test]
pub fn test_asr_positive() {
    let (cpu, _mem) = arm! {"
        mov r1, #0x60000000
        mov r0, r1, ASR #1
    "};
    assert_eq!(cpu.registers.read(0), 0x30000000);
    assert!(!cpu.registers.get_flag(arm::CpsrFlag::C));
}

#[test]
pub fn test_asr_negative() {
    let (cpu, _mem) = arm! {"
        mov r1, #0xE0000000
        mov r0, r1, ASR #1
    "};
    assert_eq!(cpu.registers.read(0), 0xF0000000);
    assert!(!cpu.registers.get_flag(arm::CpsrFlag::C));
}

#[test]
pub fn test_asrs_imm() {
    let (cpu, _mem) = arm! {"
        mov     r1, #0xE0000001
        movs    r0, r1, ASR #1
    "};
    assert_eq!(cpu.registers.read(0), 0xF0000000);
    assert!(cpu.registers.get_flag(arm::CpsrFlag::C));
}

#[test]
pub fn test_asrs_reg() {
    let (cpu, _mem) = arm! {"
        mov     r1, #0xE0000001
        mov     r2, #1
        movs    r0, r1, ASR r2
    "};
    assert_eq!(cpu.registers.read(0), 0xF0000000);
    assert!(cpu.registers.get_flag(arm::CpsrFlag::C));
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
    assert!(cpu.registers.get_flag(arm::CpsrFlag::C));
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
    assert!(cpu.registers.get_flag(arm::CpsrFlag::C));
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
    assert!(cpu.registers.get_flag(arm::CpsrFlag::C));

    let (cpu, _mem) = arm! {"
        mov     r1, #0x70000000 
        movs    r0, r1, ASR #32
    "};
    assert_eq!(cpu.registers.read(0), 0x00000000);
    assert!(!cpu.registers.get_flag(arm::CpsrFlag::C));
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
    assert!(cpu.registers.get_flag(arm::CpsrFlag::C));

    let (cpu, _mem) = arm! {"
        mov     r1, #0x70000000 
        mov     r2, #32
        movs    r0, r1, ASR r2
    "};
    assert_eq!(cpu.registers.read(0), 0x00000000);
    assert!(!cpu.registers.get_flag(arm::CpsrFlag::C));
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
    assert!(cpu.registers.get_flag(arm::CpsrFlag::C));

    let (cpu, _mem) = arm! {"
        mov     r1, #0x70000000
        mov     r2, #33
        movs    r0, r1, ASR r2
    "};
    assert_eq!(cpu.registers.read(0), 0x00000000);
    assert!(!cpu.registers.get_flag(arm::CpsrFlag::C));
}

#[test]
pub fn test_ror() {
    let (cpu, _mem) = arm! {"
        mov r1, #0x0000000F
        mov r0, r1, ROR #4
    "};
    assert_eq!(cpu.registers.read(0), 0xF0000000);
    assert!(!cpu.registers.get_flag(arm::CpsrFlag::C));
}

#[test]
pub fn test_rors_imm() {
    let (cpu, _mem) = arm! {"
        mov     r1, #0x0000000F
        movs    r0, r1, ROR #4
    "};
    assert_eq!(cpu.registers.read(0), 0xF0000000);
    assert!(cpu.registers.get_flag(arm::CpsrFlag::C));
}

#[test]
pub fn test_rors_reg() {
    let (cpu, _mem) = arm! {"
        mov     r1, #0x0000000F
        mov     r2, #4
        movs    r0, r1, ROR r2
    "};
    assert_eq!(cpu.registers.read(0), 0xF0000000);
    assert!(cpu.registers.get_flag(arm::CpsrFlag::C));
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
    assert!(cpu.registers.get_flag(arm::CpsrFlag::C));
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
    assert!(cpu.registers.get_flag(arm::CpsrFlag::C));
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
    assert!(cpu.registers.get_flag(arm::CpsrFlag::C));

    // ROR by 32 has result equal to Rm, carry out equal to bit 31 of Rm.
    let (cpu, _mem) = arm! {"
        mov     r1, #0x80000000
        movs    r0, r1, LSL #1
        mov     r1, #0x70000000
        mov     r2, #32
        movs    r0, r1, ROR r2
    "};
    assert_eq!(cpu.registers.read(0), 0x70000000);
    assert!(!cpu.registers.get_flag(arm::CpsrFlag::C));
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
    assert!(cpu.registers.get_flag(arm::CpsrFlag::C));

    // ROR by 32 has result equal to Rm, carry out equal to bit 31 of Rm.
    let (cpu, _mem) = arm! {"
        mov     r1, #0x80000000
        movs    r0, r1, LSL #1
        mov     r1, #0x00000007
        mov     r2, #36
        movs    r0, r1, ROR r2
    "};
    assert_eq!(cpu.registers.read(0), 0x70000000);
    assert!(!cpu.registers.get_flag(arm::CpsrFlag::C));
}

#[test]
pub fn test_rrx_carry_clear() {
    let (cpu, _mem) = arm! {"
        mov r1, #0x10000001
        mov r0, r1, RRX
    "};
    assert_eq!(cpu.registers.read(0), 0x08000000);
    assert!(!cpu.registers.get_flag(arm::CpsrFlag::C));
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
    assert!(cpu.registers.get_flag(arm::CpsrFlag::C));
}

#[test]
pub fn test_rrxs_carry_clear() {
    let (cpu, _mem) = arm! {"
        mov     r1, #0x10000001
        movs    r0, r1, RRX
    "};
    assert_eq!(cpu.registers.read(0), 0x08000000);
    assert!(cpu.registers.get_flag(arm::CpsrFlag::C));

    let (cpu, _mem) = arm! {"
        mov     r1, #0x10000000
        movs    r0, r1, RRX
    "};
    assert_eq!(cpu.registers.read(0), 0x08000000);
    assert!(!cpu.registers.get_flag(arm::CpsrFlag::C));
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
    assert!(cpu.registers.get_flag(arm::CpsrFlag::C));

    let (cpu, _mem) = arm! {"
        mov     r1, #0x80000000
        movs    r0, r1, LSL #1
        mov     r1, #0x10000000
        movs    r0, r1, RRX
    "};
    assert_eq!(cpu.registers.read(0), 0x88000000);
    assert!(!cpu.registers.get_flag(arm::CpsrFlag::C));
}
