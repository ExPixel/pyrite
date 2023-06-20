use util::bits::BitOps;

use crate::{
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
