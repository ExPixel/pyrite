use std::{
    collections::HashMap,
    ffi::OsStr,
    io::{self, Write},
    path::{Path, PathBuf},
    process::{self, Command},
    sync::{Arc, Mutex, OnceLock, Weak},
};

use tempfile::{NamedTempFile, TempPath};

fn find_arm_binary_uncached(name: &str) -> Option<PathBuf> {
    let arm_none_eabi_name = PathBuf::from(&(format!("arm-none-eabi-{name}")));

    #[cfg(target_os = "windows")]
    let arm_none_eabi_name_exe = PathBuf::from(&(format!("arm-none-eabi-{name}.exe")));

    if let Ok(path) = which::which(&arm_none_eabi_name) {
        return Some(path);
    }

    if let Ok(arm_binaries_path) = std::env::var("ARM_BINARIES_DIR") {
        let path = PathBuf::from(&arm_binaries_path).join(&arm_none_eabi_name);
        if path.exists() {
            return Some(path);
        }

        #[cfg(target_os = "windows")]
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

    #[cfg(target_os = "windows")]
    {
        let path = devkitarm_path.join(&arm_none_eabi_name_exe);
        if path.exists() {
            return Some(path);
        }

        // The path for devkitPro is usually /opt/devkitPro
        if let Ok(path_without_opt) = path.strip_prefix("/opt") {
            let path_from_c = Path::new("C:\\").join(path_without_opt);
            if path_from_c.exists() {
                return Some(path_from_c);
            }
        }
    }

    let devkitarm_bin_path = devkitarm_path.join("bin");

    let path = devkitarm_bin_path.join(&arm_none_eabi_name);
    if path.exists() {
        return Some(path);
    }

    #[cfg(target_os = "windows")]
    {
        let path = devkitarm_bin_path.join(&arm_none_eabi_name_exe);
        if path.exists() {
            return Some(path);
        }

        // The path for devkitARM is usually /opt/devkitPro/devkitARM
        if let Ok(path_without_opt) = path.strip_prefix("/opt") {
            let path_from_c = Path::new("C:\\").join(path_without_opt);
            if path_from_c.exists() {
                return Some(path_from_c);
            }
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

fn run_arm_executable(
    name: &str,
    args: &[&OsStr],
    stdin: Option<&str>,
) -> io::Result<process::ExitStatus> {
    println!("executing: {name:?} {args:?}");

    let binary_path = find_arm_binary(name)
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "binary for program not found"))?;

    let mut cmd = Command::new(binary_path);
    cmd.args(args);
    cmd.stdout(process::Stdio::piped());
    cmd.stderr(process::Stdio::piped());
    if stdin.is_some() {
        cmd.stdin(process::Stdio::piped());
    } else {
        cmd.stdin(process::Stdio::null());
    }
    let child = cmd.spawn()?;

    if let Some(stdin) = stdin {
        child
            .stdin
            .as_ref()
            .expect("no stdin")
            .write_all(stdin.as_bytes())?;
    }
    let output = child.wait_with_output()?;

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

static INTERNAL_TEMPFILE_DIRECTORY: OnceLock<PathBuf> = OnceLock::new();

pub fn set_internal_tempfile_directory<P: AsRef<Path>>(path: P) {
    let _ = INTERNAL_TEMPFILE_DIRECTORY.set(path.as_ref().into());
}

fn get_tempfile_directory() -> Option<&'static Path> {
    INTERNAL_TEMPFILE_DIRECTORY.get().map(|p| p.as_path())
}

fn tempfile_internal() -> io::Result<NamedTempFile> {
    if let Some(tmpdir) = get_tempfile_directory() {
        NamedTempFile::new_in(tmpdir)
    } else {
        NamedTempFile::new()
    }
}

fn temppath_internal() -> io::Result<TempPath> {
    tempfile_internal().map(|file| file.into_temp_path())
}

pub mod arm {
    use crate::temppath_internal;

    use super::{run_arm_executable, LinkerScript};
    use std::{borrow::Cow, io, path::Path};

    pub fn assemble(source: &str, linker_script: LinkerScript) -> io::Result<Vec<u8>> {
        let mut source = Cow::Borrowed(source);
        if !source.ends_with('\n') {
            let mut new_source = String::with_capacity(source.len() + 1);
            new_source.push_str(&source);
            new_source.push('\n');
            source = Cow::Owned(new_source);
        }
        let linker_script_path: &Path = &linker_script.0;

        let object_file_path = temppath_internal()?;
        let as_args = &[
            "-mcpu=arm7tdmi".as_ref(),
            "-march=armv4t".as_ref(),
            "-mthumb-interwork".as_ref(),
            "-o".as_ref(),
            object_file_path.as_ref(),
        ];
        let status = run_arm_executable("as", as_args, Some(&*source))?;
        if !status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, "failed to assemble"));
        }

        let elf_file_path = temppath_internal()?;
        let ld_args = &[
            "-T".as_ref(),
            linker_script_path.as_ref(),
            "-o".as_ref(),
            elf_file_path.as_ref(),
            object_file_path.as_ref(),
        ];
        let status = run_arm_executable("ld", ld_args, None)?;
        if !status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, "failed to link"));
        }

        let bin_file_path = temppath_internal()?;
        let objcopy_args = &[
            "-O".as_ref(),
            "binary".as_ref(),
            elf_file_path.as_ref(),
            bin_file_path.as_ref(),
        ];
        let status = run_arm_executable("objcopy", objcopy_args, None)?;
        if !status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, "failed to objcopy"));
        }

        let objdump_args = &[
            "-b".as_ref(),
            "binary".as_ref(),
            "-m".as_ref(),
            "armv4t".as_ref(),
            "--adjust-vma=0x0".as_ref(),
            "-D".as_ref(),
            bin_file_path.as_ref(),
        ];
        let status = run_arm_executable("objdump", objdump_args, None)?;
        if !status.success() {
            let message = "failed to objdump (disassemble)";
            return Err(io::Error::new(io::ErrorKind::Other, message));
        }

        std::fs::read(bin_file_path)
    }
}

pub mod thumb {
    use crate::temppath_internal;

    use super::{run_arm_executable, LinkerScript};
    use std::{borrow::Cow, io, path::Path};

    pub fn assemble(source: &str, linker_script: LinkerScript) -> io::Result<Vec<u8>> {
        let mut source = Cow::Borrowed(source);
        if !source.ends_with('\n') {
            let mut new_source = String::with_capacity(source.len() + 1);
            new_source.push_str(&source);
            new_source.push('\n');
            source = Cow::Owned(new_source);
        }
        let linker_script_path: &Path = &linker_script.0;

        let object_file_path = temppath_internal()?;
        let as_args = &[
            "-mthumb".as_ref(),
            "-mcpu=arm7tdmi".as_ref(),
            "-march=armv4t".as_ref(),
            "-mthumb-interwork".as_ref(),
            "-o".as_ref(),
            object_file_path.as_ref(),
        ];
        let status = run_arm_executable("as", as_args, Some(&*source))?;
        if !status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, "failed to assemble"));
        }

        let elf_file_path = temppath_internal()?;
        let ld_args = &[
            "-T".as_ref(),
            linker_script_path.as_ref(),
            "-o".as_ref(),
            elf_file_path.as_ref(),
            object_file_path.as_ref(),
        ];
        let status = run_arm_executable("ld", ld_args, None)?;
        if !status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, "failed to link"));
        }

        let bin_file_path = temppath_internal()?;
        let objcopy_args = &[
            "-O".as_ref(),
            "binary".as_ref(),
            elf_file_path.as_ref(),
            bin_file_path.as_ref(),
        ];
        let status = run_arm_executable("objcopy", objcopy_args, None)?;
        if !status.success() {
            return Err(io::Error::new(io::ErrorKind::Other, "failed to objcopy"));
        }

        let objdump_args = &[
            "-b".as_ref(),
            "binary".as_ref(),
            "-m".as_ref(),
            "armv4t".as_ref(),
            "-Mforce-thumb".as_ref(),
            "--adjust-vma=0x0".as_ref(),
            "-z".as_ref(),
            "-D".as_ref(),
            bin_file_path.as_ref(),
        ];
        let status = run_arm_executable("objdump", objdump_args, None)?;
        if !status.success() {
            let message = "failed to objdump (disassemble)";
            return Err(io::Error::new(io::ErrorKind::Other, message));
        }

        std::fs::read(bin_file_path)
    }
}

#[derive(Clone)]
pub struct LinkerScript(Arc<TempPath>);
#[derive(Clone)]
pub struct LinkerScriptWeakRef(Weak<TempPath>);

impl LinkerScript {
    pub fn new(source: &str) -> io::Result<LinkerScript> {
        let mut file = tempfile_internal()?;
        file.write_all(source.as_bytes())?;
        Ok(LinkerScript(Arc::new(file.into_temp_path())))
    }

    pub fn weak(&self) -> LinkerScriptWeakRef {
        LinkerScriptWeakRef(Arc::downgrade(&self.0))
    }
}

impl LinkerScriptWeakRef {
    pub fn upgrade(&self) -> Option<LinkerScript> {
        self.0.upgrade().map(LinkerScript)
    }
}

pub const SIMPLE_LINKER_SCRIPT: &str = r#"
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

    /* The BSS section for uninitialized data */
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
}"#;
