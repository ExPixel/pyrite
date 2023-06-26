use std::fmt::Display;

use util::bits::{BitOps, IntoBit};

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
#[repr(u32)]
pub enum CpuMode {
    /// User mode (usr) is the usual ARM program execution state,
    /// and is used for executing most application programs.
    User = 0b10000,

    /// System mode is a priviledged user mode for the operating system.
    /// NOTE: System mode can only be entered from another priviledged mode
    /// by modifying the the mode bit of the Current Program Status Register (CPSR),
    System = 0b11111,

    /// Fast Interrupt (FIQ) mode supports a data transfer or channel process.
    FIQ = 0b10001,

    /// Interrupt (IRQ) mode is used for general-purpose interrupt handling.
    IRQ = 0b10010,

    /// Supervisor mode is a protected mode for the operating system.
    Supervisor = 0b10011,

    /// Abort mode is entered after a data or instruction prefetch Abort.
    Abort = 0b10111,

    /// Undefined mode is entered when an undefined instruction is executed.
    Undefined = 0b11011,

    /// Used to represent any mode that is not defined by the ARMv4T instruction set.
    Invalid = 0b00000,
}

impl CpuMode {
    pub fn name(self) -> &'static str {
        match self {
            CpuMode::User => "User",
            CpuMode::System => "System",
            CpuMode::FIQ => "FIQ",
            CpuMode::IRQ => "IRQ",
            CpuMode::Supervisor => "Supervisor",
            CpuMode::Abort => "Abort",
            CpuMode::Undefined => "Undefined",
            CpuMode::Invalid => "Invalid",
        }
    }

    pub fn is_priviledged(self) -> bool {
        self != CpuMode::User && self != CpuMode::Invalid
    }

    pub fn from_bits(mode_bits: u32) -> CpuMode {
        match mode_bits {
            0b10000 => CpuMode::User,
            0b11111 => CpuMode::System,
            0b10001 => CpuMode::FIQ,
            0b10010 => CpuMode::IRQ,
            0b10011 => CpuMode::Supervisor,
            0b10111 => CpuMode::Abort,
            0b11011 => CpuMode::Undefined,
            _ => CpuMode::Invalid,
        }
    }

    pub fn from_bits_checked(mode_bits: u32) -> Result<CpuMode, InvalidModeBits> {
        let mode = Self::from_bits(mode_bits);
        if mode != CpuMode::Invalid {
            return Ok(mode);
        }
        Err(InvalidModeBits)
    }

    #[inline(always)]
    pub fn bits(self) -> u32 {
        self as u32
    }
}

impl Display for CpuMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[repr(u8)]
pub enum CpsrFlag {
    /// Negative or Less Than
    N = 31,
    /// Zero
    Z = 30,
    /// Carry
    C = 29,
    /// Overflow
    V = 28,
    /// IRQ Disable
    I = 7,
    /// FIQ Disable
    F = 6,
    /// State/Thumb mode
    T = 5,
}

pub struct Registers {
    /// The currently in use general purpose registers (r0-r15).
    gp_registers: [u32; 16],

    /// Banked registers for non user modes:
    /// - 0-4:   r8_fiq - r12_fiq
    /// - 5-6:   r13_fiq & r14_fiq
    /// - 7-8:   r13_svc & r14_svc
    /// - 9-10:  r13_abt & r14_abt
    /// - 11-12: r13_irq & r14_irq
    /// - 13-14: r13_und & r14_und
    bk_registers: [u32; 15],

    /// banked Saved Program Status Registers (SPSR)
    bk_spsr: [u32; 5],

    /// Current Program Status Register
    cpsr: u32,

    /// Saved Program Status Register
    spsr: u32,

    // ## DEBUGGING
    // These keep track of the value of the program counter (minus 2 instructions) when a register
    // was changed.
    #[cfg(feature = "track-register-writes")]
    gp_registers_record: [u32; 16],
    #[cfg(feature = "track-register-writes")]
    bk_registers_record: [u32; 15],
}

impl Registers {
    pub fn new(mode: CpuMode) -> Registers {
        Registers {
            gp_registers: [0; 16],
            bk_registers: [0; 15],
            bk_spsr: [0; 5],
            cpsr: mode.bits(),
            spsr: 0,

            #[cfg(feature = "track-register-writes")]
            gp_registers_record: [0; 16],
            #[cfg(feature = "track-register-writes")]
            bk_registers_record: [0; 15],
        }
    }

    /// Reads and returns the value of a general purpose register.
    #[inline(always)]
    #[must_use]
    pub fn read(&self, register: u32) -> u32 {
        self.gp_registers[register as usize]
    }

    /// Writes a value to a register.
    #[inline(always)]
    pub fn write(&mut self, register: u32, value: u32) {
        self.gp_registers[register as usize] = value;

        #[cfg(feature = "track-register-writes")]
        {
            let exec_addr =
                self.gp_registers[15].wrapping_sub(if self.get_flag(CpsrFlag::T) { 4 } else { 8 });
            self.gp_registers_record[register as usize] = exec_addr;
        }
    }

    #[cfg(feature = "track-register-writes")]
    #[inline(always)]
    pub fn register_change_location(&self, register: u32) -> u32 {
        self.gp_registers_record[register as usize]
    }

    #[cfg(not(feature = "track-register-writes"))]
    #[inline(always)]
    pub fn register_change_location(&self, _register: u32) -> u32 {
        0
    }

    pub fn write_with_mode(&mut self, tmp_mode: CpuMode, register: u32, value: u32) {
        let old_mode = self.read_mode();
        self.write_mode(tmp_mode);
        self.write(register, value);
        self.write_mode(old_mode);
    }

    pub fn read_with_mode(&mut self, tmp_mode: CpuMode, register: u32) -> u32 {
        let old_mode = self.read_mode();
        self.write_mode(tmp_mode);
        let value = self.read(register);
        self.write_mode(old_mode);
        value
    }

    #[inline]
    #[must_use]
    pub fn get_flag(&self, flag: CpsrFlag) -> bool {
        self.cpsr.get_bit(flag as u8)
    }

    #[inline]
    pub fn set_flag(&mut self, flag: CpsrFlag) {
        self.cpsr = self.cpsr.set_bit(flag as u8);
    }

    #[inline]
    pub fn clear_flag(&mut self, flag: CpsrFlag) {
        self.cpsr = self.cpsr.clear_bit(flag as u8);
    }

    #[inline]
    pub fn put_flag(&mut self, flag: CpsrFlag, value: impl IntoBit) {
        self.cpsr = self.cpsr.put_bit(flag as u8, value.into_bit());
    }

    /// Sets the mode of the CPU. This will also change the mode bits in the CPSR register
    /// and properly swap register values to their corresponding banked values for the new mode.
    ///
    /// ## Returns
    ///
    /// The previous mode.
    pub fn write_mode(&mut self, new_mode: CpuMode) -> CpuMode {
        let old_mode = self.read_mode();
        self.on_mode_switch(old_mode, new_mode);
        self.cpsr = (self.cpsr & 0xFFFFFFE0) | new_mode.bits();
        old_mode
    }

    /// Sets the mode bits of the CPSR register. This will also change the mode of the CPU
    /// and properly swap register values to their corresponding banked values for the new mode.
    pub fn write_mode_bits(&mut self, mode_bits: u32) {
        let old_mode = self.read_mode();

        let new_mode = CpuMode::from_bits_checked(mode_bits).unwrap_or_else(|_| {
            eprintln!("wrote invalid CPU mode 0b{:05b}", mode_bits);
            CpuMode::Invalid
        });
        self.on_mode_switch(old_mode, new_mode);
        self.cpsr = (self.cpsr & 0xFFFFFFE0) | mode_bits;
    }

    /// Returns the current mode of the CPU.
    #[inline(always)]
    #[must_use]
    pub fn read_mode(&self) -> CpuMode {
        CpuMode::from_bits(self.cpsr & 0x1F)
    }

    /// Returns the current mode bits of the CPSR register (lowest 5bits) will all other bits set to 0.
    #[inline(always)]
    #[must_use]
    pub fn read_mode_bits(&self) -> u32 {
        self.cpsr & 0x1F
    }

    /// Returns the value of the CPSR register.
    #[inline(always)]
    #[must_use]
    pub fn read_cpsr(&self) -> u32 {
        self.cpsr
    }

    /// Sets the value of the CPSR. If the mode bits are changed
    /// The mode of the CPU will be changed accordingly and banked registers will be loaded.
    pub fn write_cpsr(&mut self, value: u32) {
        let old_mode_bits = self.read_mode_bits();
        self.cpsr = value;
        let new_mode_bits = self.read_mode_bits();

        if old_mode_bits != new_mode_bits {
            let old_mode = CpuMode::from_bits(old_mode_bits);
            let new_mode = CpuMode::from_bits_checked(new_mode_bits).unwrap_or_else(|_| {
                eprintln!("wrote invalid CPU mode 0b{:05b}", new_mode_bits);
                CpuMode::Invalid
            });
            self.on_mode_switch(old_mode, new_mode);
        }
    }

    // #TODO(LOW): might want to make this panic or show a warning in debug mode
    //             when it is called and the CPU is in User or System mode.
    /// Reads the value of the Saved Program Status Register (SPSR)
    /// for the current mode. This will return a garbage value for the User and
    /// System modes.
    #[inline(always)]
    #[must_use]
    pub fn read_spsr(&self) -> u32 {
        self.spsr
    }

    // #TODO(LOW): might want to make this panic or show a warning in debug mode
    //             when it is called and the CPU is in User or System mode.
    /// Writes to the Saved Program Status Register (SPSR)
    /// for the current mode. In this emulation all modes have an SPSRs but the System
    /// and User mode SPSRs are not saved on a mode switch.
    #[inline(always)]
    pub fn write_spsr(&mut self, value: u32) {
        self.spsr = value;
    }

    /// Called during a mode switch to switch the general purpose registers
    /// and the spsr to their proper banked versions.
    fn on_mode_switch(&mut self, old_mode: CpuMode, new_mode: CpuMode) {
        let mut swap_reg = |gp: usize, bk: usize| {
            std::mem::swap(&mut self.gp_registers[gp], &mut self.bk_registers[bk]);

            #[cfg(feature = "track-register-writes")]
            std::mem::swap(
                &mut self.gp_registers_record[gp],
                &mut self.bk_registers_record[bk],
            );
        };

        if old_mode == new_mode {
            /* NOP */
            return;
        }

        if old_mode != CpuMode::User && old_mode != CpuMode::System {
            // if the old mode isn't user or system (which are our default modes)
            // change to system mode:
            match old_mode {
                CpuMode::FIQ => {
                    swap_reg(9, 1);
                    swap_reg(10, 2);
                    swap_reg(11, 3);
                    swap_reg(12, 4);
                    swap_reg(13, 5);
                    swap_reg(14, 6);
                    self.bk_spsr[0] = self.spsr;
                }

                CpuMode::Supervisor => {
                    swap_reg(13, 7);
                    swap_reg(14, 8);
                    self.bk_spsr[1] = self.spsr;
                }

                CpuMode::Abort => {
                    swap_reg(13, 9);
                    swap_reg(14, 10);
                    self.bk_spsr[2] = self.spsr;
                }

                CpuMode::IRQ => {
                    swap_reg(13, 11);
                    swap_reg(14, 12);
                    self.bk_spsr[3] = self.spsr;
                }

                CpuMode::Undefined => {
                    swap_reg(13, 13);
                    swap_reg(14, 14);
                    self.bk_spsr[4] = self.spsr;
                }

                CpuMode::User | CpuMode::System => { /* NOP */ }

                _ => unreachable!("bad old cpu mode in on_mode_switch: {old_mode:?}"),
            }
        }

        // now we can continue on as if we're switching from system mode.

        match new_mode {
            CpuMode::FIQ => {
                swap_reg(8, 0);
                swap_reg(9, 1);
                swap_reg(10, 2);
                swap_reg(11, 3);
                swap_reg(12, 4);
                swap_reg(13, 5);
                swap_reg(14, 6);
                self.spsr = self.bk_spsr[0];
            }

            CpuMode::Supervisor => {
                swap_reg(13, 7);
                swap_reg(14, 8);
                self.spsr = self.bk_spsr[1];
            }

            CpuMode::Abort => {
                swap_reg(13, 9);
                swap_reg(14, 10);
                self.spsr = self.bk_spsr[2];
            }

            CpuMode::IRQ => {
                swap_reg(13, 11);
                swap_reg(14, 12);
                self.spsr = self.bk_spsr[3];
            }

            CpuMode::Undefined => {
                swap_reg(13, 13);
                swap_reg(14, 14);
                self.spsr = self.bk_spsr[4];
            }

            CpuMode::User | CpuMode::System => { /* NOP */ }

            _ => unreachable!("bad new cpu mode in on_mode_switch: {new_mode:?}"),
        }
    }
}

pub struct InvalidModeBits;

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, ops::Range};

    use rand::Rng;

    use super::*;

    #[test]
    fn register_read_write() {
        let mut rng = rand::thread_rng();
        let values: [u32; 16] = std::array::from_fn(|_| rng.gen_range(u32::MIN..=u32::MAX));
        let mut registers = Registers::new(CpuMode::System);

        for register in 0..16 {
            registers.write(register, values[register as usize]);
            assert_eq!(registers.read(register), values[register as usize]);
        }
    }

    #[test]
    fn register_read_write_bank_switched() {
        let mut rng = rand::thread_rng();
        let unbanked_values: [u32; 16] =
            std::array::from_fn(|_| rng.gen_range(u32::MIN..=u32::MAX));
        let mut expected_values = HashMap::<(u32, CpuMode), u32>::new();
        let mut registers = Registers::new(CpuMode::System);

        let mut init_registers = |mode: CpuMode, banked: Range<u32>| {
            for register in 0..16 {
                let value = if banked.contains(&register) {
                    rng.gen_range(u32::MIN..=u32::MAX)
                } else {
                    unbanked_values[register as usize]
                };
                registers.write_with_mode(mode, register, value);
                expected_values.insert((register, mode), value);
            }
        };

        init_registers(CpuMode::User, 0..0);
        init_registers(CpuMode::System, 0..0);
        init_registers(CpuMode::FIQ, 8..(12 + 1));
        init_registers(CpuMode::Supervisor, 13..(14 + 1));
        init_registers(CpuMode::Abort, 13..(14 + 1));
        init_registers(CpuMode::IRQ, 13..(14 + 1));
        init_registers(CpuMode::Undefined, 13..(14 + 1));

        let mut assert_registers = |mode: CpuMode| {
            for register in 0..16 {
                let &expected = expected_values
                    .get(&(register, mode))
                    .expect("no register value for mode");
                assert_eq!(
                    expected,
                    registers.read_with_mode(mode, register),
                    "invalid value for r{register} in {mode} mode"
                );
            }
        };

        assert_registers(CpuMode::User);
        assert_registers(CpuMode::System);
        assert_registers(CpuMode::FIQ);
        assert_registers(CpuMode::Supervisor);
        assert_registers(CpuMode::Abort);
        assert_registers(CpuMode::IRQ);
        assert_registers(CpuMode::Undefined);
    }
}
