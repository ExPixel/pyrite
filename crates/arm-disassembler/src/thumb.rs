use std::fmt::Write;

pub fn disasm(instr: u16, address: u32) -> ThumbInstr {
    ThumbInstr::Undefined(instr)
}

#[derive(Debug)]
pub enum ThumbInstr {
    Undefined(u16),
}

impl ThumbInstr {
    pub(crate) fn write_mnemonic<W: Write>(&self, mut f: W) -> std::fmt::Result {
        match self {
            ThumbInstr::Undefined(_) => write!(f, "undef"),
        }
    }

    pub(crate) fn write_arguments<W: Write>(&self, mut f: W) -> std::fmt::Result {
        match self {
            ThumbInstr::Undefined(instr) => write!(f, "{:04x}", instr),
        }
    }

    pub(crate) fn write_comment<W: Write>(&self, mut _f: W) -> std::fmt::Result {
        match self {
            ThumbInstr::Undefined(_) => Ok(()),
        }
    }

    pub fn mnemonic(&self) -> crate::Mnemonic<'_, Self> {
        crate::Mnemonic(self)
    }

    pub fn arguments(&self) -> crate::Arguments<'_, Self> {
        crate::Arguments(self)
    }

    pub fn comment(&self) -> crate::Comment<'_, Self> {
        crate::Comment(self)
    }
}
