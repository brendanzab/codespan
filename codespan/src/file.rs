#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};

use crate::{ByteIndex, ColumnIndex, LineIndex, Location, Span};

/// A handle that points to a file in the database.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "memory_usage", derive(heapsize_derive::HeapSizeOf))]
pub struct FileId(usize);

/// A span that is situated in a source file.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "memory_usage", derive(heapsize_derive::HeapSizeOf))]
pub struct FileSpan {
    pub id: FileId,
    pub span: Span,
}

impl FileSpan {
    pub fn new(id: FileId, span: Span) -> FileSpan {
        FileSpan { id, span }
    }
}

/// The contents of a file that is stored in the database.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialization", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "memory_usage", derive(heapsize_derive::HeapSizeOf))]
struct File {
    /// The name of the file.
    name: String,
    /// The source code of the file.
    contents: String,
    /// The starting byte indices in the source code.
    line_starts: Vec<ByteIndex>,
}

impl File {
    fn new(name: String, contents: String) -> File {
        // Pre-compute the line starting positions
        let line_starts = std::iter::once(0)
            .chain(contents.match_indices('\n').map(|(i, _)| i as u32 + 1))
            .chain(std::iter::once(contents.len() as u32))
            .map(ByteIndex::from)
            .collect();

        File {
            name,
            contents,
            line_starts,
        }
    }

    /// Get the name of the file.
    fn name(&self) -> &str {
        &self.name
    }

    /// Get a slice to the contents of the file.
    fn contents(&self) -> &str {
        &self.contents
    }

    /// Get a slice to the line start indices of the file.
    fn line_starts(&self) -> &[ByteIndex] {
        &self.line_starts
    }

    fn byte_index(&self, line: LineIndex, column: ColumnIndex) -> Option<ByteIndex> {
        let line_start = *self.line_starts().get(line.to_usize())?;

        Some(column.to_byte_index(self.contents(), line_start))
    }

    fn line_span(&self, line_index: LineIndex) -> Option<Span> {
        let line_start = *self.line_starts().get(line_index.to_usize())?;
        let next_line_start = *self.line_starts().get(line_index.to_usize() + 1)?;

        Some(Span::new(line_start, next_line_start))
    }

    fn location(&self, byte_index: ByteIndex) -> Option<Location> {
        let line_starts = self.line_starts();
        match line_starts.binary_search(&byte_index) {
            // Found the start of a line
            Ok(line) => Some(Location {
                line: LineIndex::from(line as u32),
                column: ColumnIndex::from(0),
            }),
            // Found something in the middle of a line
            Err(next_line) => {
                let line = LineIndex::from(next_line as u32 - 1);
                let line_start = line_starts[line.to_usize()];
                let column = ColumnIndex::from_str(self.contents(), line_start, byte_index)?;

                Some(Location { line, column })
            },
        }
    }

    /// Return a slice of the source file, given a span.
    fn source(&self, span: Span) -> Option<&str> {
        let start = span.start().to_usize();
        let end = span.end().to_usize();

        self.contents.get(start..end)
    }
}

/// A database of source files.
#[derive(Debug, Clone)]
pub struct Files {
    files: Vec<File>,
}

impl Files {
    /// Create a new, empty database.
    pub fn new() -> Files {
        Files { files: Vec::new() }
    }

    /// Add a file to the database, returning the handle that can be used to refer to it again.
    pub fn add(&mut self, name: impl Into<String>, contents: impl Into<String>) -> FileId {
        let file_id = FileId(self.files.len());
        self.files.push(File::new(name.into(), contents.into()));
        file_id
    }

    /// Get the name of the file.
    pub fn name(&self, file_id: FileId) -> &str {
        self.files[file_id.0].name()
    }

    pub fn byte_index(
        &self,
        file_id: FileId,
        line: impl Into<LineIndex>,
        column: impl Into<ColumnIndex>,
    ) -> Option<ByteIndex> {
        self.files[file_id.0].byte_index(line.into(), column.into())
    }

    pub fn line_span(&self, file_id: FileId, line_index: impl Into<LineIndex>) -> Option<Span> {
        self.files[file_id.0].line_span(line_index.into())
    }

    pub fn location(&self, file_id: FileId, byte_index: impl Into<ByteIndex>) -> Option<Location> {
        self.files[file_id.0].location(byte_index.into())
    }

    /// Return a slice of the source file, given a span.
    pub fn source(&self, file_id: FileId, span: Span) -> Option<&str> {
        self.files[file_id.0].source(span)
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn line_starts() {
        let mut files = Files::new();
        let file_id = files.add("test", "foo\nbar\r\n\nbaz");

        assert_eq!(
            files.files[file_id.0].line_starts(),
            [
                ByteIndex::from(0),  // "foo\n"
                ByteIndex::from(4),  // "bar\r\n"
                ByteIndex::from(9),  // ""
                ByteIndex::from(10), // "baz"
                ByteIndex::from(13),
            ],
        );
    }

    #[test]
    fn location() {
        let mut files = Files::new();
        let file_id = files.add("test", "foo\nbar\r\n\nbaz");

        assert_eq!(files.location(file_id, 0), Some(Location::new(0, 0)),);
        assert_eq!(files.location(file_id, 7), Some(Location::new(1, 3)),);
        assert_eq!(files.location(file_id, 8), Some(Location::new(1, 4)),);
        assert_eq!(files.location(file_id, 9), Some(Location::new(2, 0)),);
        assert_eq!(files.location(file_id, 100), None);
    }

    #[test]
    fn line_span_sources() {
        let mut files = Files::new();
        let file_id = files.add("test", "foo\nbar\r\n\nbaz");

        let line_sources = (0..5)
            .map(|line| {
                let line_span = files.line_span(file_id, line)?;
                files.source(file_id, line_span)
            })
            .collect::<Vec<_>>();

        assert_eq!(
            line_sources,
            [
                Some("foo\n"),
                Some("bar\r\n"),
                Some("\n"),
                Some("baz"),
                None,
            ],
        );
    }
}
