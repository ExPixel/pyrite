use arm::{AccessType, CpsrFlag, Cpu, CpuMode, InstructionSet, Memory, Waitstates};

use self::asm::assemble;

pub mod asm;
pub mod proptest_util;

#[derive(Default)]
pub struct TestMemory {
    data: Vec<u8>,
}

impl TestMemory {
    pub fn view32(&self, address: u32) -> u32 {
        self.view16(address) as u32 | ((self.view16(address.wrapping_add(2)) as u32) << 16)
    }

    pub fn view16(&self, address: u32) -> u16 {
        self.view8(address) as u16 | ((self.view8(address.wrapping_add(1)) as u16) << 8)
    }

    pub fn view8(&self, address: u32) -> u8 {
        self.data[address as usize % self.data.len()]
    }
}

impl Memory for TestMemory {
    fn load8(&mut self, address: u32, _access: AccessType) -> (u8, Waitstates) {
        let address = address as usize % self.data.len();
        (self.data[address], Waitstates::zero())
    }

    fn store8(&mut self, address: u32, value: u8, _access: AccessType) -> Waitstates {
        let address = address as usize % self.data.len();
        self.data[address] = value;
        Waitstates::zero()
    }
}

/// An opcode that is actually an undefined instruction that is
/// used for signaling the end of execution in ARM mode.
const ARM_END_OPCODE: u32 = 0xF777F777;

/// An opcode that is used to signal the end of execution in THUMB mode.
/// By itself this is an undefined instruction. (2 of them make a branch with link but w/e)
const THUMB_END_OPCODE: u16 = 0xF777;

pub fn execute_arm(source: &str) -> (Cpu, TestMemory) {
    let mut exec = Executor::new(InstructionSet::Arm);
    exec.push(source);
    (exec.cpu, exec.mem)
}

pub struct Executor {
    pub cpu: Cpu,
    pub mem: TestMemory,

    data: String,
    source: String,
    base_isa: InstructionSet,
}

impl Executor {
    pub fn new(base_isa: InstructionSet) -> Self {
        Executor {
            cpu: Cpu::uninitialized(base_isa, CpuMode::System),
            mem: TestMemory::default(),
            source: String::new(),
            data: String::new(),
            base_isa,
        }
    }

    pub fn clear_source(&mut self) {
        self.source.clear();
    }

    pub fn data(&mut self, data_source: &str) {
        self.data.push_str(data_source);
        self.data.push('\n');
    }

    pub fn push_no_exec(&mut self, source: &str) {
        self.source.push_str(source);
        self.source.push('\n');
    }

    pub fn push(&mut self, source: &str) {
        self.push_no_exec(source);
        self.execute();
    }

    fn execute(&mut self) {
        let mut source = String::new();
        source.push_str(".text\n");
        source.push_str(".global _start\n");
        source.push_str("_start:\n");
        source.push_str(&self.source);
        source.push_str(".text\n");
        source.push_str("_exit:\n");
        source.push_str(".word 0xF777F777\n");
        if !self.data.is_empty() {
            source.push_str(".data\n");
            source.push_str(&self.data);
        }
        println!("source:\n{source}\n");
        self.mem.data = assemble(self.base_isa, &source).unwrap();

        self.cpu
            .registers
            .put_flag(CpsrFlag::T, self.base_isa == InstructionSet::Thumb);
        self.cpu.branch(0, &mut self.mem);

        let start_time = std::time::Instant::now();
        let mut steps_since_time_chek = 0;

        loop {
            let next_pc = self.cpu.next_execution_address();

            // break in ARM mode
            if !self.cpu.registers.get_flag(CpsrFlag::T)
                && self.mem.view32(next_pc) == ARM_END_OPCODE
            {
                break;
            }

            // break in THUMB mode
            if self.cpu.registers.get_flag(CpsrFlag::T)
                && self.mem.view16(next_pc) == THUMB_END_OPCODE
            {
                break;
            }

            if steps_since_time_chek >= 1024 {
                if start_time.elapsed() > std::time::Duration::from_secs(5) {
                    panic!("emulator timeout: 0x{next_pc:08X}");
                }
                steps_since_time_chek = 0;
            } else {
                steps_since_time_chek += 1;
            }

            self.cpu.step(&mut self.mem);
        }
    }
}

macro_rules! arm {
    ($source:expr) => {
        $crate::common::execute_arm(&format!($source))
    };
}
