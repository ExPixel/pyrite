use util::bits::BitOps;

use crate::{
    alu::{BinaryOp, ExtractThumbOperand},
    cpu::Cpu,
    memory::Memory,
    transfer::{Ldr, SingleDataTransfer},
    CpuException, Cycles,
};

pub fn todo(instr: u32, cpu: &mut Cpu, _memory: &mut dyn Memory) -> Cycles {
    let address = cpu.registers.read(15).wrapping_sub(4);
    todo!("TODO: addr=0x{address:08X}; instr=0x{instr:04X}");
}

/// move shifted register
///
/// `LSL Rd, Rs, #Offset5`  
/// `LSR Rd, Rs, #Offset5`  
/// `ASR Rd, Rs, #Offset5`  
pub fn thumb_move_shifted_register<O>(instr: u32, cpu: &mut Cpu, _memory: &mut dyn Memory) -> Cycles
where
    O: BinaryOp,
{
    let rd = instr.get_bit_range(0..=2);
    let rs = instr.get_bit_range(3..=5);
    let lhs = cpu.registers.read(rs);
    let rhs = O::transform_imm_rhs(instr.get_bit_range(6..=10));
    let result = O::execute(&cpu.registers, lhs, rhs);
    O::set_flags(&mut cpu.registers, lhs, rhs, result);
    debug_assert!(O::HAS_RESULT);
    cpu.registers.write(rd, result);
    Cycles::zero()
}

/// move/compare/add/subtract immediate
///
/// `MOV Rd, #Offset8`  
/// `CMP Rd, #Offset8`  
/// `ADD Rd, #Offset8`  
/// `ADD Rd, #Offset8`  
pub fn thumb_mov_compare_add_subtract_imm<const RD: u32, O>(
    instr: u32,
    cpu: &mut Cpu,
    _memory: &mut dyn Memory,
) -> Cycles
where
    O: BinaryOp,
{
    let lhs = cpu.registers.read(RD);
    let rhs = O::transform_imm_rhs(instr.get_bit_range(0..=7));
    let result = O::execute(&cpu.registers, lhs, rhs);
    O::set_flags(&mut cpu.registers, lhs, rhs, result);
    if O::HAS_RESULT {
        cpu.registers.write(RD, result);
    }
    Cycles::zero()
}

/// PC-relative load
///
/// `LDR Rd, [PC, #Imm]`  
pub fn thumb_pc_relative_load<const RD: u32>(
    instr: u32,
    cpu: &mut Cpu,
    memory: &mut dyn Memory,
) -> Cycles {
    // From ARM7TDMI Documentation:
    //      The value of the PC will be 4 bytes greater than the address of this instruction,
    //      but bit 1 of PC is forced to 0 to ensure it is word aligned.
    // NOTE: Only bit 1 is forced to 0 because bit 0 is forced to 0 in THUMB mode anyway.
    let pc = cpu.registers.read(15) & 0xFFFFFFFC;

    // From ARM7TDMI Documentation:
    //      The value specified by #Imm is a full 10-bit address, but must always be word-aligned
    //      (ie with bits 1:0 set to 0), since the assembler places #Imm >> 2 in field Word8.
    let offset = instr.get_bit_range(0..=7) << 2;

    let address = pc.wrapping_add(offset);

    let cycles = Ldr::<false>::transfer(RD, address, &mut cpu.registers, memory);
    cycles + Cycles::one()
}

/// add/subtract
///
/// `ADD Rd, Rs, Rn`  
/// `ADD Rd, Rs, #Offset3`  
/// `SUB Rd, Rs, Rn`  
/// `SUB Rd, Rs, #Offset3`  
pub fn thumb_add_subtract<E, O>(instr: u32, cpu: &mut Cpu, _memory: &mut dyn Memory) -> Cycles
where
    E: ExtractThumbOperand,
    O: BinaryOp,
{
    let rs = instr.get_bit_range(3..=5);
    let rd = instr.get_bit_range(0..=2);
    let lhs = cpu.registers.read(rs);
    let rhs = E::extract(instr, &cpu.registers);
    let result = O::execute(&cpu.registers, lhs, rhs);
    O::set_flags(&mut cpu.registers, lhs, rhs, result);
    debug_assert!(O::HAS_RESULT);
    cpu.registers.write(rd, result);
    Cycles::zero()
}

/// Software Interrupt (SWI)
///
/// `SWI{cond} <expression>`  
pub fn thumb_swi(_instr: u32, cpu: &mut Cpu, memory: &mut dyn Memory) -> Cycles {
    cpu.exception_internal(CpuException::Swi, memory)
}

pub fn thumb_undefined(_instr: u32, cpu: &mut Cpu, memory: &mut dyn Memory) -> Cycles {
    cpu.exception_internal(CpuException::Undefined, memory)
}
