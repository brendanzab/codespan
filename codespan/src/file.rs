#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};
use std::{error, fmt};

use crate::{ByteIndex, ColumnIndex, LineIndex, LineOffset, Location, RawIndex, Span};

#[derive(Debug, PartialEq)]
pub struct LineIndexOutOfBoundsError {
    pub given: LineIndex,
    pub max: LineIndex,
}

impl error::Error for LineIndexOutOfBoundsError {}

impl fmt::Display for LineIndexOutOfBoundsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Line index out of bounds - given: {}, max: {}",
            self.given, self.max
        )
    }
}

#[derive(Debug, PartialEq)]
pub enum LocationError {
    OutOfBounds { given: ByteIndex, span: Span },
    InvalidCharBoundary { given: ByteIndex },
}

impl error::Error for LocationError {}

impl fmt::Display for LocationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LocationError::OutOfBounds { given, span } => write!(
                f,
                "Byte index out of bounds - given: {}, span: {}",
                given, span
            ),
            LocationError::InvalidCharBoundary { given } => {
                write!(f, "Byte index within character boundary - given: {}", given)
            },
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct SpanOutOfBoundsError {
    pub given: Span,
    pub span: Span,
}

impl error::Error for SpanOutOfBoundsError {}

impl fmt::Display for SpanOutOfBoundsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Span out of bounds - given: {}, span: {}",
            self.given, self.span
        )
    }
}

/// A handle that points to a file in the database.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "memory_usage", derive(heapsize_derive::HeapSizeOf))]
pub struct FileId(u32);

/// A database of source files.
#[derive(Debug, Clone)]
pub struct Files {
    files: Vec<File>,
}

impl Files {
    /// Create a new, empty database of files.
    pub fn new() -> Files {
        Files { files: Vec::new() }
    }

    /// Add a file to the database, returning the handle that can be used to
    /// refer to it again.
    pub fn add(&mut self, name: impl Into<String>, source: impl Into<String>) -> FileId {
        let file_id = FileId(self.files.len() as u32);
        self.files.push(File::new(name.into(), source.into()));
        file_id
    }

    /// Update a source file in place.
    ///
    /// This will mean that any outstanding byte indexes will now point to
    /// invalid locations.
    pub fn update(&mut self, file_id: FileId, source: impl Into<String>) {
        self.get_mut(file_id).update(source.into())
    }

    /// Get a the source file using the file id.
    // FIXME: return an option or result?
    fn get(&self, file_id: FileId) -> &File {
        &self.files[file_id.0 as usize]
    }

    /// Get a the source file using the file id.
    // FIXME: return an option or result?
    fn get_mut(&mut self, file_id: FileId) -> &mut File {
        &mut self.files[file_id.0 as usize]
    }

    /// Get the name of the source file.
    ///
    /// ```rust
    /// use codespan::Files;
    ///
    /// let name = "test";
    ///
    /// let mut files = Files::new();
    /// let file_id = files.add(name, "hello world!");
    ///
    /// assert_eq!(files.name(file_id), name);
    /// ```
    pub fn name(&self, file_id: FileId) -> &str {
        self.get(file_id).name()
    }

    /// Get the span at the given line index.
    ///
    /// ```rust
    /// use codespan::{Files, LineIndex, LineIndexOutOfBoundsError, Span};
    ///
    /// let mut files = Files::new();
    /// let file_id = files.add("test", "foo\nbar\r\n\nbaz");
    ///
    /// let line_sources = (0..5)
    ///     .map(|line| files.line_span(file_id, line))
    ///     .collect::<Vec<_>>();
    ///
    /// assert_eq!(
    ///     line_sources,
    ///     [
    ///         Ok(Span::new(0, 4)),    // 0: "foo\n"
    ///         Ok(Span::new(4, 9)),    // 1: "bar\r\n"
    ///         Ok(Span::new(9, 10)),   // 2: ""
    ///         Ok(Span::new(10, 13)),  // 3: "baz"
    ///         Err(LineIndexOutOfBoundsError {
    ///             given: LineIndex::from(5),
    ///             max: LineIndex::from(4),
    ///         }),
    ///     ]
    /// );
    /// ```
    pub fn line_span(
        &self,
        file_id: FileId,
        line_index: impl Into<LineIndex>,
    ) -> Result<Span, LineIndexOutOfBoundsError> {
        self.get(file_id).line_span(line_index.into())
    }

    /// Get the location at the given byte index in the source file.
    ///
    /// ```rust
    /// use codespan::{ByteIndex, Files, Location, LocationError, Span};
    ///
    /// let mut files = Files::new();
    /// let file_id = files.add("test", "foo\nbar\r\n\nbaz");
    ///
    /// assert_eq!(files.location(file_id, 0), Ok(Location::new(0, 0)));
    /// assert_eq!(files.location(file_id, 7), Ok(Location::new(1, 3)));
    /// assert_eq!(files.location(file_id, 8), Ok(Location::new(1, 4)));
    /// assert_eq!(files.location(file_id, 9), Ok(Location::new(2, 0)));
    /// assert_eq!(
    ///     files.location(file_id, 100),
    ///     Err(LocationError::OutOfBounds {
    ///         given: ByteIndex::from(100),
    ///         span: Span::new(0, 13),
    ///     }),
    /// );
    /// ```
    pub fn location(
        &self,
        file_id: FileId,
        byte_index: impl Into<ByteIndex>,
    ) -> Result<Location, LocationError> {
        self.get(file_id).location(byte_index.into())
    }

    /// Get the source of the file.
    ///
    /// ```rust
    /// use codespan::Files;
    ///
    /// let source = "hello world!";
    ///
    /// let mut files = Files::new();
    /// let file_id = files.add("test", source);
    ///
    /// assert_eq!(files.source(file_id), source);
    /// ```
    pub fn source(&self, file_id: FileId) -> &str {
        self.get(file_id).source()
    }

    /// Return the span of the full source.
    ///
    /// ```rust
    /// use codespan::{Files, Span};
    ///
    /// let source = "hello world!";
    ///
    /// let mut files = Files::new();
    /// let file_id = files.add("test", source);
    ///
    /// assert_eq!(files.source_span(file_id), Span::from_str(source));
    /// ```
    pub fn source_span(&self, file_id: FileId) -> Span {
        self.get(file_id).source_span()
    }

    /// Return a slice of the source file, given a span.
    ///
    /// ```rust
    /// use codespan::{Files, Span};
    ///
    /// let mut files = Files::new();
    /// let file_id = files.add("test",  "hello world!");
    ///
    /// assert_eq!(files.source_slice(file_id, Span::new(0, 5)), Ok("hello"));
    /// assert!(files.source_slice(file_id, Span::new(0, 100)).is_err());
    /// ```
    pub fn source_slice(
        &self,
        file_id: FileId,
        span: impl Into<Span>,
    ) -> Result<&str, SpanOutOfBoundsError> {
        self.get(file_id).source_slice(span.into())
    }
}

/// A file that is stored in the database.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "memory_usage", derive(heapsize_derive::HeapSizeOf))]
struct File {
    /// The name of the file.
    name: String,
    /// The source code of the file.
    source: String,
    /// The starting byte indices in the source code.
    line_starts: Vec<ByteIndex>,
}

// FIXME: Check file size
fn compute_line_starts(source: &str) -> Vec<ByteIndex> {
    std::iter::once(0)
        .chain(source.match_indices('\n').map(|(i, _)| i as u32 + 1))
        .map(ByteIndex::from)
        .collect()
}

impl File {
    fn new(name: String, source: String) -> File {
        let line_starts = compute_line_starts(&source);

        File {
            name,
            source,
            line_starts,
        }
    }

    fn update(&mut self, source: String) {
        let line_starts = compute_line_starts(&source);
        self.source = source;
        self.line_starts = line_starts;
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn line_start(&self, line_index: LineIndex) -> Result<ByteIndex, LineIndexOutOfBoundsError> {
        use std::cmp::Ordering;

        match line_index.cmp(&self.last_line_index()) {
            Ordering::Less => Ok(self.line_starts[line_index.to_usize()]),
            Ordering::Equal => Ok(self.source_span().end()),
            Ordering::Greater => Err(LineIndexOutOfBoundsError {
                given: line_index,
                max: self.last_line_index(),
            }),
        }
    }

    fn last_line_index(&self) -> LineIndex {
        LineIndex::from(self.line_starts.len() as RawIndex)
    }

    fn line_span(&self, line_index: LineIndex) -> Result<Span, LineIndexOutOfBoundsError> {
        let line_start = self.line_start(line_index)?;
        let next_line_start = self.line_start(line_index + LineOffset::from(1))?;

        Ok(Span::new(line_start, next_line_start))
    }

    fn location(&self, byte_index: ByteIndex) -> Result<Location, LocationError> {
        use unicode_segmentation::UnicodeSegmentation;

        match self.line_starts.binary_search(&byte_index) {
            // Found the start of a line
            Ok(line) => Ok(Location::new(line as u32, 0)),
            // Found something in the middle of a line
            Err(next_line) => {
                let line_index = LineIndex::from(next_line as u32 - 1);
                let line_start_index =
                    self.line_start(line_index)
                        .map_err(|_| LocationError::OutOfBounds {
                            given: byte_index,
                            span: self.source_span(),
                        })?;
                let line_src = self
                    .source()
                    .get(line_start_index.to_usize()..byte_index.to_usize())
                    .ok_or_else(|| {
                        let given = byte_index;
                        if given >= self.source_span().end() {
                            let span = self.source_span();
                            LocationError::OutOfBounds { given, span }
                        } else {
                            LocationError::InvalidCharBoundary { given }
                        }
                    })?;

                Ok(Location {
                    line: line_index,
                    column: ColumnIndex::from(line_src.graphemes(true).count() as u32),
                })
            },
        }
    }

    fn source(&self) -> &str {
        &self.source
    }

    fn source_span(&self) -> Span {
        Span::from_str(self.source())
    }

    fn source_slice(&self, span: Span) -> Result<&str, SpanOutOfBoundsError> {
        let start = span.start().to_usize();
        let end = span.end().to_usize();

        self.source.get(start..end).ok_or_else(|| {
            let span = Span::from_str(self.source());
            SpanOutOfBoundsError { given: span, span }
        })
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use super::*;

    const TEST_SOURCE: &str = "foo\nbar\r\n\nbaz";

    #[test]
    fn line_starts() {
        let mut files = Files::new();
        let file_id = files.add("test", TEST_SOURCE);

        assert_eq!(
            files.get(file_id).line_starts,
            [
                ByteIndex::from(0),  // "foo\n"
                ByteIndex::from(4),  // "bar\r\n"
                ByteIndex::from(9),  // ""
                ByteIndex::from(10), // "baz"
            ],
        );
    }

    #[test]
    fn line_span_sources() {
        let mut files = Files::new();
        let file_id = files.add("test", TEST_SOURCE);

        let line_sources = (0..4)
            .map(|line| {
                let line_span = files.line_span(file_id, line).unwrap();
                files.source_slice(file_id, line_span)
            })
            .collect::<Vec<_>>();

        assert_eq!(
            line_sources,
            [Ok("foo\n"), Ok("bar\r\n"), Ok("\n"), Ok("baz")],
        );
    }
}
