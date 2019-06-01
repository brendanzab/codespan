#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};
use std::{fmt, ops};

/// 0-based line number
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "memory_usage", derive(heapsize_derive::HeapSizeOf))]
pub struct LineIndex(u32);

impl LineIndex {
    /// The 1-indexed line number. Useful for pretty printing source locations.
    ///
    /// ```rust
    /// use codespan::{LineIndex, LineNumber};
    ///
    /// assert_eq!(format!("{}", LineIndex::from(0).number()), "1");
    /// assert_eq!(format!("{}", LineIndex::from(3).number()), "4");
    /// ```
    pub const fn number(self) -> LineNumber {
        LineNumber(self.0 + 1)
    }

    /// Convert the index into a `usize`, for use in array indexing.
    pub fn to_usize(self) -> usize {
        self.0 as usize
    }
}

impl From<u32> for LineIndex {
    fn from(src: u32) -> LineIndex {
        LineIndex(src)
    }
}

impl ops::Add<LineSize> for LineIndex {
    type Output = LineIndex;

    fn add(self, other: LineSize) -> LineIndex {
        LineIndex::from(self.0 + other.0)
    }
}

impl ops::AddAssign<LineSize> for LineIndex {
    fn add_assign(&mut self, other: LineSize) {
        self.0 += other.0;
    }
}

impl ops::Sub<LineIndex> for LineIndex {
    type Output = LineSize;

    fn sub(self, other: LineIndex) -> LineSize {
        LineSize::from(self.0 - other.0)
    }
}

impl fmt::Debug for LineIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LineIndex(")?;
        self.0.fmt(f)?;
        write!(f, ")")
    }
}

impl fmt::Display for LineIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// A 1-indexed line number. Useful for pretty printing source locations.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "memory_usage", derive(heapsize_derive::HeapSizeOf))]
pub struct LineNumber(u32);

impl fmt::Debug for LineNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LineNumber(")?;
        self.0.fmt(f)?;
        write!(f, ")")
    }
}

impl fmt::Display for LineNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "memory_usage", derive(heapsize_derive::HeapSizeOf))]
pub struct LineSize(u32);

impl LineSize {
    pub fn to_usize(self) -> usize {
        self.0 as usize
    }
}

impl ops::Add<LineSize> for LineSize {
    type Output = LineSize;

    fn add(self, other: LineSize) -> LineSize {
        LineSize::from(self.0 + other.0)
    }
}

impl ops::AddAssign<LineSize> for LineSize {
    fn add_assign(&mut self, other: LineSize) {
        self.0 += other.0;
    }
}

impl From<u32> for LineSize {
    fn from(src: u32) -> LineSize {
        LineSize(src)
    }
}

impl fmt::Debug for LineSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LineSize(")?;
        self.0.fmt(f)?;
        write!(f, ")")
    }
}

impl fmt::Display for LineSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
