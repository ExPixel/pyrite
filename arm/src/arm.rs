use util::bits::BitOps;

#[cfg(feature = "nightly")]
use core::intrinsics::unlikely;
#[cfg(not(feature = "nightly"))]
use std::convert::identity as unlikely;

use crate::{
    alu::{BinaryOp, ExtractOp2},
    cpu::{Cpu, Cycles},
    memory::Memory,
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
pub fn arm_dataproc<const S: bool, O, E>(
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
    let result = O::execute::<S>(&mut cpu.registers, lhs, rhs);

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
