use crate::Registers;

pub struct Cpsr;
pub struct Spsr;

impl Psr for Cpsr {
    fn write(value: u32, registers: &mut Registers) {
        registers.write_cpsr(value);
    }

    fn read(registers: &Registers) -> u32 {
        registers.read_cpsr()
    }
}

impl Psr for Spsr {
    fn write(value: u32, registers: &mut Registers) {
        registers.write_spsr(value)
    }

    fn read(registers: &Registers) -> u32 {
        registers.read_spsr()
    }
}

pub trait Psr {
    fn write(value: u32, registers: &mut Registers);

    fn write_flags_only(value: u32, registers: &mut Registers) {
        let old = Self::read(registers);
        Self::write((old & !0xF0000000) | (value & 0xF0000000), registers);
    }

    fn read(registers: &Registers) -> u32;
}
