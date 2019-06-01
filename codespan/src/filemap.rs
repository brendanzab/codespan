//! Various source mapping utilities

#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};
use std::{error, fmt, io};

use crate::index::{
    ByteIndex, ByteOffset, ColumnIndex, LineIndex, LineOffset, RawIndex, RawOffset,
};
use crate::location::Location;
use crate::span::ByteSpan;

#[derive(Debug, PartialEq)]
pub enum LineIndexError {
    OutOfBounds { given: LineIndex, max: LineIndex },
}

impl error::Error for LineIndexError {}

impl fmt::Display for LineIndexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LineIndexError::OutOfBounds { given, max } => {
                write!(f, "Line out of bounds - given: {:?}, max: {:?}", given, max)
            },
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ByteIndexError {
    OutOfBounds { given: ByteIndex, span: ByteSpan },
    InvalidCharBoundary { given: ByteIndex },
}

impl error::Error for ByteIndexError {}

impl fmt::Display for ByteIndexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ByteIndexError::OutOfBounds { given, span } => write!(
                f,
                "Byte index out of bounds - given: {}, span: {}",
                given, span,
            ),
            ByteIndexError::InvalidCharBoundary { given } => write!(
                f,
                "Byte index points within a character boundary - given: {}",
                given,
            ),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum LocationError {
    LineOutOfBounds {
        given: LineIndex,
        max: LineIndex,
    },
    ColumnOutOfBounds {
        given: ColumnIndex,
        max: ColumnIndex,
    },
}

impl error::Error for LocationError {}

impl fmt::Display for LocationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LocationError::LineOutOfBounds { given, max } => {
                write!(f, "Line out of bounds - given: {:?}, max: {:?}", given, max)
            },
            LocationError::ColumnOutOfBounds { given, max } => write!(
                f,
                "Column out of bounds - given: {:?}, max: {:?}",
                given, max,
            ),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum SpanError {
    OutOfBounds { given: ByteSpan, span: ByteSpan },
}

impl error::Error for SpanError {}

impl fmt::Display for SpanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpanError::OutOfBounds { given, span } => {
                write!(f, "Span out of bounds - given: {}, span: {}", given, span)
            },
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "memory_usage", derive(heapsize_derive::HeapSizeOf))]
/// Some source code
pub struct FileMap<S = String> {
    /// The name of the file that the source came from, to be used when
    /// displaying diagnostics
    name: String,
    /// The complete source code
    src: S,
    /// The span of the source in the `CodeMap`
    span: ByteSpan,
    /// Offsets to the line beginnings in the source
    lines: Vec<ByteOffset>,
}

impl<S: AsRef<str> + From<String>> FileMap<S> {
    /// Read some source code from a file, loading it into a filemap
    pub(crate) fn from_disk(name: String, start: ByteIndex) -> io::Result<FileMap<S>> {
        use std::fs::File;
        use std::io::Read;

        let mut file = File::open(&name)?;
        let mut src = String::new();
        file.read_to_string(&mut src)?;

        Ok(FileMap::with_index(name, src.into(), start))
    }
}

impl<S> FileMap<S>
where
    S: AsRef<str>,
{
    /// Construct a new, standalone filemap.
    ///
    /// This can be useful for tests that consist of a single source file. Production code should however
    /// use `CodeMap::add_filemap` or `CodeMap::add_filemap_from_disk` instead.
    pub fn new(name: String, src: S) -> FileMap<S> {
        FileMap::with_index(name, src, ByteIndex(1))
    }

    pub(crate) fn with_index(name: String, src: S, start: ByteIndex) -> FileMap<S> {
        use std::iter;

        let span = ByteSpan::from_offset(start, ByteOffset::from_str(src.as_ref()));
        let lines = {
            let newline_off = ByteOffset::from_char_utf8('\n');
            let offsets = src
                .as_ref()
                .match_indices('\n')
                .map(|(i, _)| ByteOffset(i as RawOffset) + newline_off);

            iter::once(ByteOffset(0)).chain(offsets).collect()
        };

        FileMap {
            name,
            src,
            span,
            lines,
        }
    }

    /// The name of the file that the source came from
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The underlying source code
    pub fn src(&self) -> &str {
        &self.src.as_ref()
    }

    /// The span of the source in the `CodeMap`
    pub fn span(&self) -> ByteSpan {
        self.span
    }

    pub fn offset(
        &self,
        line: LineIndex,
        column: ColumnIndex,
    ) -> Result<ByteOffset, LocationError> {
        self.byte_index(line, column)
            .map(|index| index - self.span.start())
    }

    pub fn byte_index(
        &self,
        line: LineIndex,
        column: ColumnIndex,
    ) -> Result<ByteIndex, LocationError> {
        self.line_span(line)
            .map_err(
                |LineIndexError::OutOfBounds { given, max }| LocationError::LineOutOfBounds {
                    given,
                    max,
                },
            )
            .and_then(|span| {
                let distance = ColumnIndex(span.end().0 - span.start().0);
                if column > distance {
                    Err(LocationError::ColumnOutOfBounds {
                        given: column,
                        max: distance,
                    })
                } else {
                    Ok(span.start() + ByteOffset::from(column.0 as i64))
                }
            })
    }

    /// Returns the byte offset to the start of `line`.
    ///
    /// Lines may be delimited with either `\n` or `\r\n`.
    pub fn line_offset(&self, index: LineIndex) -> Result<ByteOffset, LineIndexError> {
        self.lines
            .get(index.to_usize())
            .cloned()
            .ok_or_else(|| LineIndexError::OutOfBounds {
                given: index,
                max: LineIndex(self.lines.len() as RawIndex - 1),
            })
    }

    /// Returns the byte index of the start of `line`.
    ///
    /// Lines may be delimited with either `\n` or `\r\n`.
    pub fn line_byte_index(&self, index: LineIndex) -> Result<ByteIndex, LineIndexError> {
        self.line_offset(index)
            .map(|offset| self.span.start() + offset)
    }

    /// Returns the byte offset to the start of `line`.
    ///
    /// Lines may be delimited with either `\n` or `\r\n`.
    pub fn line_span(&self, line: LineIndex) -> Result<ByteSpan, LineIndexError> {
        let start = self.span.start() + self.line_offset(line)?;
        let end = match self.line_offset(line + LineOffset(1)) {
            Ok(offset_hi) => self.span.start() + offset_hi,
            Err(_) => self.span.end(),
        };

        Ok(ByteSpan::new(end, start))
    }

    /// Returns the line and column location of `byte`
    pub fn location(&self, index: ByteIndex) -> Result<Location, ByteIndexError> {
        let line_index = self.find_line(index)?;
        let line_span = self.line_span(line_index).unwrap(); // line_index should be valid!
        let line_slice = self.src_slice(line_span).unwrap(); // line_span should be valid!
        let byte_col = index - line_span.start();
        let column_index =
            ColumnIndex(line_slice[..byte_col.to_usize()].chars().count() as RawIndex);

        Ok(Location::new(line_index, column_index))
    }

    /// Returns the line index that the byte index points to
    pub fn find_line(&self, index: ByteIndex) -> Result<LineIndex, ByteIndexError> {
        if index < self.span.start() || index > self.span.end() {
            Err(ByteIndexError::OutOfBounds {
                given: index,
                span: self.span,
            })
        } else {
            let offset = index - self.span.start();

            if self.src.as_ref().is_char_boundary(offset.to_usize()) {
                match self.lines.binary_search(&offset) {
                    Ok(i) => Ok(LineIndex(i as RawIndex)),
                    Err(i) => Ok(LineIndex(i as RawIndex - 1)),
                }
            } else {
                Err(ByteIndexError::InvalidCharBoundary {
                    given: self.span.start(),
                })
            }
        }
    }

    /// Get the corresponding source string for a span
    ///
    /// Returns `Err` if the span is outside the bounds of the file
    pub fn src_slice(&self, span: ByteSpan) -> Result<&str, SpanError> {
        if self.span.contains(span) {
            let start = (span.start() - self.span.start()).to_usize();
            let end = (span.end() - self.span.start()).to_usize();

            // TODO: check char boundaries
            Ok(&self.src.as_ref()[start..end])
        } else {
            Err(SpanError::OutOfBounds {
                given: span,
                span: self.span,
            })
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use std::sync::Arc;

    use crate::CodeMap;

    struct TestData {
        filemap: Arc<FileMap>,
        lines: &'static [&'static str],
    }

    impl TestData {
        fn new() -> TestData {
            let mut codemap = CodeMap::new();
            let lines = &[
                "hello!\n",
                "howdy\n",
                "\r\n",
                "hi萤\n",
                "bloop\n",
                "goopey\r\n",
            ];
            let filemap = codemap.add_filemap("test".to_owned(), lines.concat());

            TestData { filemap, lines }
        }

        fn byte_offsets(&self) -> Vec<ByteOffset> {
            let mut offset = ByteOffset(0);
            let mut byte_offsets = Vec::new();

            for line in self.lines {
                byte_offsets.push(offset);
                offset += ByteOffset::from_str(line);

                let line_end = if line.ends_with("\r\n") {
                    offset + -ByteOffset::from_char_utf8('\r') + -ByteOffset::from_char_utf8('\n')
                } else if line.ends_with("\n") {
                    offset + -ByteOffset::from_char_utf8('\n')
                } else {
                    offset
                };

                byte_offsets.push(line_end);
            }

            // bump us past the end
            byte_offsets.push(offset);

            byte_offsets
        }

        fn byte_indices(&self) -> Vec<ByteIndex> {
            let mut offsets = vec![ByteIndex::none()];
            offsets.extend(self.byte_offsets().iter().map(|&off| ByteIndex(1) + off));
            let out_of_bounds = *offsets.last().unwrap() + ByteOffset(1);
            offsets.push(out_of_bounds);
            offsets
        }

        fn line_indices(&self) -> Vec<LineIndex> {
            (0..self.lines.len() + 2)
                .map(|i| LineIndex(i as RawIndex))
                .collect()
        }
    }

    #[test]
    fn offset() {
        let test_data = TestData::new();
        assert!(test_data
            .filemap
            .offset(
                (test_data.lines.len() as u32 - 1).into(),
                (test_data.lines.last().unwrap().len() as u32).into()
            )
            .is_ok());
        assert!(test_data
            .filemap
            .offset(
                (test_data.lines.len() as u32 - 1).into(),
                (test_data.lines.last().unwrap().len() as u32 + 1).into()
            )
            .is_err());
    }

    #[test]
    fn line_offset() {
        let test_data = TestData::new();
        let offsets: Vec<_> = test_data
            .line_indices()
            .iter()
            .map(|&i| test_data.filemap.line_offset(i))
            .collect();

        assert_eq!(
            offsets,
            vec![
                Ok(ByteOffset(0)),
                Ok(ByteOffset(7)),
                Ok(ByteOffset(13)),
                Ok(ByteOffset(15)),
                Ok(ByteOffset(21)),
                Ok(ByteOffset(27)),
                Ok(ByteOffset(35)),
                Err(LineIndexError::OutOfBounds {
                    given: LineIndex(7),
                    max: LineIndex(6),
                }),
            ],
        );
    }

    #[test]
    fn line_byte_index() {
        let test_data = TestData::new();
        let offsets: Vec<_> = test_data
            .line_indices()
            .iter()
            .map(|&i| test_data.filemap.line_byte_index(i))
            .collect();

        assert_eq!(
            offsets,
            vec![
                Ok(test_data.filemap.span().start() + ByteOffset(0)),
                Ok(test_data.filemap.span().start() + ByteOffset(7)),
                Ok(test_data.filemap.span().start() + ByteOffset(13)),
                Ok(test_data.filemap.span().start() + ByteOffset(15)),
                Ok(test_data.filemap.span().start() + ByteOffset(21)),
                Ok(test_data.filemap.span().start() + ByteOffset(27)),
                Ok(test_data.filemap.span().start() + ByteOffset(35)),
                Err(LineIndexError::OutOfBounds {
                    given: LineIndex(7),
                    max: LineIndex(6),
                }),
            ],
        );
    }

    // #[test]
    // fn line_span() {
    //     let filemap = filemap();
    //     let start = filemap.span().start();

    //     assert_eq!(filemap.line_byte_index(Li(0)), Some(start + BOff(0)));
    //     assert_eq!(filemap.line_byte_index(Li(1)), Some(start + BOff(7)));
    //     assert_eq!(filemap.line_byte_index(Li(2)), Some(start + BOff(13)));
    //     assert_eq!(filemap.line_byte_index(Li(3)), Some(start + BOff(14)));
    //     assert_eq!(filemap.line_byte_index(Li(4)), Some(start + BOff(20)));
    //     assert_eq!(filemap.line_byte_index(Li(5)), Some(start + BOff(26)));
    //     assert_eq!(filemap.line_byte_index(Li(6)), None);
    // }

    #[test]
    fn location() {
        let test_data = TestData::new();
        let lines: Vec<_> = test_data
            .byte_indices()
            .iter()
            .map(|&index| test_data.filemap.location(index))
            .collect();

        assert_eq!(
            lines,
            vec![
                Err(ByteIndexError::OutOfBounds {
                    given: ByteIndex(0),
                    span: test_data.filemap.span(),
                }),
                Ok(Location::new(0, 0)),
                Ok(Location::new(0, 6)),
                Ok(Location::new(1, 0)),
                Ok(Location::new(1, 5)),
                Ok(Location::new(2, 0)),
                Ok(Location::new(2, 0)),
                Ok(Location::new(3, 0)),
                Ok(Location::new(3, 3)),
                Ok(Location::new(4, 0)),
                Ok(Location::new(4, 5)),
                Ok(Location::new(5, 0)),
                Ok(Location::new(5, 6)),
                Ok(Location::new(6, 0)),
                Err(ByteIndexError::OutOfBounds {
                    given: ByteIndex(37),
                    span: test_data.filemap.span()
                }),
            ],
        );
    }

    #[test]
    fn find_line() {
        let test_data = TestData::new();
        let lines: Vec<_> = test_data
            .byte_indices()
            .iter()
            .map(|&index| test_data.filemap.find_line(index))
            .collect();

        assert_eq!(
            lines,
            vec![
                Err(ByteIndexError::OutOfBounds {
                    given: ByteIndex(0),
                    span: test_data.filemap.span(),
                }),
                Ok(LineIndex(0)),
                Ok(LineIndex(0)),
                Ok(LineIndex(1)),
                Ok(LineIndex(1)),
                Ok(LineIndex(2)),
                Ok(LineIndex(2)),
                Ok(LineIndex(3)),
                Ok(LineIndex(3)),
                Ok(LineIndex(4)),
                Ok(LineIndex(4)),
                Ok(LineIndex(5)),
                Ok(LineIndex(5)),
                Ok(LineIndex(6)),
                Err(ByteIndexError::OutOfBounds {
                    given: ByteIndex(37),
                    span: test_data.filemap.span(),
                }),
            ],
        );
    }
}
