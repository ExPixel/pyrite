use crate::{memory::Memory, Cpu, CpuMode, Cycles};

pub type ExceptionHandler =
    Box<dyn Send + Sync + FnMut(&mut Cpu, &mut dyn Memory, CpuException) -> ExceptionHandlerResult>;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ExceptionHandlerResult {
    Handled(Cycles),
    Ignored,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CpuException {
    Reset,
    Undefined,
    Swi,
    PrefetchAbort,
    DataAbort,
    Irq,
    Fiq,
    AddressExceeds26Bit,
}

impl CpuException {
    fn name(self) -> &'static str {
        match self {
            CpuException::Reset => "Reset",
            CpuException::Undefined => "Undefined",
            CpuException::Swi => "SWI",
            CpuException::PrefetchAbort => "Prefetch Abort",
            CpuException::DataAbort => "Data Abort",
            CpuException::Irq => "IRQ",
            CpuException::Fiq => "FIQ",
            CpuException::AddressExceeds26Bit => "Address Exceeds 26 bit",
        }
    }

    pub(crate) fn info(self) -> CpuExceptionInfo {
        match self {
            CpuException::Reset => EXCEPTION_INFO_RESET,
            CpuException::Undefined => EXCEPTION_INFO_UNDEFINED,
            CpuException::Swi => EXCEPTION_INFO_SWI,
            CpuException::PrefetchAbort => EXCEPTION_INFO_PREFETCH_ABORT,
            CpuException::DataAbort => EXCEPTION_INFO_DATA_ABORT,
            CpuException::Irq => EXCEPTION_INFO_IRQ,
            CpuException::Fiq => EXCEPTION_INFO_FIQ,
            CpuException::AddressExceeds26Bit => EXCEPTION_INFO_ADDRESS_EXCEEDS_26BIT,
        }
    }
}

impl std::fmt::Display for CpuException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct CpuExceptionInfo {
    pub(crate) mode_on_entry: CpuMode,
    pub(crate) f_flag: Option<bool>,
    pub(crate) pc_adjust: u32,
    pub(crate) offset: u32,

    /// Lower number means higher priority.
    pub(crate) priority: u8,
}

impl CpuExceptionInfo {
    pub const fn new(
        priority: u8,
        mode_on_entry: CpuMode,
        f_flag: Option<bool>,
        pc_adjust: u32,
        offset: u32,
    ) -> CpuExceptionInfo {
        CpuExceptionInfo {
            priority,
            mode_on_entry,
            f_flag,
            pc_adjust,
            offset,
        }
    }
}

// The following are the exception vectors in memory. That is, when an exception arises, CPU is switched into ARM state, and the program counter (PC) is loaded by the respective address.
//   Address  Prio  Exception                  Mode on Entry      Interrupt Flags
//   BASE+00h 1     Reset                      Supervisor (_svc)  I=1, F=1
//   BASE+04h 7     Undefined Instruction      Undefined  (_und)  I=1, F=unchanged
//   BASE+08h 6     Software Interrupt (SWI)   Supervisor (_svc)  I=1, F=unchanged
//   BASE+0Ch 5     Prefetch Abort             Abort      (_abt)  I=1, F=unchanged
//   BASE+10h 2     Data Abort                 Abort      (_abt)  I=1, F=unchanged
//   BASE+14h ??    Address Exceeds 26bit      Supervisor (_svc)  I=1, F=unchanged
//   BASE+18h 4     Normal Interrupt (IRQ)     IRQ        (_irq)  I=1, F=unchanged
//   BASE+1Ch 3     Fast Interrupt (FIQ)       FIQ        (_fiq)  I=1, F=1
pub const EXCEPTION_INFO_RESET: CpuExceptionInfo =
    CpuExceptionInfo::new(1, CpuMode::Supervisor, Some(true), 0, 0x00);
pub const EXCEPTION_INFO_UNDEFINED: CpuExceptionInfo =
    CpuExceptionInfo::new(7, CpuMode::Undefined, None, 0, 0x04);
pub const EXCEPTION_INFO_SWI: CpuExceptionInfo =
    CpuExceptionInfo::new(6, CpuMode::Supervisor, None, 0, 0x08);
pub const EXCEPTION_INFO_PREFETCH_ABORT: CpuExceptionInfo =
    CpuExceptionInfo::new(5, CpuMode::Abort, None, 4, 0x0C);
pub const EXCEPTION_INFO_DATA_ABORT: CpuExceptionInfo =
    CpuExceptionInfo::new(2, CpuMode::Abort, None, 4, 0x10);
pub const EXCEPTION_INFO_IRQ: CpuExceptionInfo =
    CpuExceptionInfo::new(4, CpuMode::IRQ, None, 4, 0x18);
pub const EXCEPTION_INFO_FIQ: CpuExceptionInfo =
    CpuExceptionInfo::new(3, CpuMode::FIQ, Some(true), 4, 0x1C);

// #TODO I don't actually know the priority for the 26bit address overflow exception.
pub const EXCEPTION_INFO_ADDRESS_EXCEEDS_26BIT: CpuExceptionInfo =
    CpuExceptionInfo::new(8, CpuMode::Supervisor, None, 4, 0x14);

pub const EXCEPTION_BASE: u32 = 0;
