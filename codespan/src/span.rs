#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::{ByteIndex, ByteSize};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "memory_usage", derive(heapsize_derive::HeapSizeOf))]
pub struct Span {
    start: ByteIndex,
    end: ByteIndex,
}

impl Span {
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

    pub fn from_str(s: &str) -> Span {
        Span::new(0, s.len() as u32)
    }

    pub fn merge(self, other: Span) -> Span {
        Span::new(self.start(), other.end())
    }

    pub fn with_start(&self, start: impl Into<ByteIndex>) -> Span {
        Span::new(start, self.end())
    }

    pub fn with_end(&self, end: impl Into<ByteIndex>) -> Span {
        Span::new(self.start(), end)
    }

    pub fn start_span(&self) -> Span {
        self.with_end(self.start())
    }

    pub fn end_span(&self) -> Span {
        self.with_start(self.end())
    }

    pub fn start(&self) -> ByteIndex {
        self.start
    }

    pub fn end(&self) -> ByteIndex {
        self.end
    }

    pub fn contains(self, span: Span) -> bool {
        self.start() >= span.start() && self.end() < span.end()
    }

    pub fn contains_index(self, index: impl Into<ByteIndex>) -> bool {
        let index = index.into();
        self.start() <= index && index < self.end()
    }

    pub fn len(&self) -> ByteSize {
        self.end() - self.start()
    }
}

impl fmt::Debug for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{start}, {end})",
            start = self.start(),
            end = self.end(),
        )
    }
}
