use std::{cmp, fmt};

use index::{ByteIndex, ByteOffset};

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
