use std::sync::{Arc, Mutex};

use arm::emu::{CpuException, Cycles, ExceptionHandlerResult};
use arm_devkit::{LinkerScript, LinkerScriptWeakRef};
use gba::{Gba, GbaMemoryMappedHardware, NoopGbaAudioOutput, NoopGbaVideoOutput};

pub fn execute(source: &str) -> Gba {
    let mut exec = Executor::new(source);
    exec.execute();
    exec.gba
}

pub struct Executor {
    pub gba: Gba,
    source: String,
}

impl Executor {
    pub fn new(source: impl Into<String>) -> Self {
        Executor {
            gba: Gba::new(),
            source: source.into(),
        }
    }

    fn execute(&mut self) {
        let preamble = ".text\n.arm\n.global _start\n_start:\n";
        let mut source = String::with_capacity(self.source.len() + preamble.len());
        source.push_str(preamble);
        source.push_str(&self.source);
        println!("source:\n{source}\n");

        // Use cargo's temp file directory. Good to have this set epecially on Windows
        // where you're likely to have your code ignored by your antivirus (e.g. Windows Defender)
        // but not your temporary directory.
        arm_devkit::set_internal_tempfile_directory(env!("CARGO_TARGET_TMPDIR"));

        self.gba
            .set_gamepak(arm_devkit::arm::assemble(&source, simple_linker_script()).unwrap());
        self.gba.reset();

        let execution_ended: Arc<Mutex<bool>> = Arc::default();
        let execution_ended_from_handler = execution_ended.clone();
        self.gba
            .cpu
            .set_exception_handler(move |cpu, memory, exception| {
                if exception == CpuException::Swi {
                    let memory = memory
                        .as_mut_any()
                        .downcast_mut::<GbaMemoryMappedHardware>()
                        .unwrap();
                    let comment = if cpu.registers.get_flag(arm::emu::CpsrFlag::T) {
                        let instr = memory.view16(cpu.exception_address());
                        (instr as u32) & 0xFF
                    } else {
                        let instr = memory.view32(cpu.exception_address());
                        instr & 0xFFFFFF
                    };

                    if comment == 0xCE {
                        *execution_ended_from_handler.lock().unwrap() = true;
                        return ExceptionHandlerResult::Handled(Cycles::from(1));
                    }
                }
                ExceptionHandlerResult::Ignored
            });

        let start_time = std::time::Instant::now();
        let mut steps_since_time_chek = 0;

        loop {
            if *execution_ended.lock().unwrap() {
                break;
            }

            if steps_since_time_chek >= 1024 {
                if start_time.elapsed() > std::time::Duration::from_secs(5) {
                    let next_pc = self.gba.cpu.next_execution_address();
                    panic!("emulator timeout: 0x{next_pc:08X}");
                }
                steps_since_time_chek = 0;
            } else {
                steps_since_time_chek += 1;
            }

            self.gba
                .step(&mut NoopGbaVideoOutput, &mut NoopGbaAudioOutput);
        }
    }
}

fn simple_linker_script() -> LinkerScript {
    let mut locked = match SCRIPT.lock() {
        Ok(lock) => lock,
        Err(err) => err.into_inner(),
    };

    if let Some(script) = locked
        .as_ref()
        .and_then(|maybe_script| maybe_script.upgrade())
    {
        return script;
    }

    let script = LinkerScript::new(SOURCE).expect("failed to create linker script");
    *locked = Some(script.weak());
    return script;

    static SCRIPT: Mutex<Option<LinkerScriptWeakRef>> = Mutex::new(None);
    static SOURCE: &str = include_str!("../data/simple.ld");
}

#[macro_export]
macro_rules! emu {
    ($source:expr) => {
        $crate::common::execute(&format!($source))
    };
}
