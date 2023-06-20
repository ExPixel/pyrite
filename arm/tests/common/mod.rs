use arm::{CpsrFlag, Cpu, CpuMode, Cycles, InstructionSet, Memory};

use self::asm::assemble;

pub mod asm;

#[derive(Default)]
pub struct TestMemory {
    data: Vec<u8>,
}

impl TestMemory {
    pub fn view32(&mut self, address: u32) -> u32 {
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
    fn load8(&mut self, address: u32, cycles: Option<&mut arm::Cycles>) -> u8 {
        let address = address as usize % self.data.len();
        if let Some(cycles) = cycles {
            *cycles += Cycles::one();
        }
        self.data[address]
    }

    fn store8(&mut self, address: u32, value: u8, cycles: Option<&mut arm::Cycles>) {
        let address = address as usize % self.data.len();
        if let Some(cycles) = cycles {
            *cycles += Cycles::one();
        }
        self.data[address] = value;
    }
}

/// An opcode that is actually an undefined instruction that is
/// used for signaling the end of execution in ARM mode.
const ARM_END_OPCODE: u32 = 0xF777F777;

/// An opcode that is used to signal the end of execution in THUMB mode.
/// By itself this is an undefined instruction. (2 of them make a branch with link but w/e)
const THUMB_END_OPCODE: u16 = 0xF777;

pub fn execute_arm(name: &str, source: &str) -> (Cpu, TestMemory) {
    let mut exec = Executor::new(name, InstructionSet::Arm);
    exec.push(source);
    (exec.cpu, exec.mem)
}

pub struct Executor {
    pub cpu: Cpu,
    pub mem: TestMemory,
    pub name: String,

    data: String,
    source: String,
    base_isa: InstructionSet,
    count: u32,
}

impl Executor {
    pub fn new(name: impl Into<String>, base_isa: InstructionSet) -> Self {
        Executor {
            cpu: Cpu::uninitialized(base_isa, CpuMode::System),
            mem: TestMemory::default(),
            name: name.into(),
            source: String::new(),
            data: String::new(),
            base_isa,
            count: 0,
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
        self.count += 1;
    }

    pub fn push(&mut self, source: &str) {
        self.push_no_exec(source);
        self.execute();
    }

    fn execute(&mut self) {
        let name = format!("{}-{}", self.name, self.count);

        let mut source = String::new();
        if !self.data.is_empty() {
            source.push_str(".data\n");
            source.push_str(&self.data);
        }
        source.push_str(".text\n");
        source.push_str(&self.source);
        source.push_str(".text\n");
        source.push_str("_exit:\n");
        source.push_str(".word 0xF777F777\n");
        self.mem.data = assemble(self.base_isa, &name, &source).unwrap();

        self.cpu
            .registers
            .put_flag(CpsrFlag::T, self.base_isa == InstructionSet::Thumb);
        self.cpu.branch(0, &mut self.mem);

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

            self.cpu.step(&mut self.mem);
        }
    }
}
