pub mod arm;
pub mod thumb;

pub enum AnyInstr {
    Arm(arm::ArmInstr),
    Thumb(thumb::ThumbInstr),
}

impl AnyInstr {
    pub fn mnemonic(&self) -> crate::Mnemonic<'_, Self> {
        Mnemonic(self)
    }

    pub fn arguments(&self) -> crate::Arguments<'_, Self> {
        Arguments(self)
    }

    pub fn comment(&self) -> crate::Comment<'_, Self> {
        Comment(self)
    }
}

impl From<arm::ArmInstr> for AnyInstr {
    fn from(instr: arm::ArmInstr) -> Self {
        Self::Arm(instr)
    }
}

impl From<thumb::ThumbInstr> for AnyInstr {
    fn from(instr: thumb::ThumbInstr) -> Self {
        Self::Thumb(instr)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Mnemonic<'i, I>(&'i I);

#[derive(Debug, Clone, Copy)]
pub struct Arguments<'i, I>(&'i I);

#[derive(Debug, Clone, Copy)]
pub struct Comment<'i, I>(&'i I);

impl std::fmt::Display for Mnemonic<'_, arm::ArmInstr> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buffer = WriteBuffer::<32>::new();
        self.0.write_mnemonic(&mut buffer)?;
        f.pad(buffer.as_str())
    }
}

impl std::fmt::Display for Mnemonic<'_, thumb::ThumbInstr> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buffer = WriteBuffer::<32>::new();
        self.0.write_mnemonic(&mut buffer)?;
        f.pad(buffer.as_str())
    }
}

impl std::fmt::Display for Mnemonic<'_, AnyInstr> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            AnyInstr::Arm(instr) => Mnemonic(instr).fmt(f),
            AnyInstr::Thumb(instr) => Mnemonic(instr).fmt(f),
        }
    }
}

impl std::fmt::Display for Arguments<'_, arm::ArmInstr> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buffer = WriteBuffer::<32>::new();
        self.0.write_arguments(&mut buffer)?;
        f.pad(buffer.as_str())
    }
}

impl std::fmt::Display for Arguments<'_, thumb::ThumbInstr> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buffer = WriteBuffer::<32>::new();
        self.0.write_arguments(&mut buffer)?;
        f.pad(buffer.as_str())
    }
}

impl std::fmt::Display for Arguments<'_, AnyInstr> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            AnyInstr::Arm(instr) => Arguments(instr).fmt(f),
            AnyInstr::Thumb(instr) => Arguments(instr).fmt(f),
        }
    }
}

impl std::fmt::Display for Comment<'_, arm::ArmInstr> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buffer = WriteBuffer::<32>::new();
        self.0.write_comment(&mut buffer)?;
        f.pad(buffer.as_str())
    }
}

impl std::fmt::Display for Comment<'_, thumb::ThumbInstr> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buffer = WriteBuffer::<32>::new();
        self.0.write_comment(&mut buffer)?;
        f.pad(buffer.as_str())
    }
}

impl std::fmt::Display for Comment<'_, AnyInstr> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            AnyInstr::Arm(instr) => Comment(instr).fmt(f),
            AnyInstr::Thumb(instr) => Comment(instr).fmt(f),
        }
    }
}

struct WriteBuffer<const N: usize> {
    len: usize,
    buffer: [u8; N],
}

impl<const N: usize> WriteBuffer<N> {
    fn new() -> Self {
        Self {
            len: 0,
            buffer: [0; N],
        }
    }

    pub fn as_str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.buffer[..self.len]) }
    }
}

impl std::fmt::Write for &'_ mut WriteBuffer<32> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        let bytes = s.as_bytes();

        if bytes.len() > self.buffer.len() - self.len {
            return Err(std::fmt::Error);
        }

        self.buffer[self.len..self.len + bytes.len()].copy_from_slice(bytes);
        self.len += bytes.len();
        Ok(())
    }
}
