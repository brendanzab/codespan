//! Various source mapping utilities

use std::borrow::Cow;
use std::{fmt, io};
use std::path::PathBuf;

use index::{ByteIndex, ByteOffset, ColumnIndex, LineIndex, LineOffset, RawIndex, RawOffset};
use span::ByteSpan;

#[derive(Clone, Debug)]
pub enum FileName {
    /// A real file on disk
    Real(PathBuf),
    /// A synthetic file, eg. from the REPL
    Virtual(Cow<'static, str>),
}

impl FileName {
    pub fn real<T: Into<PathBuf>>(name: T) -> FileName {
        FileName::Real(name.into())
    }

    pub fn virtual_<T: Into<Cow<'static, str>>>(name: T) -> FileName {
        FileName::Virtual(name.into())
    }
}

impl fmt::Display for FileName {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            FileName::Real(ref path) => write!(fmt, "{}", path.display()),
            FileName::Virtual(ref name) => write!(fmt, "<{}>", name),
        }
    }
}

#[derive(Debug, Fail, PartialEq)]
pub enum LineIndexError {
    #[fail(display = "Line out of bounds - given: {:?}, max: {:?}", given, max)]
    OutOfBounds { given: LineIndex, max: LineIndex },
}

#[derive(Debug, Fail, PartialEq)]
pub enum ByteIndexError {
    #[fail(display = "Byte index out of bounds - given: {}, span: {}", given, span)]
    OutOfBounds { given: ByteIndex, span: ByteSpan },
    #[fail(display = "Byte index points within a character boundary - given: {}", given)]
    InvalidCharBoundary { given: ByteIndex },
}

#[derive(Debug, Fail, PartialEq)]
pub enum SpanError {
    #[fail(display = "Span out of bounds - given: {}, span: {}", given, span)]
    OutOfBounds { given: ByteSpan, span: ByteSpan },
}

#[derive(Debug)]
/// Some source code
pub struct FileMap {
    /// The name of the file that the source came from
    name: FileName,
    /// The complete source code
    src: String,
    /// The span of the source in the `CodeMap`
    span: ByteSpan,
    /// Offsets to the line beginnings in the source
    lines: Vec<ByteOffset>,
}

impl FileMap {
    pub fn anonymous(src: String) -> FileMap {
        Self::new(FileName::virtual_(""), src, ByteIndex::from(0))
    }

    /// Construct a new filemap, creating an index of line start locations
    pub(crate) fn new(name: FileName, src: String, start: ByteIndex) -> FileMap {
        use std::iter;

        let span = ByteSpan::from_offset(start, ByteOffset::from_str(&src));
        let lines = {
            let newline_off = ByteOffset::from_char_utf8('\n');
            let offsets = src.match_indices('\n')
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

    /// Read some source code from a file, loading it into a filemap
    pub(crate) fn from_disk<P: Into<PathBuf>>(name: P, start: ByteIndex) -> io::Result<FileMap> {
        use std::fs::File;
        use std::io::Read;

        let name = name.into();
        let mut file = File::open(&name)?;
        let mut src = String::new();
        file.read_to_string(&mut src)?;

        Ok(FileMap::new(FileName::Real(name), src, start))
    }

    /// The name of the file that the source came from
    pub fn name(&self) -> &FileName {
        &self.name
    }

    /// The underlying source code
    pub fn src(&self) -> &str {
        &self.src
    }

    /// The span of the source in the `CodeMap`
    pub fn span(&self) -> ByteSpan {
        self.span
    }

    /// Returns the byte offset to the start of `line`
    pub fn line_offset(&self, index: LineIndex) -> Result<ByteOffset, LineIndexError> {
        self.lines
            .get(index.to_usize())
            .cloned()
            .ok_or_else(|| LineIndexError::OutOfBounds {
                given: index,
                max: LineIndex(self.lines.len() as RawIndex - 1),
            })
    }

    /// Returns the byte index of the start of `line`
    pub fn line_byte_index(&self, index: LineIndex) -> Result<ByteIndex, LineIndexError> {
        self.line_offset(index)
            .map(|offset| self.span.start() + offset)
    }

    /// Returns the byte offset to the start of `line`
    pub fn line_span(&self, line: LineIndex) -> Result<ByteSpan, LineIndexError> {
        let start = self.span.start() + self.line_offset(line)?;
        let end = match self.line_offset(line + LineOffset(1)) {
            Ok(offset_hi) => self.span.start() + offset_hi,
            Err(_) => self.span.end(),
        };

        Ok(ByteSpan::new(end, start))
    }

    /// Returns the line and column location of `byte`
    pub fn location(&self, index: ByteIndex) -> Result<(LineIndex, ColumnIndex), ByteIndexError> {
        let line_index = self.find_line(index)?;
        let line_span = self.line_span(line_index).unwrap(); // line_index should be valid!
        let line_slice = self.src_slice(line_span).unwrap(); // line_span should be valid!
        let byte_col = index - line_span.start();
        let column_index =
            ColumnIndex(line_slice[..byte_col.to_usize()].chars().count() as RawIndex);

        Ok((line_index, column_index))
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

            if self.src.is_char_boundary(offset.to_usize()) {
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
            Ok(&self.src[start..end])
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
    use std::sync::Arc;
    use {CodeMap, FileMap, FileName};

    use super::*;

    struct TestData {
        filemap: Arc<FileMap>,
        lines: &'static [&'static str],
    }

    impl TestData {
        fn new() -> TestData {
            let mut codemap = CodeMap::new();
            let lines = &["hello!\n", "howdy\n", "\n", "hi萤\n", "bloop\n"];
            let filemap = codemap.add_filemap(FileName::Virtual("test".into()), lines.concat());

            TestData { filemap, lines }
        }

        fn byte_offsets(&self) -> Vec<ByteOffset> {
            let mut offset = ByteOffset(0);
            let mut byte_offsets = Vec::new();

            for line in self.lines {
                byte_offsets.push(offset);
                offset += ByteOffset::from_str(line);
                byte_offsets.push(offset + -ByteOffset::from_char_utf8('\n'));
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
                Ok(ByteOffset(14)),
                Ok(ByteOffset(20)),
                Ok(ByteOffset(26)),
                Err(LineIndexError::OutOfBounds {
                    given: LineIndex(6),
                    max: LineIndex(5),
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
                Ok(test_data.filemap.span().start() + ByteOffset(14)),
                Ok(test_data.filemap.span().start() + ByteOffset(20)),
                Ok(test_data.filemap.span().start() + ByteOffset(26)),
                Err(LineIndexError::OutOfBounds {
                    given: LineIndex(6),
                    max: LineIndex(5),
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
                Ok((LineIndex(0), ColumnIndex(0))),
                Ok((LineIndex(0), ColumnIndex(6))),
                Ok((LineIndex(1), ColumnIndex(0))),
                Ok((LineIndex(1), ColumnIndex(5))),
                Ok((LineIndex(2), ColumnIndex(0))),
                Ok((LineIndex(2), ColumnIndex(0))),
                Ok((LineIndex(3), ColumnIndex(0))),
                Ok((LineIndex(3), ColumnIndex(3))),
                Ok((LineIndex(4), ColumnIndex(0))),
                Ok((LineIndex(4), ColumnIndex(5))),
                Ok((LineIndex(5), ColumnIndex(0))),
                Err(ByteIndexError::OutOfBounds {
                    given: ByteIndex(28),
                    span: test_data.filemap.span(),
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
                Err(ByteIndexError::OutOfBounds {
                    given: ByteIndex(28),
                    span: test_data.filemap.span(),
                }),
            ],
        );
    }
}
