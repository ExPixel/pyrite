mod arm;
mod lookup;
mod thumb;

pub use arm::{disasm_arm, ArmInstruction};
pub use thumb::{disasm_thumb, ThumbInstruction};

pub struct DisasmOptions {
    uppercase_mnemonic: bool,
}

pub struct Mnemonic<'a, T>(&'a T, &'a DisasmOptions);

impl std::fmt::Display for Mnemonic<'_, ArmInstruction> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.write_mnemonic(f, self.1)
    }
}

pub struct Arguments<'a, T>(&'a T, &'a DisasmOptions);

impl std::fmt::Display for Arguments<'_, ArmInstruction> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.write_args(f, self.1)
    }
}
