//! Various source mapping utilities

use std::borrow::Cow;
use std::{fmt, io};
use std::path::PathBuf;

use pos::{ByteOffset, BytePos, ColumnIndex, LineIndex, RawOffset, RawPos, Span};

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
pub enum BytePosError {
    #[fail(display = "Byte position out of bounds - given: {}, span: {}", given, span)]
    OutOfBounds { given: BytePos, span: Span },
}

#[derive(Debug, Fail, PartialEq)]
pub enum SpanError {
    #[fail(display = "Span out of bounds - given: {}, span: {}", given, span)]
    OutOfBounds { given: Span, span: Span },
}

#[derive(Debug)]
/// Some source code
pub struct FileMap {
    /// The name of the file that the source came from
    name: FileName,
    /// The complete source code
    src: String,
    /// The span of the source in the `CodeMap`
    span: Span,
    /// Offsets to the line beginnings in the source
    lines: Vec<ByteOffset>,
}

impl FileMap {
    /// Construct a new filemap, creating an index of line start locations
    pub(crate) fn new(name: FileName, src: String, start_pos: BytePos) -> FileMap {
        use std::iter;

        let span = Span::from_offset(start_pos, ByteOffset::from_str(&src));
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
    pub(crate) fn from_disk<P: Into<PathBuf>>(name: P, start_pos: BytePos) -> io::Result<FileMap> {
        use std::fs::File;
        use std::io::Read;

        let name = name.into();
        let mut file = File::open(&name)?;
        let mut src = String::new();
        file.read_to_string(&mut src)?;

        Ok(FileMap::new(FileName::Real(name), src, start_pos))
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
    pub fn span(&self) -> Span {
        self.span
    }

    /// Returns the byte offset to the start of `line`
    pub fn line_offset(&self, index: LineIndex) -> Result<ByteOffset, LineIndexError> {
        self.lines
            .get(index.to_usize())
            .cloned()
            .ok_or_else(|| LineIndexError::OutOfBounds {
                given: index,
                max: LineIndex(self.lines.len() as RawPos - 1),
            })
    }

    /// Returns the byte offset to the start of `line`
    pub fn line_pos(&self, index: LineIndex) -> Result<BytePos, LineIndexError> {
        self.line_offset(index)
            .map(|offset| self.span.lo() + offset)
    }

    /// Returns the byte offset to the start of `line`
    pub fn line_span(&self, line: LineIndex) -> Result<Span, LineIndexError> {
        let lo = self.span.lo() + self.line_offset(line)?;
        let hi = match self.line_offset(LineIndex(line.0 + 1)) {
            Ok(offset_hi) => self.span.lo() + offset_hi,
            Err(_) => self.span.hi(),
        };

        Ok(Span::new(hi, lo))
    }

    /// Returns the line and column location of `byte`
    pub fn location(&self, pos: BytePos) -> Result<(LineIndex, ColumnIndex), BytePosError> {
        let line_index = self.find_line_at_pos(pos)?;
        let line_span = self.line_span(line_index).unwrap(); // line_index should be valid!
        let line_slice = self.src_slice(line_span).unwrap(); // line_span should be valid!
        let byte_col = pos - line_span.lo();
        let column_index = ColumnIndex(line_slice[..byte_col.to_usize()].chars().count() as RawPos);

        Ok((line_index, column_index))
    }

    /// Returns the line index that the byte position points to
    pub fn find_line_at_pos(&self, pos: BytePos) -> Result<LineIndex, BytePosError> {
        let span = self.span;
        match () {
            () if pos < span.lo() => Err(BytePosError::OutOfBounds { given: pos, span }),
            () if pos > span.hi() => Err(BytePosError::OutOfBounds { given: pos, span }),
            () => match self.lines.binary_search(&(pos - span.lo())) {
                Ok(i) => Ok(LineIndex(i as RawPos)),
                Err(i) => Ok(LineIndex(i as RawPos - 1)),
            },
        }
    }

    /// Get the corresponding source string for a span
    ///
    /// Returns `Err` if the span is outside the bounds of the file
    pub fn src_slice(&self, span: Span) -> Result<&str, SpanError> {
        match () {
            () if !self.span.contains(span) => Err(SpanError::OutOfBounds {
                given: span,
                span: self.span,
            }),
            () => {
                let lo = (span.lo() - self.span.lo()).to_usize();
                let hi = (span.hi() - self.span.lo()).to_usize();

                Ok(&self.src[lo..hi])
            },
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

        fn byte_positions(&self) -> Vec<BytePos> {
            let mut offsets = vec![BytePos::none()];
            offsets.extend(self.byte_offsets().iter().map(|&off| BytePos(1) + off));
            let out_of_bounds = *offsets.last().unwrap() + ByteOffset(1);
            offsets.push(out_of_bounds);
            offsets
        }

        fn line_indices(&self) -> Vec<LineIndex> {
            (0..self.lines.len() + 2)
                .map(|i| LineIndex(i as RawPos))
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
    fn line_pos() {
        let test_data = TestData::new();
        let offsets: Vec<_> = test_data
            .line_indices()
            .iter()
            .map(|&i| test_data.filemap.line_pos(i))
            .collect();

        assert_eq!(
            offsets,
            vec![
                Ok(test_data.filemap.span().lo() + ByteOffset(0)),
                Ok(test_data.filemap.span().lo() + ByteOffset(7)),
                Ok(test_data.filemap.span().lo() + ByteOffset(13)),
                Ok(test_data.filemap.span().lo() + ByteOffset(14)),
                Ok(test_data.filemap.span().lo() + ByteOffset(20)),
                Ok(test_data.filemap.span().lo() + ByteOffset(26)),
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
    //     let lo = filemap.span().lo();

    //     assert_eq!(filemap.line_pos(Li(0)), Some(lo + BOff(0)));
    //     assert_eq!(filemap.line_pos(Li(1)), Some(lo + BOff(7)));
    //     assert_eq!(filemap.line_pos(Li(2)), Some(lo + BOff(13)));
    //     assert_eq!(filemap.line_pos(Li(3)), Some(lo + BOff(14)));
    //     assert_eq!(filemap.line_pos(Li(4)), Some(lo + BOff(20)));
    //     assert_eq!(filemap.line_pos(Li(5)), Some(lo + BOff(26)));
    //     assert_eq!(filemap.line_pos(Li(6)), None);
    // }

    #[test]
    fn location() {
        let test_data = TestData::new();
        let lines: Vec<_> = test_data
            .byte_positions()
            .iter()
            .map(|&pos| test_data.filemap.location(pos))
            .collect();

        assert_eq!(
            lines,
            vec![
                Err(BytePosError::OutOfBounds {
                    given: BytePos(0),
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
                Err(BytePosError::OutOfBounds {
                    given: BytePos(28),
                    span: test_data.filemap.span(),
                }),
            ],
        );
    }

    #[test]
    fn find_line_at_pos() {
        let test_data = TestData::new();
        let lines: Vec<_> = test_data
            .byte_positions()
            .iter()
            .map(|&pos| test_data.filemap.find_line_at_pos(pos))
            .collect();

        assert_eq!(
            lines,
            vec![
                Err(BytePosError::OutOfBounds {
                    given: BytePos(0),
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
                Err(BytePosError::OutOfBounds {
                    given: BytePos(28),
                    span: test_data.filemap.span(),
                }),
            ],
        );
    }
}
