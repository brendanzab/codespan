#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};
use std::{fmt, ops};
use unicode_segmentation::UnicodeSegmentation;

use crate::{ByteIndex, ByteSize};

/// 0-based column number, segmented using grapheme clusters.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "memory_usage", derive(heapsize_derive::HeapSizeOf))]
pub struct ColumnIndex(u32);

impl ColumnIndex {
    pub fn from_str(
        src: &str,
        line_start_byte: ByteIndex,
        column_byte: ByteIndex,
    ) -> Option<ColumnIndex> {
        let line_src = &src.get(line_start_byte.to_usize()..column_byte.to_usize())?;
        Some(ColumnIndex::from(line_src.graphemes(true).count() as u32))
    }

    /// The 1-indexed column number. Useful for pretty printing source locations.
    ///
    /// ```rust
    /// use codespan::{ColumnIndex, ColumnNumber};
    ///
    /// assert_eq!(format!("{}", ColumnIndex::from(0).number()), "1");
    /// assert_eq!(format!("{}", ColumnIndex::from(3).number()), "4");
    /// ```
    pub const fn number(self) -> ColumnNumber {
        ColumnNumber(self.0 + 1)
    }

    /// Convert the index into a `usize`, for use in array indexing.
    pub fn to_usize(self) -> usize {
        self.0 as usize
    }

    /// Convert to a byte size, based on a unicode string.
    pub fn to_byte_size(self, line_src: &str) -> ByteSize {
        line_src
            .graphemes(true)
            .map(ByteSize::from_str_len_utf8)
            .fold(ByteSize::from(0), |acc, size| acc + size)
    }

    /// Convert to a byte index, based on a unicode string and a starting index.
    pub fn to_byte_index(self, src: &str, line_start_byte: ByteIndex) -> ByteIndex {
        line_start_byte + self.to_byte_size(&src[line_start_byte.to_usize()..])
    }
}

impl From<u32> for ColumnIndex {
    fn from(src: u32) -> ColumnIndex {
        ColumnIndex(src)
    }
}

impl ops::Add<ColumnSize> for ColumnIndex {
    type Output = ColumnIndex;

    fn add(self, other: ColumnSize) -> ColumnIndex {
        ColumnIndex::from(self.0 + other.0)
    }
}

impl ops::AddAssign<ColumnSize> for ColumnIndex {
    fn add_assign(&mut self, other: ColumnSize) {
        self.0 += other.0;
    }
}

impl ops::Sub<ColumnIndex> for ColumnIndex {
    type Output = ColumnSize;

    fn sub(self, other: ColumnIndex) -> ColumnSize {
        ColumnSize::from(self.0 - other.0)
    }
}

impl fmt::Debug for ColumnIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ColumnIndex(")?;
        self.0.fmt(f)?;
        write!(f, ")")
    }
}

impl fmt::Display for ColumnIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// A 1-indexed column number. Useful for pretty printing source locations.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "memory_usage", derive(heapsize_derive::HeapSizeOf))]
pub struct ColumnNumber(u32);

impl fmt::Debug for ColumnNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ColumnNumber(")?;
        self.0.fmt(f)?;
        write!(f, ")")
    }
}

impl fmt::Display for ColumnNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "memory_usage", derive(heapsize_derive::HeapSizeOf))]
pub struct ColumnSize(u32);

impl ColumnSize {
    pub fn to_usize(self) -> usize {
        self.0 as usize
    }
}

impl From<u32> for ColumnSize {
    fn from(src: u32) -> ColumnSize {
        ColumnSize(src)
    }
}

impl ops::Add<ColumnSize> for ColumnSize {
    type Output = ColumnSize;

    fn add(self, other: ColumnSize) -> ColumnSize {
        ColumnSize::from(self.0 + other.0)
    }
}

impl ops::AddAssign<ColumnSize> for ColumnSize {
    fn add_assign(&mut self, other: ColumnSize) {
        self.0 += other.0;
    }
}

impl fmt::Debug for ColumnSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ColumnSize(")?;
        self.0.fmt(f)?;
        write!(f, ")")
    }
}

impl fmt::Display for ColumnSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
