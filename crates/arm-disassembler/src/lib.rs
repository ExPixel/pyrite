mod arm;
mod lookup;
mod thumb;

pub use arm::{disasm_arm, ArmInstruction};
pub use thumb::{disasm_thumb, ThumbInstruction};

pub struct DisasmOptions {}
