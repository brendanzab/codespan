#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::Range;

use crate::{ByteIndex, RawIndex};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
pub struct Span {
    start: ByteIndex,
    end: ByteIndex,
}

impl Span {
    /// Create a new span from a starting and ending span.
    pub fn new(start: impl Into<ByteIndex>, end: impl Into<ByteIndex>) -> Span {
        let start = start.into();
        let end = end.into();

        assert!(end >= start);

        Span { start, end }
    }

    /// Gives an empty span at the start of a source.
    pub const fn initial() -> Span {
        Span {
            start: ByteIndex(0),
            end: ByteIndex(0),
        }
    }

    /// Measure the span of a string.
    ///
    /// ```rust
    /// use codespan::{ByteIndex, Span};
    ///
    /// let span = Span::from_str("hello");
    ///
    /// assert_eq!(span, Span::new(0, 5));
    /// ```
    pub fn from_str(s: &str) -> Span {
        Span::new(0, s.len() as u32)
    }

    /// Combine two spans by taking the start of the earlier span
    /// and the end of the later span.
    ///
    /// Note: this will work even if the two spans are disjoint.
    /// If this doesn't make sense in your application, you should handle it yourself.
    /// In that case, you can use `Span::disjoint` as a convenience function.
    ///
    /// ```rust
    /// use codespan::Span;
    ///
    /// let span1 = Span::new(0, 4);
    /// let span2 = Span::new(10, 16);
    ///
    /// assert_eq!(Span::merge(span1, span2), Span::new(0, 16));
    /// ```
    pub fn merge(self, other: Span) -> Span {
        use std::cmp::{max, min};

        let start = min(self.start, other.start);
        let end = max(self.end, other.end);
        Span::new(start, end)
    }

    /// A helper function to tell whether two spans do not overlap.
    ///
    /// ```
    /// use codespan::Span;
    /// let span1 = Span::new(0, 4);
    /// let span2 = Span::new(10, 16);
    /// assert!(span1.disjoint(span2));
    /// ```
    pub fn disjoint(self, other: Span) -> bool {
        let (first, last) = if self.end < other.end {
            (self, other)
        } else {
            (other, self)
        };
        first.end <= last.start
    }

    /// Get the starting byte index.
    ///
    /// ```rust
    /// use codespan::{ByteIndex, Span};
    ///
    /// let span = Span::new(0, 4);
    ///
    /// assert_eq!(span.start(), ByteIndex::from(0));
    /// ```
    pub fn start(self) -> ByteIndex {
        self.start
    }

    /// Get the ending byte index.
    ///
    /// ```rust
    /// use codespan::{ByteIndex, Span};
    ///
    /// let span = Span::new(0, 4);
    ///
    /// assert_eq!(span.end(), ByteIndex::from(4));
    /// ```
    pub fn end(self) -> ByteIndex {
        self.end
    }
}

impl Default for Span {
    fn default() -> Span {
        Span::initial()
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{start}, {end})",
            start = self.start(),
            end = self.end(),
        )
    }
}

impl<I> From<Range<I>> for Span
where
    I: Into<ByteIndex>,
{
    fn from(range: Range<I>) -> Span {
        Span::new(range.start, range.end)
    }
}

impl From<Span> for Range<usize> {
    fn from(span: Span) -> Range<usize> {
        span.start.into()..span.end.into()
    }
}

impl From<Span> for Range<RawIndex> {
    fn from(span: Span) -> Range<RawIndex> {
        span.start.0..span.end.0
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_merge() {
        use super::Span;

        // overlap
        let a = Span::from(1..5);
        let b = Span::from(3..10);
        assert_eq!(a.merge(b), Span::from(1..10));
        assert_eq!(b.merge(a), Span::from(1..10));

        // subset
        let two_four = (2..4).into();
        assert_eq!(a.merge(two_four), (1..5).into());
        assert_eq!(two_four.merge(a), (1..5).into());

        // disjoint
        let ten_twenty = (10..20).into();
        assert_eq!(a.merge(ten_twenty), (1..20).into());
        assert_eq!(ten_twenty.merge(a), (1..20).into());

        // identity
        assert_eq!(a.merge(a), a);
    }

    #[test]
    fn test_disjoint() {
        use super::Span;

        // overlap
        let a = Span::from(1..5);
        let b = Span::from(3..10);
        assert!(!a.disjoint(b));
        assert!(!b.disjoint(a));

        // subset
        let two_four = (2..4).into();
        assert!(!a.disjoint(two_four));
        assert!(!two_four.disjoint(a));

        // disjoint
        let ten_twenty = (10..20).into();
        assert!(a.disjoint(ten_twenty));
        assert!(ten_twenty.disjoint(a));

        // identity
        assert!(!a.disjoint(a));

        // off by one (upper bound)
        let c = Span::from(5..10);
        assert!(a.disjoint(c));
        assert!(c.disjoint(a));
        // off by one (lower bound)
        let d = Span::from(0..1);
        assert!(a.disjoint(d));
        assert!(d.disjoint(a));
    }
}
