//! Wrapper types that specify positions in a source file

use std::{cmp, fmt};
use std::ops::{Add, AddAssign, Neg, Sub};

/// The raw, untyped index. We use a 32-bit integer here for space efficiency,
/// assuming we won't be working with sources larger than 4GB.
pub type RawIndex = u32;

/// The raw, untyped offset.
pub type RawOffset = i64;

/// A zero-indexed line offest into a source file
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
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
    pub fn number(self) -> LineNumber {
        LineNumber(self.0 + 1)
    }

    /// Convert the index into a `usize`, for use in array indexing
    pub fn to_usize(self) -> usize {
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

/// A 1-indexed line number. Useful for pretty printing source locations.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LineNumber(RawIndex);

impl fmt::Debug for LineNumber {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "LineNumber(")?;
        self.0.fmt(f)?;
        write!(f, ")")
    }
}

impl fmt::Display for LineNumber {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// A zero-indexed column offest into a source file
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
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
    pub fn number(self) -> ColumnNumber {
        ColumnNumber(self.0 + 1)
    }

    /// Convert the index into a `usize`, for use in array indexing
    pub fn to_usize(self) -> usize {
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

/// A 1-indexed column number. Useful for pretty printing source locations.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ColumnNumber(RawIndex);

impl fmt::Debug for ColumnNumber {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ColumnNumber(")?;
        self.0.fmt(f)?;
        write!(f, ")")
    }
}

impl fmt::Display for ColumnNumber {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// A byte position in a source file
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ByteIndex(pub RawIndex);

impl ByteIndex {
    /// A byte position that will never point to a valid file
    pub fn none() -> ByteIndex {
        ByteIndex(0)
    }

    /// Convert the position into a `usize`, for use in array indexing
    pub fn to_usize(self) -> usize {
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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// A byte offset in a source file
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ByteOffset(pub RawOffset);

impl ByteOffset {
    /// Create a byte offset from a UTF8-encoded character
    ///
    /// ```rust
    /// use codespan::ByteOffset;
    ///
    /// assert_eq!(ByteOffset::from_char_utf8('A').to_usize(), 1);
    /// assert_eq!(ByteOffset::from_char_utf8('ß').to_usize(), 2);
    /// assert_eq!(ByteOffset::from_char_utf8('ℝ').to_usize(), 3);
    /// assert_eq!(ByteOffset::from_char_utf8('💣').to_usize(), 4);
    /// ```
    pub fn from_char_utf8(ch: char) -> ByteOffset {
        ByteOffset(ch.len_utf8() as RawOffset)
    }

    /// Create a byte offset from a UTF- encoded string
    ///
    /// ```rust
    /// use codespan::ByteOffset;
    ///
    /// assert_eq!(ByteOffset::from_str("A").to_usize(), 1);
    /// assert_eq!(ByteOffset::from_str("ß").to_usize(), 2);
    /// assert_eq!(ByteOffset::from_str("ℝ").to_usize(), 3);
    /// assert_eq!(ByteOffset::from_str("💣").to_usize(), 4);
    /// ```
    pub fn from_str(value: &str) -> ByteOffset {
        ByteOffset(value.len() as RawOffset)
    }

    /// Convert the offset into a `usize`, for use in array indexing
    pub fn to_usize(self) -> usize {
        self.0 as usize
    }
}

impl Default for ByteOffset {
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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Add<ByteOffset> for ByteIndex {
    type Output = ByteIndex;

    fn add(self, rhs: ByteOffset) -> ByteIndex {
        ByteIndex((self.0 as RawOffset + rhs.0) as RawIndex)
    }
}

impl AddAssign<ByteOffset> for ByteIndex {
    fn add_assign(&mut self, rhs: ByteOffset) {
        *self = *self + rhs;
    }
}

impl Neg for ByteOffset {
    type Output = ByteOffset;

    fn neg(self) -> ByteOffset {
        ByteOffset(-self.0)
    }
}

impl Add<ByteOffset> for ByteOffset {
    type Output = ByteOffset;

    fn add(self, rhs: ByteOffset) -> ByteOffset {
        ByteOffset(self.0 + rhs.0)
    }
}

impl AddAssign<ByteOffset> for ByteOffset {
    fn add_assign(&mut self, rhs: ByteOffset) {
        self.0 += rhs.0;
    }
}

impl Sub for ByteIndex {
    type Output = ByteOffset;

    fn sub(self, rhs: ByteIndex) -> ByteOffset {
        ByteOffset(self.0 as RawOffset - rhs.0 as RawOffset)
    }
}

/// A region of code in a source file
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub struct ByteSpan {
    start: ByteIndex,
    end: ByteIndex,
}

impl ByteSpan {
    /// Create a new span
    ///
    /// ```rust
    /// use codespan::{ByteIndex, ByteSpan};
    ///
    /// let span = ByteSpan::new(ByteIndex(3), ByteIndex(6));
    /// assert_eq!(span.start(), ByteIndex(3));
    /// assert_eq!(span.end(), ByteIndex(6));
    /// ```
    ///
    /// `start` and `end` are reordered to maintain the invariant that `start <= end`
    ///
    /// ```rust
    /// use codespan::{ByteIndex, ByteSpan};
    ///
    /// let span = ByteSpan::new(ByteIndex(6), ByteIndex(3));
    /// assert_eq!(span.start(), ByteIndex(3));
    /// assert_eq!(span.end(), ByteIndex(6));
    /// ```
    pub fn new(start: ByteIndex, end: ByteIndex) -> ByteSpan {
        if start <= end {
            ByteSpan { start, end }
        } else {
            ByteSpan {
                start: end,
                end: start,
            }
        }
    }

    /// Create a new span from a byte start and an offset
    pub fn from_offset(start: ByteIndex, off: ByteOffset) -> ByteSpan {
        ByteSpan::new(start, start + off)
    }

    /// A span that will never point to a valid byte range
    pub fn none() -> ByteSpan {
        ByteSpan {
            start: ByteIndex::none(),
            end: ByteIndex::none(),
        }
    }

    /// Makes a span from offsets relative to the start of this span.
    pub fn subspan(&self, begin: ByteOffset, end: ByteOffset) -> ByteSpan {
        assert!(end >= begin);
        assert!(self.start() + end <= self.end());
        ByteSpan {
            start: self.start() + begin,
            end: self.start() + end,
        }
    }

    /// Get the start index
    pub fn start(self) -> ByteIndex {
        self.start
    }

    /// Get the end index
    pub fn end(self) -> ByteIndex {
        self.end
    }

    /// Return a new span with the low byte position replaced with the supplied byte position
    ///
    /// ```rust
    /// use codespan::{ByteIndex, ByteSpan};
    ///
    /// let span = ByteSpan::new(ByteIndex(3), ByteIndex(6));
    /// assert_eq!(span.with_lo(ByteIndex(2)), ByteSpan::new(ByteIndex(2), ByteIndex(6)));
    /// assert_eq!(span.with_lo(ByteIndex(5)), ByteSpan::new(ByteIndex(5), ByteIndex(6)));
    /// assert_eq!(span.with_lo(ByteIndex(7)), ByteSpan::new(ByteIndex(6), ByteIndex(7)));
    /// ```
    pub fn with_lo(self, start: ByteIndex) -> ByteSpan {
        ByteSpan::new(start, self.end())
    }

    /// Return a new span with the high byte position replaced with the supplied byte position
    ///
    /// ```rust
    /// use codespan::{ByteIndex, ByteSpan};
    ///
    /// let span = ByteSpan::new(ByteIndex(3), ByteIndex(6));
    /// assert_eq!(span.with_hi(ByteIndex(7)), ByteSpan::new(ByteIndex(3), ByteIndex(7)));
    /// assert_eq!(span.with_hi(ByteIndex(5)), ByteSpan::new(ByteIndex(3), ByteIndex(5)));
    /// assert_eq!(span.with_hi(ByteIndex(2)), ByteSpan::new(ByteIndex(2), ByteIndex(3)));
    /// ```
    pub fn with_hi(self, end: ByteIndex) -> ByteSpan {
        ByteSpan::new(self.start(), end)
    }

    /// Return true if `self` fully encloses `other`.
    ///
    /// ```rust
    /// use codespan::{ByteIndex, ByteSpan};
    ///
    /// let a = ByteSpan::new(ByteIndex(5), ByteIndex(8));
    ///
    /// assert_eq!(a.contains(a), true);
    /// assert_eq!(a.contains(ByteSpan::new(ByteIndex(6), ByteIndex(7))), true);
    /// assert_eq!(a.contains(ByteSpan::new(ByteIndex(6), ByteIndex(10))), false);
    /// assert_eq!(a.contains(ByteSpan::new(ByteIndex(3), ByteIndex(6))), false);
    /// ```
    pub fn contains(self, other: ByteSpan) -> bool {
        self.start() <= other.start() && other.end() <= self.end()
    }

    /// Return a `ByteSpan` that would enclose both `self` and `end`.
    ///
    /// ```plain
    /// self     ~~~~~~~
    /// end                     ~~~~~~~~
    /// returns  ~~~~~~~~~~~~~~~~~~~~~~~
    /// ```
    ///
    /// ```rust
    /// use codespan::{ByteIndex, ByteSpan};
    ///
    /// let a = ByteSpan::new(ByteIndex(2), ByteIndex(5));
    /// let b = ByteSpan::new(ByteIndex(10), ByteIndex(14));
    ///
    /// assert_eq!(a.to(b), ByteSpan::new(ByteIndex(2), ByteIndex(14)));
    /// ```
    pub fn to(self, end: ByteSpan) -> ByteSpan {
        ByteSpan::new(
            cmp::min(self.start(), end.start()),
            cmp::max(self.end(), end.end()),
        )
    }

    /// Return a `ByteSpan` between the end of `self` to the beginning of `end`.
    ///
    /// ```plain
    /// self     ~~~~~~~
    /// end                     ~~~~~~~~
    /// returns         ~~~~~~~~~
    /// ```
    ///
    /// ```rust
    /// use codespan::{ByteIndex, ByteSpan};
    ///
    /// let a = ByteSpan::new(ByteIndex(2), ByteIndex(5));
    /// let b = ByteSpan::new(ByteIndex(10), ByteIndex(14));
    ///
    /// assert_eq!(a.between(b), ByteSpan::new(ByteIndex(5), ByteIndex(10)));
    /// ```
    pub fn between(self, end: ByteSpan) -> ByteSpan {
        ByteSpan::new(self.end(), end.start())
    }

    /// Return a `ByteSpan` between the beginning of `self` to the beginning of `end`.
    ///
    /// ```plain
    /// self     ~~~~~~~
    /// end                     ~~~~~~~~
    /// returns  ~~~~~~~~~~~~~~~~
    /// ```
    ///
    /// ```rust
    /// use codespan::{ByteIndex, ByteSpan};
    ///
    /// let a = ByteSpan::new(ByteIndex(2), ByteIndex(5));
    /// let b = ByteSpan::new(ByteIndex(10), ByteIndex(14));
    ///
    /// assert_eq!(a.until(b), ByteSpan::new(ByteIndex(2), ByteIndex(10)));
    /// ```
    pub fn until(self, end: ByteSpan) -> ByteSpan {
        ByteSpan::new(self.start(), end.start())
    }
}

impl fmt::Display for ByteSpan {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.start.fmt(f)?;
        write!(f, "..")?;
        self.end.fmt(f)?;
        Ok(())
    }
}
