use tracing::field::DisplayValue;

pub struct Hex<T> {
    pub value: T,
}

impl std::fmt::Display for Hex<u32> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:08X}", self.value)
    }
}

impl std::fmt::Display for Hex<u16> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:04X}", self.value)
    }
}

impl std::fmt::Display for Hex<u8> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:02X}", self.value)
    }
}

pub fn hex<T>(value: T) -> DisplayValue<Hex<T>>
where
    Hex<T>: std::fmt::Display,
{
    tracing::field::display(Hex { value })
}
