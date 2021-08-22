use codespan_reporting::files::Error;
#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};
use std::ffi::{OsStr, OsString};
use std::num::NonZeroU32;

use crate::{ByteIndex, ColumnIndex, LineIndex, LineOffset, Location, RawIndex, Span};

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
        FileId(NonZeroU32::new(index as u32 + Self::OFFSET).expect("file index cannot be stored"))
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
pub struct Files<Source> {
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
    pub fn add(&mut self, name: impl Into<OsString>, source: Source) -> FileId {
        let file_id = FileId::new(self.files.len());
        self.files.push(File::new(name.into(), source));
        file_id
    }

    /// Update a source file in place.
    ///
    /// This will mean that any outstanding byte indexes will now point to
    /// invalid locations.
    pub fn update(&mut self, file_id: FileId, source: Source) {
        self.get_mut(file_id).update(source)
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
    pub fn name(&self, file_id: FileId) -> &OsStr {
        self.get(file_id).name()
    }

    /// Get the span at the given line index.
    ///
    /// ```rust
    /// use codespan::{Files, LineIndex, Span};
    ///
    /// let mut files = Files::new();
    /// let file_id = files.add("test", "foo\nbar\r\n\nbaz");
    ///
    /// let line_sources = (0..4)
    ///     .map(|line| files.line_span(file_id, line).unwrap())
    ///     .collect::<Vec<_>>();
    ///
    /// assert_eq!(line_sources,
    ///     [
    ///         Span::new(0, 4),    // 0: "foo\n"
    ///         Span::new(4, 9),    // 1: "bar\r\n"
    ///         Span::new(9, 10),   // 2: ""
    ///         Span::new(10, 13),  // 3: "baz"
    ///     ]
    /// );
    /// assert!(files.line_span(file_id, 4).is_err());
    /// ```
    pub fn line_span(
        &self,
        file_id: FileId,
        line_index: impl Into<LineIndex>,
    ) -> Result<Span, Error> {
        self.get(file_id).line_span(line_index.into())
    }

    /// Get the line index at the given byte in the source file.
    ///
    /// ```rust
    /// use codespan::{Files, LineIndex};
    ///
    /// let mut files = Files::new();
    /// let file_id = files.add("test", "foo\nbar\r\n\nbaz");
    ///
    /// assert_eq!(files.line_index(file_id, 0), LineIndex::from(0));
    /// assert_eq!(files.line_index(file_id, 7), LineIndex::from(1));
    /// assert_eq!(files.line_index(file_id, 8), LineIndex::from(1));
    /// assert_eq!(files.line_index(file_id, 9), LineIndex::from(2));
    /// assert_eq!(files.line_index(file_id, 100), LineIndex::from(3));
    /// ```
    pub fn line_index(&self, file_id: FileId, byte_index: impl Into<ByteIndex>) -> LineIndex {
        self.get(file_id).line_index(byte_index.into())
    }

    /// Get the location at the given byte index in the source file.
    ///
    /// ```rust
    /// use codespan::{ByteIndex, Files, Location, Span};
    ///
    /// let mut files = Files::new();
    /// let file_id = files.add("test", "foo\nbar\r\n\nbaz");
    ///
    /// assert_eq!(files.location(file_id, 0).unwrap(), Location::new(0, 0));
    /// assert_eq!(files.location(file_id, 7).unwrap(), Location::new(1, 3));
    /// assert_eq!(files.location(file_id, 8).unwrap(), Location::new(1, 4));
    /// assert_eq!(files.location(file_id, 9).unwrap(), Location::new(2, 0));
    /// assert!(files.location(file_id, 100).is_err());
    /// ```
    pub fn location(
        &self,
        file_id: FileId,
        byte_index: impl Into<ByteIndex>,
    ) -> Result<Location, Error> {
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
    /// assert_eq!(files.source_slice(file_id, Span::new(0, 5)).unwrap(), "hello");
    /// assert!(files.source_slice(file_id, Span::new(0, 100)).is_err());
    /// ```
    pub fn source_slice(&self, file_id: FileId, span: impl Into<Span>) -> Result<&str, Error> {
        self.get(file_id).source_slice(span.into())
    }
}

impl<'a, Source> codespan_reporting::files::Files<'a> for Files<Source>
where
    Source: AsRef<str>,
{
    type FileId = FileId;
    type Name = String;
    type Source = &'a str;

    fn name(&self, id: FileId) -> Result<String, Error> {
        use std::path::PathBuf;

        Ok(PathBuf::from(self.name(id)).display().to_string())
    }

    fn source(&'a self, id: FileId) -> Result<&str, Error> {
        Ok(self.source(id).as_ref())
    }

    fn line_index(&self, id: FileId, byte_index: usize) -> Result<usize, Error> {
        Ok(self.line_index(id, byte_index as u32).to_usize())
    }

    fn line_range(
        &'a self,
        id: FileId,
        line_index: usize,
    ) -> Result<std::ops::Range<usize>, Error> {
        let span = self.line_span(id, line_index as u32)?;

        Ok(span.start().to_usize()..span.end().to_usize())
    }
}

/// A file that is stored in the database.
#[derive(Debug, Clone)]
// `Serialize` is only implemented on `OsString` for windows/unix
#[cfg_attr(
    all(feature = "serialization", any(windows, unix)),
    derive(Deserialize, Serialize)
)]
struct File<Source> {
    /// The name of the file.
    name: OsString,
    /// The source code of the file.
    source: Source,
    /// The starting byte indices in the source code.
    line_starts: Vec<ByteIndex>,
}

impl<Source> File<Source>
where
    Source: AsRef<str>,
{
    fn new(name: OsString, source: Source) -> Self {
        let line_starts = line_starts(source.as_ref())
            .map(|i| ByteIndex::from(i as u32))
            .collect();

        File {
            name,
            source,
            line_starts,
        }
    }

    fn update(&mut self, source: Source) {
        let line_starts = line_starts(source.as_ref())
            .map(|i| ByteIndex::from(i as u32))
            .collect();
        self.source = source;
        self.line_starts = line_starts;
    }

    fn name(&self) -> &OsStr {
        &self.name
    }

    fn line_start(&self, line_index: LineIndex) -> Result<ByteIndex, Error> {
        use std::cmp::Ordering;

        match line_index.cmp(&self.last_line_index()) {
            Ordering::Less => Ok(self.line_starts[line_index.to_usize()]),
            Ordering::Equal => Ok(self.source_span().end()),
            Ordering::Greater => Err(Error::LineTooLarge {
                given: line_index.to_usize(),
                max: self.last_line_index().to_usize(),
            }),
        }
    }

    fn last_line_index(&self) -> LineIndex {
        LineIndex::from(self.line_starts.len() as RawIndex)
    }

    fn line_span(&self, line_index: LineIndex) -> Result<Span, Error> {
        let line_start = self.line_start(line_index)?;
        let next_line_start = self.line_start(line_index + LineOffset::from(1))?;

        Ok(Span::new(line_start, next_line_start))
    }

    fn line_index(&self, byte_index: ByteIndex) -> LineIndex {
        match self.line_starts.binary_search(&byte_index) {
            // Found the start of a line
            Ok(line) => LineIndex::from(line as u32),
            Err(next_line) => LineIndex::from(next_line as u32 - 1),
        }
    }

    fn location(&self, byte_index: ByteIndex) -> Result<Location, Error> {
        let line_index = self.line_index(byte_index);
        let line_start_index = self
            .line_start(line_index)
            .map_err(|_| Error::IndexTooLarge {
                given: byte_index.to_usize(),
                max: self.source().as_ref().len() - 1,
            })?;
        let line_src = self
            .source
            .as_ref()
            .get(line_start_index.to_usize()..byte_index.to_usize())
            .ok_or_else(|| {
                let given = byte_index.to_usize();
                let max = self.source().as_ref().len() - 1;
                if given > max {
                    Error::IndexTooLarge { given, max }
                } else {
                    Error::InvalidCharBoundary { given }
                }
            })?;

        Ok(Location {
            line: line_index,
            column: ColumnIndex::from(line_src.chars().count() as u32),
        })
    }

    fn source(&self) -> &Source {
        &self.source
    }

    fn source_span(&self) -> Span {
        Span::from_str(self.source.as_ref())
    }

    fn source_slice(&self, span: Span) -> Result<&str, Error> {
        let start = span.start().to_usize();
        let end = span.end().to_usize();

        self.source.as_ref().get(start..end).ok_or_else(|| {
            let max = self.source().as_ref().len() - 1;
            Error::IndexTooLarge {
                given: if start > max { start } else { end },
                max,
            }
        })
    }
}

// NOTE: this is copied from `codespan_reporting::files::line_starts` and should be kept in sync.
fn line_starts(source: &str) -> impl '_ + Iterator<Item = usize> {
    std::iter::once(0).chain(source.match_indices('\n').map(|(i, _)| i + 1))
}

#[cfg(test)]
mod test {
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
                files.source_slice(file_id, line_span).unwrap()
            })
            .collect::<Vec<_>>();

        assert_eq!(line_sources, ["foo\n", "bar\r\n", "\n", "baz"],);
    }

    #[test]
    fn interoperability() {
        extern crate termcolor;
        use codespan_reporting::{diagnostic::*, term::emit};
        use termcolor::{ColorChoice, StandardStream};

        let mut files = Files::<String>::new();
        let file_id = files.add("test", TEST_SOURCE.to_owned());

        let diagnostic = Diagnostic::note()
            .with_message("middle")
            .with_labels(vec![Label::primary(file_id, 4..7).with_message("middle")]);

        let config = codespan_reporting::term::Config::default();
        emit(
            &mut StandardStream::stdout(ColorChoice::Auto),
            &config,
            &files,
            &diagnostic,
        )
        .unwrap();
    }
}
