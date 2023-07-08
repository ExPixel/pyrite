use crate::{cpu::Cpu, memory::Memory, CpuException, Cycles};

pub fn todo(instr: u32, cpu: &mut Cpu, _memory: &mut dyn Memory) -> Cycles {
    let address = cpu.registers.read(15).wrapping_sub(4);
    todo!("TODO: addr=0x{address:08X}; instr=0x{instr:04X}");
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
