use crate::{
    cpu::{Cpu, Cycles},
    exception::CpuException,
    memory::Memory,
};

pub fn todo(_instr: u32, cpu: &mut Cpu, memory: &mut dyn Memory) -> Cycles {
    cpu.exception_internal(CpuException::Undefined, memory)
}
