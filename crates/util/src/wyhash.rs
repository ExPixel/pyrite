pub struct WyHash {
    seed: u64,
}

impl WyHash {
    pub fn new(seed: u64) -> Self {
        WyHash { seed }
    }

    pub fn generate<T: GeneratedByWyHash>(&mut self) -> T {
        T::generate_by_wyhash(self)
    }

    pub fn next_rand(&mut self) -> u64 {
        self.seed = self.seed.wrapping_add(0x60bee2bee120fc15);
        let mut tmp = (self.seed as u128).wrapping_mul(0xa3b195354a39b70d);
        let m1 = ((tmp >> 64) ^ tmp) as u64;
        tmp = (m1 as u128).wrapping_mul(0x1b03738712fad5c9);

        ((tmp >> 64) ^ tmp) as u64 // m2
    }
}

impl Iterator for WyHash {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.next_rand())
    }
}

pub trait GeneratedByWyHash {
    fn generate_by_wyhash(wyhash: &mut WyHash) -> Self;
}

impl GeneratedByWyHash for u64 {
    fn generate_by_wyhash(wyhash: &mut WyHash) -> Self {
        wyhash.next_rand()
    }
}

impl GeneratedByWyHash for u32 {
    fn generate_by_wyhash(wyhash: &mut WyHash) -> Self {
        wyhash.next_rand() as u32
    }
}

impl GeneratedByWyHash for u16 {
    fn generate_by_wyhash(wyhash: &mut WyHash) -> Self {
        wyhash.next_rand() as u16
    }
}
