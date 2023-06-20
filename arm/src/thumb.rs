use crate::{
    cpu::{Cpu, Cycles},
    memory::Memory,
};

pub fn todo(instr: u32, cpu: &mut Cpu, _memory: &mut dyn Memory) -> Cycles {
    let address = cpu.registers.read(15).wrapping_sub(4);
    todo!("TODO: addr=0x{address:08X}; instr=0x{instr:04X}");
}
