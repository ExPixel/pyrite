use std::ops::{Add, AddAssign};

use crate::{
    exception::{CpuException, ExceptionHandler, ExceptionHandlerResult, EXCEPTION_BASE},
    lookup,
    memory::Memory,
    CpsrFlag, CpuMode, Registers,
};

pub type InstrFn = fn(u32, &mut Cpu, &mut dyn Memory) -> Cycles;

/// mov r0, r0 -- opcode for an ARM instruction that does nothing.
const ARM_NOOP_OPCODE: u32 = 0xe1a00000;

/// mov r0, r0 -- opcode for a THUMB instruction that does nothing.
const THUMB_NOOP_OPCODE: u16 = 0x46c0;

pub struct Cpu {
    pub registers: Registers,
    fetched: u32,
    decoded: u32,
    exception_handler: Option<ExceptionHandler>,
}

#[derive(PartialEq, Clone, Copy, Eq)]
pub enum InstructionSet {
    Arm,
    Thumb,
}

impl Cpu {
    /// **IMPORTANT**: [`Cpu::branch`] must always be called with the starting address of the CPU
    /// before [`Cpu::step`] if this method is used to construct a [`Cpu`]. If not the PC
    /// will be 4 bytes ahead of where it should be.
    pub fn uninitialized(isa: InstructionSet, mode: CpuMode) -> Self {
        let mut registers = Registers::new(mode);

        let noop_opcode = if isa == InstructionSet::Thumb {
            registers.set_flag(CpsrFlag::T);
            THUMB_NOOP_OPCODE as u32
        } else {
            registers.clear_flag(CpsrFlag::T);
            ARM_NOOP_OPCODE
        };

        Cpu {
            registers,
            exception_handler: None,
            fetched: noop_opcode,
            decoded: noop_opcode,
        }
    }

    pub fn new(isa: InstructionSet, mode: CpuMode, memory: &mut dyn Memory) -> Self {
        let mut cpu = Cpu::uninitialized(isa, mode);
        cpu.branch(0, memory);
        cpu
    }

    /// Steps the CPU forward. This will run the next fetch/decode/execute step of the ARM CPU pipeline
    /// as well as handle any interrupts that may have occurred while doing so. This returns the number
    /// of cycles that were required to complete the step.
    ///
    /// At the start of the step function, the program counter will be one instruction ahead of the address
    /// of the instruction that wil be executed. Before execution occurs it will be set to be two instructions
    /// ahead.
    #[inline]
    pub fn step(&mut self, memory: &mut dyn Memory) -> Cycles {
        if self.registers.get_flag(CpsrFlag::T) {
            self.step_thumb(memory)
        } else {
            self.step_arm(memory)
        }
    }

    /// Returns the number of cycles required to step the CPU in the ARM state.
    #[inline]
    fn step_arm(&mut self, memory: &mut dyn Memory) -> Cycles {
        let opcode = self.decoded;
        let exec_fn = lookup::decode_arm_opcode(opcode);
        self.decoded = self.fetched;

        let mut cycles = Cycles::zero();
        let fetch_pc = (self.registers.read(15) & !0x3).wrapping_add(4);
        self.registers.write(15, fetch_pc);
        self.fetched = memory.code32(fetch_pc, true, Some(&mut cycles));

        if check_condition(opcode >> 28, &self.registers) {
            cycles + exec_fn(opcode, self, memory)
        } else {
            cycles
        }
    }

    /// Returns the number of cycles required to step the CPU in the THUMB state.
    #[inline]
    fn step_thumb(&mut self, memory: &mut dyn Memory) -> Cycles {
        let opcode = self.decoded;
        let exec_fn = lookup::decode_thumb_opcode(opcode);
        self.decoded = self.fetched;

        let mut cycles = Cycles::zero();
        let fetch_pc = (self.registers.read(15) & !0x1).wrapping_add(2);
        self.registers.write(15, fetch_pc);
        self.fetched = memory.code16(fetch_pc, true, Some(&mut cycles)) as u32;

        cycles + exec_fn(opcode, self, memory)
    }

    pub fn branch(&mut self, address: u32, memory: &mut dyn Memory) -> Cycles {
        if self.registers.get_flag(CpsrFlag::T) {
            self.branch_thumb(address, memory)
        } else {
            self.branch_arm(address, memory)
        }
    }

    pub(crate) fn branch_arm(&mut self, address: u32, memory: &mut dyn Memory) -> Cycles {
        let address = address & !0x3;

        let mut cycles = Cycles::zero();
        let decoded = memory.code32(address, false, Some(&mut cycles));
        let fetched = memory.code32(address.wrapping_add(4), true, Some(&mut cycles));

        self.decoded = decoded;
        self.fetched = fetched;

        self.registers.write(15, address.wrapping_add(4));

        cycles
    }

    pub(crate) fn branch_thumb(&mut self, address: u32, memory: &mut dyn Memory) -> Cycles {
        let address = address & !0x1;

        let mut cycles = Cycles::zero();
        let decoded = memory.code16(address, false, Some(&mut cycles));
        let fetched = memory.code16(address.wrapping_add(2), true, Some(&mut cycles));

        self.decoded = decoded as u32;
        self.fetched = fetched as u32;

        self.registers.write(15, address.wrapping_add(2));

        cycles
    }

    /// The address of the instruction that will be executed next.
    pub fn next_execution_address(&self) -> u32 {
        if self.registers.get_flag(CpsrFlag::T) {
            self.registers.read(15).wrapping_sub(2)
        } else {
            self.registers.read(15).wrapping_sub(4)
        }
    }

    /// Sets the exception handler that will be called whenever the CPU encounters an
    /// exception such as an IRQ, SWI, ect.
    ///
    /// Exception handlers can use [`Cpu::next_execution_address`] in order to retrieve an
    /// exception's return address.
    pub fn set_exception_handler<F>(&mut self, handler: F) -> Option<ExceptionHandler>
    where
        F: 'static
            + Send
            + Sync
            + FnMut(&mut Cpu, &mut dyn Memory, CpuException) -> ExceptionHandlerResult,
    {
        self.exception_handler.replace(Box::new(handler))
    }

    pub fn exception(&mut self, exception: CpuException, memory: &mut dyn Memory) -> Cycles {
        self.exception_with_ret(exception, self.next_execution_address(), memory)
    }

    /// This version is meant to be called when an exception is thrown inside of an
    /// instruction.
    #[allow(dead_code)]
    pub(crate) fn exception_internal(
        &mut self,
        exception: CpuException,
        memory: &mut dyn Memory,
    ) -> Cycles {
        let return_addr =
            self.registers
                .read(15)
                .wrapping_sub(if self.registers.get_flag(CpsrFlag::T) {
                    2
                } else {
                    4
                });
        self.exception_with_ret(exception, return_addr, memory)
    }

    /// Actions performed by CPU when entering an exception
    ///   - R14_<new mode>=PC+nn   ;save old PC, ie. return address
    ///   - SPSR_<new mode>=CPSR   ;save old flags
    ///   - CPSR new T,M bits      ;set to T=0 (ARM state), and M4-0=new mode
    ///   - CPSR new I bit         ;IRQs disabled (I=1), done by ALL exceptions
    ///   - CPSR new F bit         ;FIQs disabled (F=1), done by Reset and FIQ only
    ///   - PC=exception_vector
    fn exception_with_ret(
        &mut self,
        exception: CpuException,
        return_addr: u32,
        memory: &mut dyn Memory,
    ) -> Cycles {
        let exception_info = exception.info();
        let exception_vector = EXCEPTION_BASE + exception_info.offset;

        // we temporarily remove the handler while processing and exception
        // we don't want reentrant exception handling and Rust's borrow checker
        // doesn't like it anyway.
        if let Some(mut handler) = self.exception_handler.take() {
            let result = handler(self, memory, exception);
            if let ExceptionHandlerResult::Handled(cycles) = result {
                return cycles;
            }
        }

        let cpsr = self.registers.read_cpsr();
        self.registers.write_mode(exception_info.mode_on_entry); // Set the entry mode.
        self.registers.write_spsr(cpsr); // Set the CPSR of the old mode to the SPSR of the new mode.
        self.registers
            .write(14, return_addr.wrapping_add(exception_info.pc_adjust)); // Save the return address.
        self.registers.clear_flag(CpsrFlag::T); // Go into ARM mode.

        self.registers.set_flag(CpsrFlag::I); // IRQ disable (done by all modes)

        if let Some(f) = exception_info.f_flag {
            self.registers.put_flag(CpsrFlag::F, f); // FIQ disable (done by RESET and FIQ only)
        }

        self.branch_arm(exception_vector, memory) // PC = exception_vector
    }
}

/// Returns true if an instruction should run based
/// the given condition code and cpsr.
fn check_condition(cond: u32, regs: &Registers) -> bool {
    match cond {
        0x0 => regs.get_flag(CpsrFlag::Z), // 0:   EQ     Z=1           equal (zero) (same)
        0x1 => !regs.get_flag(CpsrFlag::Z), // 1:   NE     Z=0           not equal (nonzero) (not same)
        0x2 => regs.get_flag(CpsrFlag::C), // 2:   CS/HS  C=1           unsigned higher or same (carry set)
        0x3 => !regs.get_flag(CpsrFlag::C), // 3:   CC/LO  C=0           unsigned lower (carry cleared)
        0x4 => regs.get_flag(CpsrFlag::N),  // 4:   MI     N=1           negative (minus)
        0x5 => !regs.get_flag(CpsrFlag::N), // 5:   PL     N=0           positive or zero (plus)
        0x6 => regs.get_flag(CpsrFlag::V),  // 6:   VS     V=1           overflow (V set)
        0x7 => !regs.get_flag(CpsrFlag::V), // 7:   VC     V=0           no overflow (V cleared)
        0x8 => regs.get_flag(CpsrFlag::C) & !regs.get_flag(CpsrFlag::Z), // 8:   HI     C=1 and Z=0   unsigned higher
        0x9 => !regs.get_flag(CpsrFlag::C) | regs.get_flag(CpsrFlag::Z), // 9:   LS     C=0 or Z=1    unsigned lower or same
        0xA => regs.get_flag(CpsrFlag::N) == regs.get_flag(CpsrFlag::V), // A:   GE     N=V           greater or equal
        0xB => regs.get_flag(CpsrFlag::N) != regs.get_flag(CpsrFlag::V), // B:   LT     N<>V          less than
        0xC => {
            // C:   GT     Z=0 and N=V   greater than
            !regs.get_flag(CpsrFlag::Z) & (regs.get_flag(CpsrFlag::N) == regs.get_flag(CpsrFlag::V))
        }
        0xD => {
            // D:   LE     Z=1 or N<>V   less or equal
            regs.get_flag(CpsrFlag::Z) | (regs.get_flag(CpsrFlag::N) != regs.get_flag(CpsrFlag::V))
        }
        0xE => true, // E:   AL     -             always (the "AL" suffix can be omitted)
        0xF => false, // F:   NV     -             never (ARMv1,v2 only) (Reserved ARMv3 and up)

        // :(
        _ => unreachable!("bad condition code: 0x{:08X} ({:04b})", cond, cond),
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Cycles(u32);

impl Cycles {
    #[inline]
    pub const fn zero() -> Self {
        Cycles(0)
    }

    #[inline]
    pub const fn one() -> Cycles {
        Cycles(1)
    }

    #[inline]
    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }
}

impl From<u32> for Cycles {
    #[inline]
    fn from(value: u32) -> Self {
        Cycles(value)
    }
}

impl From<Cycles> for u32 {
    #[inline]
    fn from(value: Cycles) -> Self {
        value.0
    }
}

impl Add for Cycles {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Cycles(self.0 + rhs.0)
    }
}

impl AddAssign for Cycles {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}
