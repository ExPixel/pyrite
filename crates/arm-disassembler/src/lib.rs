pub mod arm;
pub mod thumb;

#[derive(Debug, Clone, Copy)]
pub struct Mnemonic<'i, I>(&'i I);

#[derive(Debug, Clone, Copy)]
pub struct Arguments<'i, I>(&'i I);

#[derive(Debug, Clone, Copy)]
pub struct Comment<'i, I>(&'i I);

impl std::fmt::Display for Mnemonic<'_, arm::ArmInstr> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.write_mnemonic(f)
    }
}

impl std::fmt::Display for Mnemonic<'_, thumb::ThumbInstr> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.write_mnemonic(f)
    }
}

impl std::fmt::Display for Arguments<'_, arm::ArmInstr> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.write_arguments(f)
    }
}

impl std::fmt::Display for Arguments<'_, thumb::ThumbInstr> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.write_arguments(f)
    }
}

impl std::fmt::Display for Comment<'_, arm::ArmInstr> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.write_comment(f)
    }
}

impl std::fmt::Display for Comment<'_, thumb::ThumbInstr> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.write_comment(f)
    }
}
