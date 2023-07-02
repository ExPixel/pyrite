use util::bits::BitOps;

#[cfg(feature = "nightly")]
use core::intrinsics::unlikely;
#[cfg(not(feature = "nightly"))]
use std::convert::identity as unlikely;

use crate::{
    alu::{multiply, BinaryOp, ExtractOp2, Psr},
    clock::Cycles,
    cpu::Cpu,
    memory::Memory,
    transfer::{SDTCalculateOffset, SDTIndexingMode, SingleDataTransfer},
    CpuException,
};

pub fn todo(instr: u32, cpu: &mut Cpu, _memory: &mut dyn Memory) -> Cycles {
    let address = cpu.registers.read(15).wrapping_sub(8);
    todo!("TODO: addr=0x{address:08X}; instr=0x{instr:08X}");
}

/// Branch
///
/// B <offset>
pub fn arm_b(instr: u32, cpu: &mut Cpu, memory: &mut dyn Memory) -> Cycles {
    let offset = (instr & 0xFFFFFF).sign_extend(24).wrapping_shl(2);
    let pc = cpu.registers.read(15);
    let dest = pc.wrapping_add(offset);
    cpu.branch_arm(dest, memory)
}

/// Branch and Link
///
/// BL <offset>
pub fn arm_bl(instr: u32, cpu: &mut Cpu, memory: &mut dyn Memory) -> Cycles {
    let offset = (instr & 0xFFFFFF).sign_extend(24).wrapping_shl(2);
    let pc = cpu.registers.read(15);
    let dest = pc.wrapping_add(offset);
    cpu.registers.write(14, pc.wrapping_sub(4));
    cpu.branch_arm(dest, memory)
}

/// Data Processing Instruction
///
/// MOV,MVN (single operand instructions.)  
/// `<opcode>{cond}{S} Rd,<Op2>`
///
/// CMP,CMN,TEQ,TST (instructions which do not produce a result.)  
/// `<opcode>{cond} Rn,<Op2>`
///
/// AND,EOR,SUB,RSB,ADD,ADC,SBC,RSC,ORR,BIC  
/// `<opcode>{cond}{S} Rd,Rn,<Op2>`
pub fn arm_dataproc<O, const S: bool, E>(
    instr: u32,
    cpu: &mut Cpu,
    memory: &mut dyn Memory,
) -> Cycles
where
    O: BinaryOp,
    E: ExtractOp2,
{
    let rd = instr.get_bit_range(12..=15);
    let rn = instr.get_bit_range(16..=19);

    let mut lhs = cpu.registers.read(rn);
    let mut cycles = E::stall();

    // When using R15 as operand (Rm or Rn), the returned value
    // depends on the instruction: PC+12 if I=0,R=1 (shift by register),
    // otherwise PC+8 (shift by immediate).
    if rn == 15 && E::IS_REGISTER_SHIFT {
        lhs = lhs.wrapping_add(4);
    }

    let rhs = E::extract::<S>(instr, &mut cpu.registers);
    let result = O::execute(&cpu.registers, lhs, rhs);
    O::set_flags_if::<S>(&mut cpu.registers, lhs, rhs, result);

    // If S=1, Rd=R15; should not be used in user mode:
    //   CPSR = SPSR_<current mode>
    //   PC = result
    //   For example: MOVS PC,R14  ;return from SWI (PC=R14_svc, CPSR=SPSR_svc).
    if unlikely(rd == 15 && S) {
        cpu.registers.write_cpsr(cpu.registers.read_spsr());
        cycles += cpu.branch(result, memory);
    } else if unlikely(rd == 15 && O::HAS_RESULT) {
        cycles += cpu.branch(result, memory);
    } else if O::HAS_RESULT {
        cpu.registers.write(rd, result);
    } else {
        O::set_flags_if::<S>(&mut cpu.registers, lhs, rhs, result);
    }

    cycles
}

/// Single Data Transfer (LDR, STR)
///
/// `<LDR|STR>{cond}{B}{T} Rd,<Address>`
pub fn arm_single_data_transfer<T, O, I, const WRITEBACK: bool>(
    instr: u32,
    cpu: &mut Cpu,
    memory: &mut dyn Memory,
) -> Cycles
where
    T: SingleDataTransfer,
    O: SDTCalculateOffset,
    I: SDTIndexingMode,
{
    let rd = instr.get_bit_range(12..=15);
    let rn = instr.get_bit_range(16..=19);

    let offset = O::calculate_offset(instr, &mut cpu.registers);
    let mut address = cpu.registers.read(rn);
    address = I::calculate_transfer_address(address, offset);
    let mut cycles = T::transfer(rd, address, &mut cpu.registers, memory);

    if WRITEBACK {
        // FIXME At this point Rn is not allowed be r15 but I'm not sure if I should
        //       assert that and panic here or just log an error. No logging facilities
        //       for this part of the code at the moment though so once I figure that out
        //       I should probably take a look at this. For now I just branch anyway later.
        // From ARM Documentation:
        //      Write-back must not be specified if R15 is specified as the base register (Rn).
        //      When using R15 as the base register.
        address = I::calculate_writeback_address(address, offset);
        cpu.registers.write(rn, address);
    }

    if T::IS_LOAD && (rd == 15 || (WRITEBACK && rn == 15)) {
        let destination = cpu.registers.read(15);
        cycles += cpu.branch_arm(destination, memory);
    }

    cycles
}

/// Move value to status word
///
/// MSR - transfer register contents to PSR  
/// `MSR{cond} <psr>,Rm`
///
/// MSR - transfer register contents to PSR flag bits only  
/// `MSR{cond} <psrf>,Rm`  
/// The most significant four bits of the register contents are written to the N,Z,C
/// & V flags respectively.
///
/// MSR - transfer immediate value to PSR flag bits only  
/// `MSR{cond} <psrf>,<#expression>`
pub fn arm_msr<P, E>(instr: u32, cpu: &mut Cpu, _memory: &mut dyn Memory) -> Cycles
where
    P: Psr,
    E: ExtractOp2,
{
    let src = E::extract::<false>(instr, &mut cpu.registers);
    let flag_bits_only = (instr & 0x00010000) == 0;

    if flag_bits_only {
        P::write_flags_only(src, &mut cpu.registers);
    } else {
        P::write(src, &mut cpu.registers);
    }

    Cycles::zero()
}

/// Move status word to register
///
/// MRS{cond} Rd,<psr>
pub fn arm_mrs<P>(instr: u32, cpu: &mut Cpu, _memory: &mut dyn Memory) -> Cycles
where
    P: Psr,
{
    let src = P::read(&cpu.registers);
    let dst = instr.get_bit_range(12..=15);
    cpu.registers.write(dst, src);
    Cycles::zero()
}

/// Multiply and Multiply-Accumulate
///
/// MUL{cond}{S} Rd,Rm,Rs  
/// MLA{cond}{S} Rd,Rm,Rs,Rn
pub fn arm_mul<const S: bool, const A: bool>(
    instr: u32,
    cpu: &mut Cpu,
    _memory: &mut dyn Memory,
) -> Cycles {
    let rm = instr.get_bit_range(0..=3);
    let rs = instr.get_bit_range(8..=11);
    let rd = instr.get_bit_range(16..=19);

    let lhs = cpu.registers.read(rm);
    let rhs = cpu.registers.read(rs);
    let mut result = lhs.wrapping_mul(rhs);

    let acc_cycles = if A {
        let rn = instr.get_bit_range(12..=15);
        let accumulate = cpu.registers.read(rn);
        result = result.wrapping_add(accumulate);
        Cycles::one()
    } else {
        Cycles::zero()
    };

    if S {
        multiply::set_multiply_flags(result, &mut cpu.registers);
    }

    cpu.registers.write(rd, result);
    acc_cycles + multiply::internal_multiply_cycles(rhs)
}

/// Multiply Long and Multiply-Accumulate Long
///
/// UMULL{cond}{S} RdLo,RdHi,Rm,Rs  
/// UMLAL{cond}{S} RdLo,RdHi,Rm,Rs  
/// SMULL{cond}{S} RdLo,RdHi,Rm,Rs  
/// SMLAL{cond}{S} RdLo,RdHi,Rm,Rs  
pub fn arm_mul_long<const SIGNED: bool, const S: bool, const A: bool>(
    instr: u32,
    cpu: &mut Cpu,
    _memory: &mut dyn Memory,
) -> Cycles {
    let rm = instr.get_bit_range(0..=3);
    let rs = instr.get_bit_range(8..=11);
    let rd_lo = instr.get_bit_range(12..=15);
    let rd_hi = instr.get_bit_range(16..=19);

    let lhs = cpu.registers.read(rm) as u64;
    let rhs = cpu.registers.read(rs) as u64;

    let lhs = if SIGNED { lhs.sign_extend(32) } else { lhs };
    let rhs = if SIGNED { rhs.sign_extend(32) } else { rhs };

    let (acc, acc_cycles) = if A {
        let acc_lo = cpu.registers.read(rd_lo) as u64;
        let acc_hi = cpu.registers.read(rd_hi) as u64;
        ((acc_hi << 32) | acc_lo, Cycles::one())
    } else {
        (0, Cycles::zero())
    };

    let result = lhs.wrapping_mul(rhs).wrapping_add(acc);
    dbg!(result as i64);
    dbg!(rd_lo);
    dbg!(rd_hi);
    dbg!(result as u32);
    dbg!((result >> 32) as u32);

    if S {
        multiply::set_multiply_flags(result, &mut cpu.registers);
    }

    cpu.registers.write(rd_lo, result as u32);
    cpu.registers.write(rd_hi, (result >> 32) as u32);
    acc_cycles + multiply::internal_multiply_cycles(rhs as u32)
}

pub fn arm_swi(_instr: u32, cpu: &mut Cpu, memory: &mut dyn Memory) -> Cycles {
    cpu.exception_internal(CpuException::Swi, memory)
}

pub fn arm_undefined(_instr: u32, cpu: &mut Cpu, memory: &mut dyn Memory) -> Cycles {
    cpu.exception_internal(CpuException::Undefined, memory)
}

/// ARM9
pub fn arm_blx(instr: u32, cpu: &mut Cpu, memory: &mut dyn Memory) -> Cycles {
    arm_undefined(instr, cpu, memory)
}

// ARM9
pub fn arm_bkpt(instr: u32, cpu: &mut Cpu, memory: &mut dyn Memory) -> Cycles {
    arm_undefined(instr, cpu, memory)
}

// ARM9
pub fn arm_clz(instr: u32, cpu: &mut Cpu, memory: &mut dyn Memory) -> Cycles {
    arm_undefined(instr, cpu, memory)
}

// Used for unsupported M-Extension instructions
pub fn arm_m_extension_undefined(instr: u32, cpu: &mut Cpu, memory: &mut dyn Memory) -> Cycles {
    arm_undefined(instr, cpu, memory)
}
