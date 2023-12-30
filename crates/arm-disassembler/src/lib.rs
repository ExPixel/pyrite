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
