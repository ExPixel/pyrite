pub mod arm;
pub mod common;
pub mod thumb;

pub enum AnyInstr {
    Arm(arm::ArmInstr),
    Thumb(thumb::ThumbInstr),
}

impl AnyInstr {
    pub fn mnemonic(&self) -> crate::Mnemonic<'_, Self> {
        Mnemonic(self)
    }

    pub fn arguments<'s>(
        &'s self,
        addr: u32,
        m: Option<&'s dyn MemoryView>,
    ) -> crate::Arguments<'s, 's, Self> {
        Arguments(self, addr, m)
    }

    pub fn comment<'s>(
        &'s self,
        addr: u32,
        m: Option<&'s dyn MemoryView>,
    ) -> crate::Comment<'s, 's, Self> {
        Comment(self, addr, m)
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

#[derive(Clone, Copy)]
pub struct Arguments<'i, 'm, I>(&'i I, u32, Option<&'m dyn MemoryView>);

#[derive(Clone, Copy)]
pub struct Comment<'i, 'm, I>(&'i I, u32, Option<&'m dyn MemoryView>);

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

impl std::fmt::Display for Arguments<'_, '_, arm::ArmInstr> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buffer = WriteBuffer::<32>::new();
        self.0.write_arguments(&mut buffer)?;
        f.pad(buffer.as_str())
    }
}

impl std::fmt::Display for Arguments<'_, '_, thumb::ThumbInstr> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buffer = WriteBuffer::<32>::new();
        self.0.write_arguments(&mut buffer, self.1, self.2)?;
        f.pad(buffer.as_str())
    }
}

impl std::fmt::Display for Arguments<'_, '_, AnyInstr> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            AnyInstr::Arm(instr) => Arguments(instr, self.1, self.2).fmt(f),
            AnyInstr::Thumb(instr) => Arguments(instr, self.1, self.2).fmt(f),
        }
    }
}

impl std::fmt::Display for Comment<'_, '_, arm::ArmInstr> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buffer = WriteBuffer::<32>::new();
        self.0.write_comment(&mut buffer, self.1, self.2)?;
        f.pad(buffer.as_str())
    }
}

impl std::fmt::Display for Comment<'_, '_, thumb::ThumbInstr> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buffer = WriteBuffer::<32>::new();
        self.0.write_comment(&mut buffer, self.1, self.2)?;
        f.pad(buffer.as_str())
    }
}

impl std::fmt::Display for Comment<'_, '_, AnyInstr> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            AnyInstr::Arm(instr) => Comment(instr, self.1, self.2).fmt(f),
            AnyInstr::Thumb(instr) => Comment(instr, self.1, self.2).fmt(f),
        }
    }
}

impl<I: std::fmt::Debug> std::fmt::Debug for Comment<'_, '_, I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Comment")
            .field(&self.0)
            .field(&self.1)
            .field(&self.2.map(|_| "<memory>"))
            .finish()
    }
}

impl<I: std::fmt::Debug> std::fmt::Debug for Arguments<'_, '_, I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Arguments")
            .field(&self.0)
            .field(&self.1)
            .field(&self.2.map(|_| "<memory>"))
            .finish()
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

pub trait MemoryView {
    fn view8(&self, address: u32) -> u8;
    fn view16(&self, address: u32) -> u16;
    fn view32(&self, address: u32) -> u32;
}

impl MemoryView for &'_ [u8] {
    fn view8(&self, address: u32) -> u8 {
        self.get(address as usize).copied().unwrap_or(0)
    }

    fn view16(&self, address: u32) -> u16 {
        u16::from_le_bytes([
            self.get(address as usize).copied().unwrap_or(0),
            self.get((address.wrapping_add(1)) as usize)
                .copied()
                .unwrap_or(0),
        ])
    }

    fn view32(&self, address: u32) -> u32 {
        u32::from_le_bytes([
            self.get(address as usize).copied().unwrap_or(0),
            self.get((address.wrapping_add(1)) as usize)
                .copied()
                .unwrap_or(0),
            self.get((address.wrapping_add(2)) as usize)
                .copied()
                .unwrap_or(0),
            self.get((address.wrapping_add(3)) as usize)
                .copied()
                .unwrap_or(0),
        ])
    }
}
