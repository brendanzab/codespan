//! Wrapper types that specify positions in a source file

use std::{cmp, fmt};
use std::ops::{Add, AddAssign, Neg, Sub};

/// The raw, untyped position. We use a 32-bit integer here for space efficiency,
/// assuming we won't be working with sources larger than 4GB.
pub type RawPos = u32;

/// The raw, untyped offset.
pub type RawOffset = i64;

/// A zero-indexed line offest into a source file
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LineIndex(pub RawPos);

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
pub struct LineNumber(RawPos);

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
pub struct ColumnIndex(pub RawPos);

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
pub struct ColumnNumber(RawPos);

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
pub struct BytePos(pub RawPos);

impl BytePos {
    /// A byte position that will never point to a valid file
    pub fn none() -> BytePos {
        BytePos(0)
    }

    /// Convert the position into a `usize`, for use in array indexing
    pub fn to_usize(self) -> usize {
        self.0 as usize
    }
}

impl Default for BytePos {
    fn default() -> BytePos {
        BytePos(0)
    }
}

impl fmt::Debug for BytePos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BytePos(")?;
        self.0.fmt(f)?;
        write!(f, ")")
    }
}

impl fmt::Display for BytePos {
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

impl Add<ByteOffset> for BytePos {
    type Output = BytePos;

    fn add(self, rhs: ByteOffset) -> BytePos {
        BytePos((self.0 as RawOffset + rhs.0) as RawPos)
    }
}

impl AddAssign<ByteOffset> for BytePos {
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

impl Sub for BytePos {
    type Output = ByteOffset;

    fn sub(self, rhs: BytePos) -> ByteOffset {
        ByteOffset(self.0 as RawOffset - rhs.0 as RawOffset)
    }
}

/// A region of code in a source file
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub struct ByteSpan {
    lo: BytePos,
    hi: BytePos,
}

impl ByteSpan {
    /// Create a new span
    ///
    /// ```rust
    /// use codespan::{BytePos, ByteSpan};
    ///
    /// let span = ByteSpan::new(BytePos(3), BytePos(6));
    /// assert_eq!(span.lo(), BytePos(3));
    /// assert_eq!(span.hi(), BytePos(6));
    /// ```
    ///
    /// `lo` are reordered `hi` to maintain the invariant that `lo <= hi`
    ///
    /// ```rust
    /// use codespan::{BytePos, ByteSpan};
    ///
    /// let span = ByteSpan::new(BytePos(6), BytePos(3));
    /// assert_eq!(span.lo(), BytePos(3));
    /// assert_eq!(span.hi(), BytePos(6));
    /// ```
    pub fn new(lo: BytePos, hi: BytePos) -> ByteSpan {
        if lo <= hi {
            ByteSpan { lo, hi }
        } else {
            ByteSpan { lo: hi, hi: lo }
        }
    }

    /// Create a new span from a byte start and an offset
    pub fn from_offset(lo: BytePos, off: ByteOffset) -> ByteSpan {
        ByteSpan::new(lo, lo + off)
    }

    /// A span that will never point to a valid byte range
    pub fn none() -> ByteSpan {
        ByteSpan {
            lo: BytePos::none(),
            hi: BytePos::none(),
        }
    }

    /// Makes a span from offsets relative to the start of this span.
    pub fn subspan(&self, begin: ByteOffset, end: ByteOffset) -> ByteSpan {
        assert!(end >= begin);
        assert!(self.lo() + end <= self.hi());
        ByteSpan {
            lo: self.lo() + begin,
            hi: self.lo() + end,
        }
    }

    /// Get the low byte position
    pub fn lo(self) -> BytePos {
        self.lo
    }

    /// Get the high byte position
    pub fn hi(self) -> BytePos {
        self.hi
    }

    /// Return a new span with the low byte position replaced with the supplied byte position
    ///
    /// ```rust
    /// use codespan::{BytePos, ByteSpan};
    ///
    /// let span = ByteSpan::new(BytePos(3), BytePos(6));
    /// assert_eq!(span.with_lo(BytePos(2)), ByteSpan::new(BytePos(2), BytePos(6)));
    /// assert_eq!(span.with_lo(BytePos(5)), ByteSpan::new(BytePos(5), BytePos(6)));
    /// assert_eq!(span.with_lo(BytePos(7)), ByteSpan::new(BytePos(6), BytePos(7)));
    /// ```
    pub fn with_lo(self, lo: BytePos) -> ByteSpan {
        ByteSpan::new(lo, self.hi())
    }

    /// Return a new span with the high byte position replaced with the supplied byte position
    ///
    /// ```rust
    /// use codespan::{BytePos, ByteSpan};
    ///
    /// let span = ByteSpan::new(BytePos(3), BytePos(6));
    /// assert_eq!(span.with_hi(BytePos(7)), ByteSpan::new(BytePos(3), BytePos(7)));
    /// assert_eq!(span.with_hi(BytePos(5)), ByteSpan::new(BytePos(3), BytePos(5)));
    /// assert_eq!(span.with_hi(BytePos(2)), ByteSpan::new(BytePos(2), BytePos(3)));
    /// ```
    pub fn with_hi(self, hi: BytePos) -> ByteSpan {
        ByteSpan::new(self.lo(), hi)
    }

    /// Return true if `self` fully encloses `other`.
    ///
    /// ```rust
    /// use codespan::{BytePos, ByteSpan};
    ///
    /// let a = ByteSpan::new(BytePos(5), BytePos(8));
    ///
    /// assert_eq!(a.contains(a), true);
    /// assert_eq!(a.contains(ByteSpan::new(BytePos(6), BytePos(7))), true);
    /// assert_eq!(a.contains(ByteSpan::new(BytePos(6), BytePos(10))), false);
    /// assert_eq!(a.contains(ByteSpan::new(BytePos(3), BytePos(6))), false);
    /// ```
    pub fn contains(self, other: ByteSpan) -> bool {
        self.lo() <= other.lo() && other.hi() <= self.hi()
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
    /// use codespan::{BytePos, ByteSpan};
    ///
    /// let a = ByteSpan::new(BytePos(2), BytePos(5));
    /// let b = ByteSpan::new(BytePos(10), BytePos(14));
    ///
    /// assert_eq!(a.to(b), ByteSpan::new(BytePos(2), BytePos(14)));
    /// ```
    pub fn to(self, end: ByteSpan) -> ByteSpan {
        ByteSpan::new(cmp::min(self.lo(), end.lo()), cmp::max(self.hi(), end.hi()))
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
    /// use codespan::{BytePos, ByteSpan};
    ///
    /// let a = ByteSpan::new(BytePos(2), BytePos(5));
    /// let b = ByteSpan::new(BytePos(10), BytePos(14));
    ///
    /// assert_eq!(a.between(b), ByteSpan::new(BytePos(5), BytePos(10)));
    /// ```
    pub fn between(self, end: ByteSpan) -> ByteSpan {
        ByteSpan::new(self.hi(), end.lo())
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
    /// use codespan::{BytePos, ByteSpan};
    ///
    /// let a = ByteSpan::new(BytePos(2), BytePos(5));
    /// let b = ByteSpan::new(BytePos(10), BytePos(14));
    ///
    /// assert_eq!(a.until(b), ByteSpan::new(BytePos(2), BytePos(10)));
    /// ```
    pub fn until(self, end: ByteSpan) -> ByteSpan {
        ByteSpan::new(self.lo(), end.lo())
    }
}

impl fmt::Display for ByteSpan {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.lo.fmt(f)?;
        write!(f, "..")?;
        write!(f, "..")?;
        write!(f, "..")?;
        self.hi.fmt(f)?;
        Ok(())
    }
}
