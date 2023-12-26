pub fn disasm(instr: u16) -> ThumbInstr {
    ThumbInstr::Undefined(instr)
}

#[derive(Debug)]
pub enum ThumbInstr {
    Undefined(u16),
}

impl ThumbInstr {
    pub(crate) fn write_mnemonic(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThumbInstr::Undefined(_) => f.pad("undef"),
        }
    }

    pub(crate) fn write_arguments(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThumbInstr::Undefined(instr) => <u16 as std::fmt::Display>::fmt(instr, f),
        }
    }

    pub(crate) fn write_comment(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
