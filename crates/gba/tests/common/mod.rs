use std::{
    path::Path,
    sync::{
        atomic::{self, AtomicBool},
        Arc, Mutex,
    },
};

use arm::{
    disasm::MemoryView as _,
    emu::{CpuException, Cycles, ExceptionHandlerResult},
};
use arm_devkit::{LinkerScript, LinkerScriptWeakRef};
use gba::{
    video::LineBuffer, Gba, GbaMemoryMappedHardware, GbaVideoOutput, NoopGbaAudioOutput,
    NoopGbaVideoOutput,
};

#[allow(dead_code)]
pub fn execute(original_source: &str) -> Gba {
    let preamble = ".text\n.arm\n.global _start\n_start:\n";
    let mut source = String::with_capacity(original_source.len() + preamble.len());
    source.push_str(preamble);
    source.push_str(original_source);
    println!("source:\n{source}\n");

    // Use cargo's temp file directory. Good to have this set epecially on Windows
    // where you're likely to have your code ignored by your antivirus (e.g. Windows Defender)
    arm_devkit::set_internal_tempfile_directory(env!("CARGO_TARGET_TMPDIR"));

    let mut gba = Gba::new();
    gba.set_gamepak(arm_devkit::arm::assemble(&source, simple_linker_script()).unwrap());
    gba.reset();

    let execution_ended: Arc<AtomicBool> = Arc::default();
    let execution_ended_from_handler = execution_ended.clone();
    gba.cpu
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
                    execution_ended_from_handler.store(true, atomic::Ordering::Release);
                    return ExceptionHandlerResult::Handled(Cycles::from(1));
                }
            }
            ExceptionHandlerResult::Ignored
        });

    let start_time = std::time::Instant::now();
    let mut steps_since_time_chek = 0;

    loop {
        if execution_ended.load(atomic::Ordering::Acquire) {
            break;
        }

        if steps_since_time_chek >= 1024 {
            if start_time.elapsed() > std::time::Duration::from_secs(5) {
                let next_pc = gba.cpu.next_execution_address();
                panic!("emulator timeout: 0x{next_pc:08X}");
            }
            steps_since_time_chek = 0;
        } else {
            steps_since_time_chek += 1;
        }

        gba.step(&mut NoopGbaVideoOutput, &mut NoopGbaAudioOutput);
    }

    gba
}

#[allow(dead_code)]
pub fn execute_until<P: AsRef<Path>, DF, VF, AF>(
    rom_path: P,
    mut done: DF,
    on_video_line: VF,
    _on_audio_line: AF,
) -> Gba
where
    DF: FnMut(&mut Gba) -> bool,
    VF: FnMut(usize, &LineBuffer),
    AF: FnMut(),
{
    let mut gba = Gba::new();
    gba.reset();
    let rom_path = rom_path.as_ref();
    gba.set_gamepak(std::fs::read(rom_path).expect("error reading ROM file"));
    let mut video_output = GbaVideoFnOutput::new(on_video_line);

    let execution_started = std::time::Instant::now();
    while !(done)(&mut gba) {
        if execution_started.elapsed() > std::time::Duration::from_secs(5) {
            let next_pc = gba.cpu.next_execution_address();
            panic!("emulator timeout: 0x{next_pc:08X}");
        }
        gba.step(&mut video_output, &mut NoopGbaAudioOutput);
    }

    gba
}

struct GbaVideoFnOutput<F> {
    f: F,
}

impl<F> GbaVideoFnOutput<F> {
    #[allow(dead_code)]
    fn new(f: F) -> Self {
        Self { f }
    }
}

impl<F> GbaVideoOutput for GbaVideoFnOutput<F>
where
    F: FnMut(usize, &LineBuffer),
{
    fn gba_line_ready(&mut self, line: usize, data: &LineBuffer) {
        (self.f)(line, data);
    }
}

#[allow(dead_code)]
pub fn video_noop(_: usize, _: &LineBuffer) {}
#[allow(dead_code)]
pub fn audio_noop() {}

#[allow(dead_code)]
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
macro_rules! emu_arm {
    ($source:expr) => {
        $crate::common::execute(&format!($source))
    };
}
