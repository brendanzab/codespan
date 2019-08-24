#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::Range;

use crate::ByteIndex;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "memory_usage", derive(heapsize_derive::HeapSizeOf))]
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
    pub fn initial() -> Span {
        Span::new(0, 0)
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

    /// Combine two spans by taking the start of the first span and the end of
    /// the other span.
    ///
    /// ```rust
    /// use codespan::{ByteIndex, Span};
    ///
    /// let span1 = Span::new(0, 4);
    /// let span2 = Span::new(10, 16);
    ///
    /// assert_eq!(Span::merge(span1, span2), Span::new(0, 16));
    /// ```
    pub fn merge(self, other: Span) -> Span {
        Span::new(self.start(), other.end())
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
    pub fn start(&self) -> ByteIndex {
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
    pub fn end(&self) -> ByteIndex {
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
