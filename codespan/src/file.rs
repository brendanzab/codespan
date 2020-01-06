#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};
use std::num::NonZeroU32;
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
pub struct FileId(NonZeroU32);

impl FileId {
    /// Offset of our `FileId`'s numeric value to an index on `Files::files`.
    ///
    /// This is to ensure the first `FileId` is non-zero for memory layout optimisations (e.g.
    /// `Option<FileId>` is 4 bytes)
    const OFFSET: u32 = 1;

    fn new(index: usize) -> FileId {
        FileId(NonZeroU32::new(index as u32 + Self::OFFSET).unwrap())
    }

    fn get(self) -> usize {
        (self.0.get() - Self::OFFSET) as usize
    }
}

/// A database of source files.
///
/// The `Source` generic parameter determines how source text is stored. Using [`String`] will have
/// `Files` take ownership of all source text. Smart pointer types such as [`Cow<'_, str>`],
/// [`Rc<str>`] or [`Arc<str>`] can be used to share the source text with the rest of the program.
///
/// [`Cow<'_, str>`]: std::borrow::Cow
/// [`Rc<str>`]: std::rc::Rc
/// [`Arc<str>`]: std::sync::Arc
#[derive(Clone, Debug)]
pub struct Files<Source>
where
    Source: AsRef<str>,
{
    files: Vec<File<Source>>,
}

impl<Source> Default for Files<Source>
where
    Source: AsRef<str>,
{
    fn default() -> Self {
        Self { files: vec![] }
    }
}

impl<Source> Files<Source>
where
    Source: AsRef<str>,
{
    /// Create a new, empty database of files.
    pub fn new() -> Self {
        Files::<Source>::default()
    }

    /// Add a file to the database, returning the handle that can be used to
    /// refer to it again.
    pub fn add(&mut self, name: impl Into<String>, source: Source) -> FileId {
        let file_id = FileId::new(self.files.len());
        self.files.push(File::new(name.into(), source.into()));
        file_id
    }

    /// Update a source file in place.
    ///
    /// This will mean that any outstanding byte indexes will now point to
    /// invalid locations.
    pub fn update(&mut self, file_id: FileId, source: Source) {
        self.get_mut(file_id).update(source.into())
    }

    /// Get a the source file using the file id.
    // FIXME: return an option or result?
    fn get(&self, file_id: FileId) -> &File<Source> {
        &self.files[file_id.get()]
    }

    /// Get a the source file using the file id.
    // FIXME: return an option or result?
    fn get_mut(&mut self, file_id: FileId) -> &mut File<Source> {
        &mut self.files[file_id.get()]
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
    /// assert_eq!(*files.source(file_id), source);
    /// ```
    pub fn source(&self, file_id: FileId) -> &Source {
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
struct File<Source>
where
    Source: AsRef<str>,
{
    /// The name of the file.
    name: String,
    /// The source code of the file.
    source: Source,
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

impl<Source> File<Source>
where
    Source: AsRef<str>,
{
    fn new(name: String, source: Source) -> Self {
        let line_starts = compute_line_starts(source.as_ref());

        File {
            name,
            source,
            line_starts,
        }
    }

    fn update(&mut self, source: Source) {
        let line_starts = compute_line_starts(source.as_ref());
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
                    .source
                    .as_ref()
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

    fn source(&self) -> &Source {
        &self.source
    }

    fn source_span(&self) -> Span {
        Span::from_str(self.source.as_ref())
    }

    fn source_slice(&self, span: Span) -> Result<&str, SpanOutOfBoundsError> {
        let start = span.start().to_usize();
        let end = span.end().to_usize();

        self.source.as_ref().get(start..end).ok_or_else(|| {
            let span = Span::from_str(self.source.as_ref());
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
        let mut files = Files::<String>::new();
        let file_id = files.add("test", TEST_SOURCE.to_owned());

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
        // Also make sure we can use `Arc` for source
        use std::sync::Arc;

        let mut files = Files::<Arc<str>>::new();
        let file_id = files.add("test", TEST_SOURCE.into());

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
