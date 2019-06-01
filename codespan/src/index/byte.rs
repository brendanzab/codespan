#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};
use std::{fmt, ops};

/// Byte index into a text string
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "memory_usage", derive(heapsize_derive::HeapSizeOf))]
pub struct ByteIndex(u32);

impl ByteIndex {
    /// Convert the index into a `usize`, for use in array indexing
    pub fn to_usize(self) -> usize {
        self.0 as usize
    }
}

impl From<u32> for ByteIndex {
    fn from(src: u32) -> ByteIndex {
        ByteIndex(src)
    }
}

impl ops::Add<ByteSize> for ByteIndex {
    type Output = ByteIndex;

    fn add(self, other: ByteSize) -> ByteIndex {
        ByteIndex::from(self.0 + other.0)
    }
}

impl ops::AddAssign<ByteSize> for ByteIndex {
    fn add_assign(&mut self, other: ByteSize) {
        self.0 += other.0;
    }
}

impl ops::Sub<ByteIndex> for ByteIndex {
    type Output = ByteSize;

    fn sub(self, other: ByteIndex) -> ByteSize {
        ByteSize::from(self.0 - other.0)
    }
}

impl fmt::Debug for ByteIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ByteIndex(")?;
        self.0.fmt(f)?;
        write!(f, ")")
    }
}

impl fmt::Display for ByteIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "memory_usage", derive(heapsize_derive::HeapSizeOf))]
pub struct ByteSize(u32);

impl ByteSize {
    pub fn from_char_len_utf8(ch: char) -> ByteSize {
        ByteSize::from(ch.len_utf8() as u32)
    }

    pub fn from_char_len_utf16(ch: char) -> ByteSize {
        ByteSize::from(ch.len_utf16() as u32)
    }

    pub fn from_str_len_utf8(s: &str) -> ByteSize {
        ByteSize::from(s.len() as u32)
    }

    pub fn to_usize(self) -> usize {
        self.0 as usize
    }
}

impl ops::Add<ByteSize> for ByteSize {
    type Output = ByteSize;

    fn add(self, other: ByteSize) -> ByteSize {
        ByteSize::from(self.0 + other.0)
    }
}

impl ops::AddAssign<ByteSize> for ByteSize {
    fn add_assign(&mut self, other: ByteSize) {
        self.0 += other.0;
    }
}

impl From<u32> for ByteSize {
    fn from(src: u32) -> ByteSize {
        ByteSize(src)
    }
}

impl fmt::Debug for ByteSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ByteSize(")?;
        self.0.fmt(f)?;
        write!(f, ")")
    }
}

impl fmt::Display for ByteSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
