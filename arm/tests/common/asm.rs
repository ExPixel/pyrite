use std::{
    collections::HashMap,
    ffi::OsStr,
    io,
    path::{Path, PathBuf},
    process::{self, Command},
    sync::{atomic::AtomicU32, Mutex, OnceLock},
};

use arm::InstructionSet;

fn find_arm_binary_uncached(name: &str) -> Option<PathBuf> {
    let arm_none_eabi_name = PathBuf::from(&(format!("arm-none-eabi-{name}")));

    #[cfg(target = "windows")]
    let arm_none_eabi_name_exe = PathBuf::from(&(format!("arm-none-eabi-{name}.exe")));

    if let Ok(path) = which::which(&arm_none_eabi_name) {
        return Some(path);
    }

    if let Ok(arm_binaries_path) = std::env::var("ARM_BINARIES_DIR") {
        let path = PathBuf::from(&arm_binaries_path).join(&arm_none_eabi_name);
        if path.exists() {
            return Some(path);
        }

        #[cfg(target = "windows")]
        {
            let path = PathBuf::from(&arm_binaries_path).join(&arm_none_eabi_name_exe);
            if path.exists() {
                return Some(path);
            }
        }
    }

    let devkitarm_path = if let Ok(devkitarm_path) = std::env::var("DEVKITARM") {
        PathBuf::from(devkitarm_path)
    } else if let Ok(devkitpro_path) = std::env::var("DEVKITPRO") {
        PathBuf::from(devkitpro_path).join("devkitARM")
    } else {
        return None;
    };

    let path = devkitarm_path.join(&arm_none_eabi_name);
    if path.exists() {
        return Some(path);
    }

    #[cfg(target = "windows")]
    {
        let path = devkitarm_path.join(&arm_none_eabi_name_exe);
        if path.exists() {
            return Some(path);
        }
    }

    let devkitarm_bin_path = devkitarm_path.join("bin");

    let path = devkitarm_bin_path.join(&arm_none_eabi_name);
    if path.exists() {
        return Some(path);
    }

    #[cfg(target = "windows")]
    {
        let path = devkitarm_bin_path.join(&arm_none_eabi_name_exe);
        if path.exists() {
            return Some(path);
        }
    }

    None
}

fn find_arm_binary(name: &str) -> Option<PathBuf> {
    static ARM_BINARY_CACHE: OnceLock<Mutex<HashMap<String, PathBuf>>> = OnceLock::new();

    let cache = ARM_BINARY_CACHE.get_or_init(Default::default);
    let mut cache = cache.lock().unwrap();

    if let Some(path) = cache.get(name) {
        return Some(path.clone());
    }

    if let Some(uncached_path) = find_arm_binary_uncached(name) {
        cache.insert(name.to_owned(), uncached_path.clone());
        Some(uncached_path)
    } else {
        None
    }
}

fn run_arm_executable(name: &str, args: &[&OsStr]) -> io::Result<process::ExitStatus> {
    println!("executing: {name:?} {args:?}");

    let binary_path = find_arm_binary(name)
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "binary for program not found"))?;

    let output = Command::new(binary_path)
        .args(args)
        .stdout(process::Stdio::piped())
        .stderr(process::Stdio::piped())
        .output()?;

    let mut had_output = false;

    let in_obj_dump = name.eq_ignore_ascii_case("objdump");
    let mut in_obj_dump_preamble = true;

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if !in_obj_dump {
            println!("  out: {}", line.trim_end());
            had_output = true;
            continue;
        }

        // For objdump we do some special formatting for the output:
        if !in_obj_dump_preamble {
            println!("    {}", line.trim());
            had_output = true;
        }

        // After we encounter one of these lines:
        //    00000000 <.data>:
        //    00000000 <.text>:
        // we are no longer in the preamble.
        if line.contains(">:") {
            println!("  {}", line.trim());
            in_obj_dump_preamble = false;
            had_output = true;
        }
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    for (idx, line) in stderr.lines().enumerate() {
        if idx == 0 && had_output {
            println!()
        }
        println!("  err: {}", line.trim_end());
    }

    Ok(output.status)
}

pub fn assemble(isa: InstructionSet, source: &str) -> std::io::Result<Vec<u8>> {
    use rand::Rng as _;

    let tmp_dir = Path::new(env!("CARGO_TARGET_TMPDIR"));

    static FILENAME_INCREMENT: AtomicU32 = AtomicU32::new(0);

    let mut rng = rand::thread_rng();
    let cnt = FILENAME_INCREMENT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let rnd = rng.gen_range(0u64..=u64::MAX);

    let fname = format!("asm-{rnd}-{cnt}");

    let source_file_path = tmp_dir.join(format!("{fname}.s"));
    let object_file_path = tmp_dir.join(format!("{fname}.o"));
    let elf_file_path = tmp_dir.join(format!("{fname}.elf"));
    let bin_file_path = tmp_dir.join(format!("{fname}.bin"));

    let files_to_destroy = [
        &source_file_path as &Path,
        &object_file_path,
        &elf_file_path,
        &bin_file_path,
    ];
    let _file_destructor = FileDestructor::new(&files_to_destroy);

    let preamble = "\
    .global _start
    _start:\n";
    let mut new_source = String::with_capacity(preamble.len() + source.len() + 1);
    new_source.push_str(preamble);
    new_source.push_str(source);
    new_source.push('\n');
    let source = new_source;

    // `as` outputs a warning if the file does not end with a newline or if there
    // is no `_start:` symbol.

    std::fs::write(&source_file_path, source)?;

    let as_output = if isa == InstructionSet::Arm {
        run_arm_executable(
            "as",
            &[
                "-mcpu=arm7tdmi".as_ref(),
                "-march=armv4t".as_ref(),
                "-mthumb-interwork".as_ref(),
                "-o".as_ref(),
                object_file_path.as_ref(),
                source_file_path.as_ref(),
            ],
        )?
    } else {
        run_arm_executable(
            "as",
            &[
                "-mthumb".as_ref(),
                "-mcpu=arm7tdmi".as_ref(),
                "-march=armv4t".as_ref(),
                "-mthumb-interwork".as_ref(),
                "-o".as_ref(),
                object_file_path.as_ref(),
                source_file_path.as_ref(),
            ],
        )?
    };
    if !as_output.success() {
        panic!("failed to assemble {}", source_file_path.display());
    }

    if !run_arm_executable(
        "ld",
        &[
            "-T".as_ref(),
            simple_linker_script().as_ref(),
            "-o".as_ref(),
            elf_file_path.as_ref(),
            object_file_path.as_ref(),
        ],
    )?
    .success()
    {
        panic!("failed to link {}", object_file_path.display());
    }

    if !run_arm_executable(
        "objcopy",
        &[
            "-O".as_ref(),
            "binary".as_ref(),
            elf_file_path.as_ref(),
            bin_file_path.as_ref(),
        ],
    )?
    .success()
    {
        panic!("failed to extract binary from {}", elf_file_path.display());
    }

    let objdump_output = if isa == InstructionSet::Arm {
        run_arm_executable(
            "objdump",
            &[
                "-b".as_ref(),
                "binary".as_ref(),
                "-m".as_ref(),
                "armv4t".as_ref(),
                "--adjust-vma=0x0".as_ref(),
                "-D".as_ref(),
                bin_file_path.as_ref(),
            ],
        )?
    } else {
        run_arm_executable(
            "objdump",
            &[
                "-b".as_ref(),
                "binary".as_ref(),
                "-m".as_ref(),
                "armv4t".as_ref(),
                "-Mforce-thumb".as_ref(),
                "--adjust-vma=0x0".as_ref(),
                "-dj".as_ref(),
                ".text".as_ref(),
                bin_file_path.as_ref(),
            ],
        )?
    };
    if !objdump_output.success() {
        panic!("failed to disassemble binary {}", bin_file_path.display())
    }

    std::fs::read(&bin_file_path)
}

fn simple_linker_script() -> &'static Path {
    const SIMPLE_LINKER_SCRIPT: &str = "
    ENTRY(_start);
    SECTIONS
    {
        . = 0x0;

        /* Place special section .text.prologue before everything else */
        .text : {
            . = ALIGN(4);
            *(.text.prologue);
            *(.text*);
            . = ALIGN(4);
        }

        /* Output the data sections */
        .data : {
            . = ALIGN(4);
            *(.data*);
        }

        .rodata : {
            . = ALIGN(4);
            *(.rodata*);
        }

        /* The BSS section for uniitialized data */
        .bss : {
            . = ALIGN(4);
            __bss_start = .;
            *(COMMON);
            *(.bss);
            . = ALIGN(4);
            __bss_end = .;
        }

        /* Size of the BSS section in case it is needed */
        __bss_size = ((__bss_end)-(__bss_start));

        /* Remove the note that may be placed before the code by LD */
        /DISCARD/ : {
            *(.note.gnu.build-id);
            *(.ARM.attributes);
        }
    }
    ";
    static SIMPLE_LINKER_SCRIPT_PATH: OnceLock<PathBuf> = OnceLock::new();

    SIMPLE_LINKER_SCRIPT_PATH.get_or_init(|| {
        let tmp_dir = Path::new(env!("CARGO_TARGET_TMPDIR"));
        let path = tmp_dir.join("simple-linker-script.ld");
        std::fs::write(&path, SIMPLE_LINKER_SCRIPT).expect("failed to write simple linker script");
        path
    })
}

struct FileDestructor<'p> {
    paths: &'p [&'p Path],
}

impl<'p> FileDestructor<'p> {
    pub fn new(paths: &'p [&'p Path]) -> Self {
        FileDestructor { paths }
    }
}

impl<'p> Drop for FileDestructor<'p> {
    fn drop(&mut self) {
        for &path in self.paths {
            if !path.exists() {
                continue;
            }

            if let Err(err) = std::fs::remove_file(path) {
                eprintln!(
                    "error occurred while deleting path `{}`: {}",
                    path.display(),
                    err
                );
            }
        }
    }
}
