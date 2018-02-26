use std::{cmp, fmt};

use index::{ByteIndex, Index};

/// A region of code in a source file
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub struct Span<I> {
    start: I,
    end: I,
}

impl<I: Index> Span<I> {
    /// Create a new span
    ///
    /// ```rust
    /// use codespan::{ByteIndex, Span};
    ///
    /// let span = Span::new(ByteIndex(3), ByteIndex(6));
    /// assert_eq!(span.start(), ByteIndex(3));
    /// assert_eq!(span.end(), ByteIndex(6));
    /// ```
    ///
    /// `start` and `end` are reordered to maintain the invariant that `start <= end`
    ///
    /// ```rust
    /// use codespan::{ByteIndex, Span};
    ///
    /// let span = Span::new(ByteIndex(6), ByteIndex(3));
    /// assert_eq!(span.start(), ByteIndex(3));
    /// assert_eq!(span.end(), ByteIndex(6));
    /// ```
    pub fn new(start: I, end: I) -> Span<I> {
        if start <= end {
            Span { start, end }
        } else {
            Span {
                start: end,
                end: start,
            }
        }
    }

    /// Makes a span from offsets relative to the start of this span.
    pub fn subspan(&self, begin: I::Offset, end: I::Offset) -> Span<I> {
        assert!(end >= begin);
        assert!(self.start() + end <= self.end());
        Span {
            start: self.start() + begin,
            end: self.start() + end,
        }
    }

    /// Create a new span from a byte start and an offset
    pub fn from_offset(start: I, off: I::Offset) -> Span<I> {
        Span::new(start, start + off)
    }

    /// Get the start index
    pub fn start(self) -> I {
        self.start
    }

    /// Get the end index
    pub fn end(self) -> I {
        self.end
    }

    /// Return a new span with the low byte position replaced with the supplied byte position
    ///
    /// ```rust
    /// use codespan::{ByteIndex, Span};
    ///
    /// let span = Span::new(ByteIndex(3), ByteIndex(6));
    /// assert_eq!(span.with_lo(ByteIndex(2)), Span::new(ByteIndex(2), ByteIndex(6)));
    /// assert_eq!(span.with_lo(ByteIndex(5)), Span::new(ByteIndex(5), ByteIndex(6)));
    /// assert_eq!(span.with_lo(ByteIndex(7)), Span::new(ByteIndex(6), ByteIndex(7)));
    /// ```
    pub fn with_lo(self, start: I) -> Span<I> {
        Span::new(start, self.end())
    }

    /// Return a new span with the high byte position replaced with the supplied byte position
    ///
    /// ```rust
    /// use codespan::{ByteIndex, Span};
    ///
    /// let span = Span::new(ByteIndex(3), ByteIndex(6));
    /// assert_eq!(span.with_hi(ByteIndex(7)), Span::new(ByteIndex(3), ByteIndex(7)));
    /// assert_eq!(span.with_hi(ByteIndex(5)), Span::new(ByteIndex(3), ByteIndex(5)));
    /// assert_eq!(span.with_hi(ByteIndex(2)), Span::new(ByteIndex(2), ByteIndex(3)));
    /// ```
    pub fn with_hi(self, end: I) -> Span<I> {
        Span::new(self.start(), end)
    }

    /// Return true if `self` fully encloses `other`.
    ///
    /// ```rust
    /// use codespan::{ByteIndex, Span};
    ///
    /// let a = Span::new(ByteIndex(5), ByteIndex(8));
    ///
    /// assert_eq!(a.contains(a), true);
    /// assert_eq!(a.contains(Span::new(ByteIndex(6), ByteIndex(7))), true);
    /// assert_eq!(a.contains(Span::new(ByteIndex(6), ByteIndex(10))), false);
    /// assert_eq!(a.contains(Span::new(ByteIndex(3), ByteIndex(6))), false);
    /// ```
    pub fn contains(self, other: Span<I>) -> bool {
        self.start() <= other.start() && other.end() <= self.end()
    }

    /// Return a `Span` that would enclose both `self` and `end`.
    ///
    /// ```plain
    /// self     ~~~~~~~
    /// end                     ~~~~~~~~
    /// returns  ~~~~~~~~~~~~~~~~~~~~~~~
    /// ```
    ///
    /// ```rust
    /// use codespan::{ByteIndex, Span};
    ///
    /// let a = Span::new(ByteIndex(2), ByteIndex(5));
    /// let b = Span::new(ByteIndex(10), ByteIndex(14));
    ///
    /// assert_eq!(a.to(b), Span::new(ByteIndex(2), ByteIndex(14)));
    /// ```
    pub fn to(self, end: Span<I>) -> Span<I> {
        Span::new(
            cmp::min(self.start(), end.start()),
            cmp::max(self.end(), end.end()),
        )
    }

    /// Return a `Span` between the end of `self` to the beginning of `end`.
    ///
    /// ```plain
    /// self     ~~~~~~~
    /// end                     ~~~~~~~~
    /// returns         ~~~~~~~~~
    /// ```
    ///
    /// ```rust
    /// use codespan::{ByteIndex, Span};
    ///
    /// let a = Span::new(ByteIndex(2), ByteIndex(5));
    /// let b = Span::new(ByteIndex(10), ByteIndex(14));
    ///
    /// assert_eq!(a.between(b), Span::new(ByteIndex(5), ByteIndex(10)));
    /// ```
    pub fn between(self, end: Span<I>) -> Span<I> {
        Span::new(self.end(), end.start())
    }

    /// Return a `Span` between the beginning of `self` to the beginning of `end`.
    ///
    /// ```plain
    /// self     ~~~~~~~
    /// end                     ~~~~~~~~
    /// returns  ~~~~~~~~~~~~~~~~
    /// ```
    ///
    /// ```rust
    /// use codespan::{ByteIndex, Span};
    ///
    /// let a = Span::new(ByteIndex(2), ByteIndex(5));
    /// let b = Span::new(ByteIndex(10), ByteIndex(14));
    ///
    /// assert_eq!(a.until(b), Span::new(ByteIndex(2), ByteIndex(10)));
    /// ```
    pub fn until(self, end: Span<I>) -> Span<I> {
        Span::new(self.start(), end.start())
    }
}

impl<I: fmt::Display> fmt::Display for Span<I> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.start.fmt(f)?;
        write!(f, "..")?;
        self.end.fmt(f)?;
        Ok(())
    }
}

pub type ByteSpan = Span<ByteIndex>;
