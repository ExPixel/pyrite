use std::ops::{Add, AddAssign};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Cycles(u32);

impl Cycles {
    #[inline]
    pub const fn zero() -> Self {
        Cycles(0)
    }

    #[inline]
    pub const fn one() -> Cycles {
        Cycles(1)
    }

    #[inline]
    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }
}

impl From<u32> for Cycles {
    #[inline]
    fn from(value: u32) -> Self {
        Cycles(value)
    }
}

impl From<Cycles> for u32 {
    #[inline]
    fn from(value: Cycles) -> Self {
        value.0
    }
}

impl Add for Cycles {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Cycles(self.0 + rhs.0)
    }
}

impl AddAssign for Cycles {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Add<Waitstates> for Cycles {
    type Output = Self;

    fn add(self, rhs: Waitstates) -> Self::Output {
        Cycles(self.0 + rhs.0)
    }
}

impl AddAssign<Waitstates> for Cycles {
    fn add_assign(&mut self, rhs: Waitstates) {
        self.0 += rhs.0
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Waitstates(u32);

impl Waitstates {
    #[inline]
    pub const fn zero() -> Self {
        Waitstates(0)
    }

    #[inline]
    pub const fn one() -> Waitstates {
        Waitstates(1)
    }

    #[inline]
    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }
}

impl From<u32> for Waitstates {
    #[inline]
    fn from(value: u32) -> Self {
        Waitstates(value)
    }
}

impl From<Waitstates> for u32 {
    #[inline]
    fn from(value: Waitstates) -> Self {
        value.0
    }
}

impl Add for Waitstates {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Waitstates(self.0 + rhs.0)
    }
}

impl AddAssign for Waitstates {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Add<Cycles> for Waitstates {
    type Output = Cycles;

    fn add(self, rhs: Cycles) -> Self::Output {
        Cycles(self.0 + rhs.0)
    }
}
