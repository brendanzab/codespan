//! Wrapper types that specify positions in a source file

#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, AddAssign, Neg, Sub, SubAssign};

/// The raw, untyped index. We use a 32-bit integer here for space efficiency,
/// assuming we won't be working with sources larger than 4GB.
pub type RawIndex = u32;

/// The raw, untyped offset.
pub type RawOffset = i64;

/// A zero-indexed line offset into a source file
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
pub struct LineIndex(pub RawIndex);

impl LineIndex {
    /// The 1-indexed line number. Useful for pretty printing source locations.
    ///
    /// ```rust
    /// use codespan::{LineIndex, LineNumber};
    ///
    /// assert_eq!(format!("{}", LineIndex(0).number()), "1");
    /// assert_eq!(format!("{}", LineIndex(3).number()), "4");
    /// ```
    pub const fn number(self) -> LineNumber {
        LineNumber(self.0 + 1)
    }

    /// Convert the index into a `usize`, for use in array indexing
    pub const fn to_usize(self) -> usize {
        self.0 as usize
    }
}

impl Default for LineIndex {
    fn default() -> LineIndex {
        LineIndex(0)
    }
}

impl fmt::Debug for LineIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
pub struct LineNumber(RawIndex);

impl LineNumber {
    /// Convert the number into a `usize`
    pub const fn to_usize(self) -> usize {
        self.0 as usize
    }
}

impl fmt::Debug for LineNumber {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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

/// A line offset in a source file
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
pub struct LineOffset(pub RawOffset);

impl Default for LineOffset {
    fn default() -> LineOffset {
        LineOffset(0)
    }
}

impl fmt::Debug for LineOffset {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "LineOffset(")?;
        self.0.fmt(f)?;
        write!(f, ")")
    }
}

impl fmt::Display for LineOffset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// A zero-indexed column offset into a source file
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
pub struct ColumnIndex(pub RawIndex);

impl ColumnIndex {
    /// The 1-indexed column number. Useful for pretty printing source locations.
    ///
    /// ```rust
    /// use codespan::{ColumnIndex, ColumnNumber};
    ///
    /// assert_eq!(format!("{}", ColumnIndex(0).number()), "1");
    /// assert_eq!(format!("{}", ColumnIndex(3).number()), "4");
    /// ```
    pub const fn number(self) -> ColumnNumber {
        ColumnNumber(self.0 + 1)
    }

    /// Convert the index into a `usize`, for use in array indexing
    pub const fn to_usize(self) -> usize {
        self.0 as usize
    }
}

impl Default for ColumnIndex {
    fn default() -> ColumnIndex {
        ColumnIndex(0)
    }
}

impl fmt::Debug for ColumnIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
pub struct ColumnNumber(RawIndex);

impl fmt::Debug for ColumnNumber {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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

/// A column offset in a source file
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
pub struct ColumnOffset(pub RawOffset);

impl Default for ColumnOffset {
    fn default() -> ColumnOffset {
        ColumnOffset(0)
    }
}

impl fmt::Debug for ColumnOffset {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ColumnOffset(")?;
        self.0.fmt(f)?;
        write!(f, ")")
    }
}

impl fmt::Display for ColumnOffset {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// A byte position in a source file.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
pub struct ByteIndex(pub RawIndex);

impl ByteIndex {
    /// Convert the position into a `usize`, for use in array indexing
    pub const fn to_usize(self) -> usize {
        self.0 as usize
    }
}

impl Default for ByteIndex {
    fn default() -> ByteIndex {
        ByteIndex(0)
    }
}

impl fmt::Debug for ByteIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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

/// A byte offset in a source file
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
pub struct ByteOffset(pub RawOffset);

impl ByteOffset {
    /// Create a byte offset from a UTF8-encoded character
    ///
    /// ```rust
    /// use codespan::ByteOffset;
    ///
    /// assert_eq!(ByteOffset::from_char_len('A').to_usize(), 1);
    /// assert_eq!(ByteOffset::from_char_len('ß').to_usize(), 2);
    /// assert_eq!(ByteOffset::from_char_len('ℝ').to_usize(), 3);
    /// assert_eq!(ByteOffset::from_char_len('💣').to_usize(), 4);
    /// ```
    pub fn from_char_len(ch: char) -> ByteOffset {
        ByteOffset(ch.len_utf8() as RawOffset)
    }

    /// Create a byte offset from a UTF- encoded string
    ///
    /// ```rust
    /// use codespan::ByteOffset;
    ///
    /// assert_eq!(ByteOffset::from_str_len("A").to_usize(), 1);
    /// assert_eq!(ByteOffset::from_str_len("ß").to_usize(), 2);
    /// assert_eq!(ByteOffset::from_str_len("ℝ").to_usize(), 3);
    /// assert_eq!(ByteOffset::from_str_len("💣").to_usize(), 4);
    /// ```
    pub fn from_str_len(value: &str) -> ByteOffset {
        ByteOffset(value.len() as RawOffset)
    }

    /// Convert the offset into a `usize`, for use in array indexing
    pub const fn to_usize(self) -> usize {
        self.0 as usize
    }
}

impl Default for ByteOffset {
    #[inline]
    fn default() -> ByteOffset {
        ByteOffset(0)
    }
}

impl fmt::Debug for ByteOffset {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ByteOffset(")?;
        self.0.fmt(f)?;
        write!(f, ")")
    }
}

impl fmt::Display for ByteOffset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// A relative offset between two indices
///
/// These can be thought of as 1-dimensional vectors
pub trait Offset: Copy + Ord
where
    Self: Neg<Output = Self>,
    Self: Add<Self, Output = Self>,
    Self: AddAssign<Self>,
    Self: Sub<Self, Output = Self>,
    Self: SubAssign<Self>,
{
    const ZERO: Self;
}

/// Index types
///
/// These can be thought of as 1-dimensional points
pub trait Index: Copy + Ord
where
    Self: Add<<Self as Index>::Offset, Output = Self>,
    Self: AddAssign<<Self as Index>::Offset>,
    Self: Sub<<Self as Index>::Offset, Output = Self>,
    Self: SubAssign<<Self as Index>::Offset>,
    Self: Sub<Self, Output = <Self as Index>::Offset>,
{
    type Offset: Offset;
}

macro_rules! impl_index {
    ($Index:ident, $Offset:ident) => {
        impl From<RawOffset> for $Offset {
            #[inline]
            fn from(i: RawOffset) -> Self {
                $Offset(i)
            }
        }

        impl From<RawIndex> for $Index {
            #[inline]
            fn from(i: RawIndex) -> Self {
                $Index(i)
            }
        }

        impl From<$Index> for RawIndex {
            #[inline]
            fn from(index: $Index) -> RawIndex {
                index.0
            }
        }

        impl From<$Offset> for RawOffset {
            #[inline]
            fn from(offset: $Offset) -> RawOffset {
                offset.0
            }
        }

        impl From<$Index> for usize {
            #[inline]
            fn from(index: $Index) -> usize {
                index.0 as usize
            }
        }

        impl From<$Offset> for usize {
            #[inline]
            fn from(offset: $Offset) -> usize {
                offset.0 as usize
            }
        }

        impl Offset for $Offset {
            const ZERO: $Offset = $Offset(0);
        }

        impl Index for $Index {
            type Offset = $Offset;
        }

        impl Add<$Offset> for $Index {
            type Output = $Index;

            #[inline]
            fn add(self, rhs: $Offset) -> $Index {
                $Index((self.0 as RawOffset + rhs.0) as RawIndex)
            }
        }

        impl AddAssign<$Offset> for $Index {
            #[inline]
            fn add_assign(&mut self, rhs: $Offset) {
                *self = *self + rhs;
            }
        }

        impl Neg for $Offset {
            type Output = $Offset;

            #[inline]
            fn neg(self) -> $Offset {
                $Offset(-self.0)
            }
        }

        impl Add<$Offset> for $Offset {
            type Output = $Offset;

            #[inline]
            fn add(self, rhs: $Offset) -> $Offset {
                $Offset(self.0 + rhs.0)
            }
        }

        impl AddAssign<$Offset> for $Offset {
            #[inline]
            fn add_assign(&mut self, rhs: $Offset) {
                self.0 += rhs.0;
            }
        }

        impl Sub<$Offset> for $Offset {
            type Output = $Offset;

            #[inline]
            fn sub(self, rhs: $Offset) -> $Offset {
                $Offset(self.0 - rhs.0)
            }
        }

        impl SubAssign<$Offset> for $Offset {
            #[inline]
            fn sub_assign(&mut self, rhs: $Offset) {
                self.0 -= rhs.0;
            }
        }

        impl Sub for $Index {
            type Output = $Offset;

            #[inline]
            fn sub(self, rhs: $Index) -> $Offset {
                $Offset(self.0 as RawOffset - rhs.0 as RawOffset)
            }
        }

        impl Sub<$Offset> for $Index {
            type Output = $Index;

            #[inline]
            fn sub(self, rhs: $Offset) -> $Index {
                $Index((self.0 as RawOffset - rhs.0 as RawOffset) as u32)
            }
        }

        impl SubAssign<$Offset> for $Index {
            #[inline]
            fn sub_assign(&mut self, rhs: $Offset) {
                self.0 = (self.0 as RawOffset - rhs.0) as RawIndex;
            }
        }
    };
}

impl_index!(LineIndex, LineOffset);
impl_index!(ColumnIndex, ColumnOffset);
impl_index!(ByteIndex, ByteOffset);
