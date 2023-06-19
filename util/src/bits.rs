use std::ops::{Add, BitAnd, BitOr, Not, RangeBounds, Shl, Shr, Sub};

pub trait BitOps:
    Sized
    + BitOr<Output = Self>
    + BitAnd<Output = Self>
    + Not<Output = Self>
    + Shl<u32, Output = Self>
    + Shr<u32, Output = Self>
    + Add<Output = Self>
    + Sub<Output = Self>
    + PartialEq
    + Copy
{
    const ONE: Self;
    const ZERO: Self;
    const MAX: Self;
    const BITS: u32;

    #[inline(always)]
    fn get_bit<I: IntoBitIndex>(self, index: I) -> bool {
        (self & (Self::ONE << index.into_bit_index())) != Self::ZERO
    }

    #[inline(always)]
    fn put_bit<I: IntoBitIndex, B: IntoBit>(self, index: I, bit: B) -> Self {
        if bit.into_bit() {
            self.set_bit(index)
        } else {
            self.clear_bit(index)
        }
    }

    #[inline(always)]
    fn set_bit<I: IntoBitIndex>(self, index: I) -> Self {
        let index = index.into_bit_index();
        self | (Self::ONE << index)
    }

    #[inline(always)]
    fn clear_bit<I: IntoBitIndex>(self, index: I) -> Self {
        let index = index.into_bit_index();
        self & !(Self::ONE << index)
    }

    #[inline(always)]
    fn mask(size: u32) -> Self {
        if size < Self::BITS {
            (Self::ONE << size) - Self::ONE
        } else {
            Self::MAX
        }
    }

    #[inline]
    fn get_bit_range<R: RangeBounds<I>, I: IntoBitIndex + Copy>(self, range: R) -> Self {
        let start = match range.start_bound() {
            std::ops::Bound::Included(v) => v.into_bit_index(),
            std::ops::Bound::Excluded(v) => v.into_bit_index() + 1,
            std::ops::Bound::Unbounded => 0u32,
        };

        let end = match range.end_bound() {
            std::ops::Bound::Included(v) => v.into_bit_index(),
            std::ops::Bound::Excluded(v) => v.into_bit_index() - 1,
            std::ops::Bound::Unbounded => Self::BITS - 1,
        };

        let mask = Self::mask(end - start + 1);

        (self >> start) & mask
    }

    #[inline]
    fn put_bit_range<R: RangeBounds<I>, I: IntoBitIndex + Copy>(
        self,
        range: R,
        value: Self,
    ) -> Self {
        let start = match range.start_bound() {
            std::ops::Bound::Included(v) => v.into_bit_index(),
            std::ops::Bound::Excluded(v) => v.into_bit_index() + 1,
            std::ops::Bound::Unbounded => 0u32,
        };
        let end = match range.end_bound() {
            std::ops::Bound::Included(v) => v.into_bit_index(),
            std::ops::Bound::Excluded(v) => v.into_bit_index() - 1,
            std::ops::Bound::Unbounded => Self::BITS - 1,
        };
        let mask = Self::mask(end - start + 1);

        (self & !(mask << start)) | ((value & mask) << start)
    }
}

pub trait IntoBitIndex {
    fn into_bit_index(self) -> u32;
}

pub trait IntoBit {
    fn into_bit(self) -> bool;
}

impl BitOps for u8 {
    const ONE: Self = 1;
    const ZERO: Self = 0;
    const MAX: Self = u8::MAX;
    const BITS: u32 = 8;
}

impl BitOps for u16 {
    const ONE: Self = 1;
    const ZERO: Self = 0;
    const MAX: Self = u16::MAX;
    const BITS: u32 = 16;
}

impl BitOps for u32 {
    const ONE: Self = 1;
    const ZERO: Self = 0;
    const MAX: Self = u32::MAX;
    const BITS: u32 = 32;
}

impl BitOps for u64 {
    const ONE: Self = 1;
    const ZERO: Self = 0;
    const MAX: Self = u64::MAX;
    const BITS: u32 = 64;
}

impl BitOps for u128 {
    const ONE: Self = 1;
    const ZERO: Self = 0;
    const MAX: Self = u128::MAX;
    const BITS: u32 = 128;
}

impl BitOps for i8 {
    const ONE: Self = 1;
    const ZERO: Self = 0;
    const MAX: Self = i8::MAX;
    const BITS: u32 = 8;
}

impl BitOps for i16 {
    const ONE: Self = 1;
    const ZERO: Self = 0;
    const MAX: Self = i16::MAX;
    const BITS: u32 = 16;
}

impl BitOps for i32 {
    const ONE: Self = 1;
    const ZERO: Self = 0;
    const MAX: Self = i32::MAX;
    const BITS: u32 = 32;
}

impl BitOps for i64 {
    const ONE: Self = 1;
    const ZERO: Self = 0;
    const MAX: Self = i64::MAX;
    const BITS: u32 = 64;
}

impl BitOps for i128 {
    const ONE: Self = 1;
    const ZERO: Self = 0;
    const MAX: Self = i128::MAX;
    const BITS: u32 = 128;
}

impl IntoBit for bool {
    #[inline(always)]
    fn into_bit(self) -> bool {
        self
    }
}

impl IntoBit for u8 {
    #[inline(always)]
    fn into_bit(self) -> bool {
        self != 0
    }
}

impl IntoBit for u16 {
    #[inline(always)]
    fn into_bit(self) -> bool {
        self != 0
    }
}

impl IntoBit for u32 {
    #[inline(always)]
    fn into_bit(self) -> bool {
        self != 0
    }
}

impl IntoBit for i8 {
    #[inline(always)]
    fn into_bit(self) -> bool {
        self != 0
    }
}

impl IntoBit for i16 {
    #[inline(always)]
    fn into_bit(self) -> bool {
        self != 0
    }
}

impl IntoBit for i32 {
    #[inline(always)]
    fn into_bit(self) -> bool {
        self != 0
    }
}

impl IntoBitIndex for u8 {
    #[inline(always)]
    fn into_bit_index(self) -> u32 {
        self as u32
    }
}

impl IntoBitIndex for u16 {
    #[inline(always)]
    fn into_bit_index(self) -> u32 {
        self as u32
    }
}

impl IntoBitIndex for u32 {
    #[inline(always)]
    fn into_bit_index(self) -> u32 {
        self
    }
}

impl IntoBitIndex for i32 {
    #[inline(always)]
    fn into_bit_index(self) -> u32 {
        self.min(0) as u32
    }
}
