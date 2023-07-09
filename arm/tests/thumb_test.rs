#[macro_use]
pub mod common;

use arm::{CpsrFlag, CpuMode};

use crate::common::operands::{bools, imm3, imm32, imm8};

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
        let (cpu, _mem) = thumb! {"ldr r0, =#{value}"};
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
    fn test_add_imm8(lhs in imm32(), rhs in imm8()) {
        println!("lhs = {lhs}, rhs = {rhs}");
        let (cpu, _mem) = thumb! {"
                ldr r0, =#{lhs}
                add r0, #{rhs}
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
    fn test_sub_imm8(lhs in imm32(), rhs in imm8()) {
        let (cpu, _mem) = thumb! {"
                ldr r0, =#{lhs}
                sub r0, #{rhs}
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

    #[test]
    fn test_mov_imm8(rhs in imm8()) {
        let (cpu, _mem) = thumb! {"
            mov     r0, #{rhs}
        "};

        assert_eq!(cpu.registers.get_flag(CpsrFlag::N), rhs < 0);
        assert_eq!(cpu.registers.get_flag(CpsrFlag::Z), rhs == 0);
    }

    #[test]
    fn test_cmp_imm8(lhs in imm32(), rhs in imm8()) {
        let (cpu, _mem) = thumb! {"
            ldr     r0, =#{lhs}
            cmp     r0, #{rhs}
        "};

        let expected_result = lhs.wrapping_sub(rhs);
        let expected_n = expected_result < 0;
        let expected_z = expected_result == 0;
        let expected_c = (lhs as u32) >= (rhs as u32);
        let expected_v = (lhs as i32).overflowing_sub(rhs as i32).1;

        assert_eq!(cpu.registers.get_flag(CpsrFlag::N), expected_n);
        assert_eq!(cpu.registers.get_flag(CpsrFlag::Z), expected_z);
        assert_eq!(cpu.registers.get_flag(CpsrFlag::C), expected_c);
        assert_eq!(cpu.registers.get_flag(CpsrFlag::V), expected_v);
    }

    #[test]
    fn test_sbc(lhs in imm32(), rhs in imm32(), initial_carry in bools()) {
        let (cpu, _mem) = if initial_carry {
            thumb! {"
                ldr r0, =#{lhs}
                ldr r1, =#{rhs}

                ldr r2, =0x80000000 @ set carry flag
                lsl r2, r2, #1

                sbc r0, r1
            "}
        } else {
            thumb! {"
                ldr r0, =#{lhs}
                ldr r1, =#{rhs}
                sbc r0, r1
            "}
        };

        let expected_result = lhs.wrapping_sub(rhs).wrapping_sub(!initial_carry as i32);
        let expected_n = expected_result < 0;
        let expected_z = expected_result == 0;
        let expected_c = (lhs as u32 as u64) >= (rhs as u32 as u64 + (!initial_carry) as u64);
        let expected_v = ((((lhs as u32) >> 31) ^ (rhs as u32)) & (((lhs as u32) >> 31) ^ (expected_result as u32))) != 0;

        assert_eq!(cpu.registers.read(0), expected_result as u32);
        assert_eq!(cpu.registers.get_flag(CpsrFlag::N), expected_n);
        assert_eq!(cpu.registers.get_flag(CpsrFlag::Z), expected_z);
        assert_eq!(cpu.registers.get_flag(CpsrFlag::C), expected_c);
        assert_eq!(cpu.registers.get_flag(CpsrFlag::V), expected_v);
    }

    #[test]
    fn test_adc(lhs in imm32(), rhs in imm32(), initial_carry in bools()) {
        let (cpu, _mem) = if initial_carry {
            thumb! {"
                ldr r0, =#{lhs}
                ldr r1, =#{rhs}

                ldr r2, =0x80000000 @ set carry flag
                lsl r2, r2, #1

                adc r0, r1
            "}
        } else {
            thumb! {"
                ldr r0, =#{lhs}
                ldr r1, =#{rhs}
                adc r0, r1
            "}
        };

        let expected_result = lhs.wrapping_add(rhs).wrapping_add(initial_carry as i32);
        let expected_n = expected_result < 0;
        let expected_z = expected_result == 0;
        let expected_c = (lhs as u32).overflowing_add(rhs as u32).1
            | (lhs as u32)
                .wrapping_add(rhs as u32)
                .overflowing_add(initial_carry as u32)
                .1;
        let expected_v = lhs.overflowing_add(rhs).1
            | lhs
                .wrapping_add(rhs)
                .overflowing_add(initial_carry as i32)
                .1;

        assert_eq!(cpu.registers.read(0), expected_result as u32);
        assert_eq!(cpu.registers.get_flag(CpsrFlag::N), expected_n);
        assert_eq!(cpu.registers.get_flag(CpsrFlag::Z), expected_z);
        assert_eq!(cpu.registers.get_flag(CpsrFlag::C), expected_c);
        assert_eq!(cpu.registers.get_flag(CpsrFlag::V), expected_v);
    }

    #[test]
    fn test_neg(rhs in imm32()) {
        let (cpu, _mem) = thumb! {"
                ldr r1, =#{rhs}
                neg r0, r1
            "};

        let expected_result = (rhs as u32).wrapping_neg() as i32;
        let expected_n = expected_result < 0;
        let expected_z = expected_result == 0;
        let expected_c = rhs == 0;
        let expected_v = 0i32.overflowing_sub(rhs).1;

        assert_eq!(cpu.registers.read(0), expected_result as u32);
        assert_eq!(cpu.registers.get_flag(CpsrFlag::N), expected_n);
        assert_eq!(cpu.registers.get_flag(CpsrFlag::Z), expected_z);
        assert_eq!(cpu.registers.get_flag(CpsrFlag::C), expected_c);
        assert_eq!(cpu.registers.get_flag(CpsrFlag::V), expected_v);
    }

    #[test]
    fn test_mvn(lhs in imm32()) {
        let (cpu, _mem) = thumb! {"
                ldr r1, =#{lhs}
                mvn r0, r1
            "};

        let expected_result = !lhs;
        let expected_n = expected_result < 0;
        let expected_z = expected_result == 0;

        assert_eq!(cpu.registers.read(0), expected_result as u32);
        assert_eq!(cpu.registers.get_flag(CpsrFlag::N), expected_n);
        assert_eq!(cpu.registers.get_flag(CpsrFlag::Z), expected_z);
        assert!(!cpu.registers.get_flag(CpsrFlag::C));
        assert!(!cpu.registers.get_flag(CpsrFlag::V));
    }

    #[test]
    fn test_and(lhs in imm32(), rhs in imm32()) {
        let (cpu, _mem) = thumb! {"
            ldr r0, =#{lhs}
            ldr r1, =#{rhs}
            and r0, r1
        "};

        let expected_result = lhs & rhs;
        let expected_n = expected_result < 0;
        let expected_z = expected_result == 0;

        assert_eq!(cpu.registers.read(0), expected_result as u32);
        assert_eq!(cpu.registers.get_flag(CpsrFlag::N), expected_n);
        assert_eq!(cpu.registers.get_flag(CpsrFlag::Z), expected_z);
        assert!(!cpu.registers.get_flag(CpsrFlag::C));
        assert!(!cpu.registers.get_flag(CpsrFlag::V));
    }

    #[test]
    fn test_bics(lhs in imm32(), rhs in imm32()) {
        let (cpu, _mem) = thumb! {"
            ldr r0, =#{lhs}
            ldr r1, =#{rhs}
            bic r0, r1
        "};

        let expected_result = lhs & !rhs;
        let expected_n = expected_result < 0;
        let expected_z = expected_result == 0;

        assert_eq!(cpu.registers.read(0), expected_result as u32);
        assert_eq!(cpu.registers.get_flag(CpsrFlag::N), expected_n);
        assert_eq!(cpu.registers.get_flag(CpsrFlag::Z), expected_z);
        assert!(!cpu.registers.get_flag(CpsrFlag::C));
        assert!(!cpu.registers.get_flag(CpsrFlag::V));
    }

    #[test]
    fn test_orrs(lhs in imm32(), rhs in imm32()) {
        let (cpu, _mem) = thumb! {"
            ldr r0, =#{lhs}
            ldr r1, =#{rhs}
            orr r0, r1
        "};

        let expected_result = lhs | rhs;
        let expected_n = expected_result < 0;
        let expected_z = expected_result == 0;

        assert_eq!(cpu.registers.read(0), expected_result as u32);
        assert_eq!(cpu.registers.get_flag(CpsrFlag::N), expected_n);
        assert_eq!(cpu.registers.get_flag(CpsrFlag::Z), expected_z);
        assert!(!cpu.registers.get_flag(CpsrFlag::C));
        assert!(!cpu.registers.get_flag(CpsrFlag::V));
    }

    #[test]
    fn test_eors(lhs in imm32(), rhs in imm32()) {
        let (cpu, _mem) = thumb! {"
            ldr r0, =#{lhs}
            ldr r1, =#{rhs}
            eor r0, r1
        "};

        let expected_result = lhs ^ rhs;
        let expected_n = expected_result < 0;
        let expected_z = expected_result == 0;

        assert_eq!(cpu.registers.read(0), expected_result as u32);
        assert_eq!(cpu.registers.get_flag(CpsrFlag::N), expected_n);
        assert_eq!(cpu.registers.get_flag(CpsrFlag::Z), expected_z);
        assert!(!cpu.registers.get_flag(CpsrFlag::C));
        assert!(!cpu.registers.get_flag(CpsrFlag::V));
    }

    #[test]
    fn test_cmp(lhs in imm32(), rhs in imm32()) {
        let (cpu, _mem) = arm! {"
            ldr r0, =#{lhs}
            ldr r1, =#{rhs}
            cmp r0, r1
        "};

        let expected_result = lhs.wrapping_sub(rhs);
        let expected_n = expected_result < 0;
        let expected_z = expected_result == 0;
        let expected_c = (lhs as u32) >= (rhs as u32);
        let expected_v = (lhs as i32).overflowing_sub(rhs as i32).1;

        assert_eq!(cpu.registers.get_flag(CpsrFlag::N), expected_n);
        assert_eq!(cpu.registers.get_flag(CpsrFlag::Z), expected_z);
        assert_eq!(cpu.registers.get_flag(CpsrFlag::C), expected_c);
        assert_eq!(cpu.registers.get_flag(CpsrFlag::V), expected_v);
    }

    #[test]
    fn test_cmn(lhs in imm32(), rhs in imm32()) {
        let (cpu, _mem) = thumb! {"
            ldr r0, =#{lhs}
            ldr r1, =#{rhs}
            cmn r0, r1
        "};

        let expected_result = lhs.wrapping_add(rhs);
        let expected_n = expected_result < 0;
        let expected_z = expected_result == 0;
        let expected_c = (lhs as u32).overflowing_add(rhs as u32).1;
        let expected_v = (lhs as i32).overflowing_add(rhs as i32).1;

        assert_eq!(cpu.registers.get_flag(CpsrFlag::N), expected_n);
        assert_eq!(cpu.registers.get_flag(CpsrFlag::Z), expected_z);
        assert_eq!(cpu.registers.get_flag(CpsrFlag::C), expected_c);
        assert_eq!(cpu.registers.get_flag(CpsrFlag::V), expected_v);
    }

    #[test]
    fn test_tsts(lhs in imm32(), rhs in imm32()) {
        let (cpu, _mem) = thumb! {"
            ldr r0, =#{lhs}
            ldr r1, =#{rhs}
            tst r0, r1
        "};

        let expected_result = lhs & rhs;
        let expected_n = expected_result < 0;
        let expected_z = expected_result == 0;

        assert_eq!(cpu.registers.get_flag(CpsrFlag::N), expected_n);
        assert_eq!(cpu.registers.get_flag(CpsrFlag::Z), expected_z);
        assert!(!cpu.registers.get_flag(CpsrFlag::C));
        assert!(!cpu.registers.get_flag(CpsrFlag::V));
    }


    #[test]
    fn test_mul(lhs in imm32(), rhs in imm32()) {
        let (cpu, _mem) = thumb! {"
            ldr r0, =#{lhs}
            ldr r1, =#{rhs}
            mul r0, r1
        "};

        let expected_result = lhs.wrapping_mul(rhs);
        let expected_n = expected_result < 0;
        let expected_z = expected_result == 0;

        assert_eq!(cpu.registers.read(0), expected_result as u32);
        assert_eq!(cpu.registers.get_flag(CpsrFlag::N), expected_n);
        assert_eq!(cpu.registers.get_flag(CpsrFlag::Z), expected_z);
    }
}

#[test]
pub fn test_swi() {
    let (cpu, _mem) = arm! {"
        b   main        @ Reset
        b   _exit       @ Undefined
        b   swi_handler @ SWI

    main:
        ldr r0, =main_thumb+1
        bx  r0
        b   _exit
    .pool
    swi_handler:
        add     r1, #2
        movs    pc, lr  @ return from SWI
        b       _exit

    .thumb
    main_thumb:
        ldr r1, =12
        swi 0
    .hword 0xF777F777    @ EXIT (FIXME: change to to a branch once those are implemented)
    "};

    assert_eq!(cpu.registers.read(1), 14);
    assert_eq!(cpu.registers.read_mode(), CpuMode::System);
}
