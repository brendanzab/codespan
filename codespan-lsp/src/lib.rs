//! Utilities for translating from codespan types into Language Server Protocol (LSP) types

use std::{error, fmt, ops::Range};

use codespan_reporting::files::Files;

// WARNING: Be extremely careful when adding new imports here, as it could break
// the compatible version range that we claim in our `Cargo.toml`. This could
// potentially break down-stream builds on a `cargo update`. This is an
// absolute no-no, breaking much of what we enjoy about Cargo!
use lsp_types::{Position as LspPosition, Range as LspRange};

#[derive(Debug, PartialEq)]
pub enum Error {
    ColumnOutOfBounds { given: usize, max: usize },
    Location(LocationError),
    LineIndexOutOfBounds(LineIndexOutOfBoundsError),
    SpanOutOfBounds(SpanOutOfBoundsError),
    MissingFile,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ColumnOutOfBounds { given, max } => {
                write!(f, "Column out of bounds - given: {}, max: {}", given, max)
            }
            Error::Location(e) => e.fmt(f),
            Error::LineIndexOutOfBounds(e) => e.fmt(f),
            Error::SpanOutOfBounds(e) => e.fmt(f),
            Error::MissingFile => write!(f, "File does not exit"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct LineIndexOutOfBoundsError {
    pub given: usize,
    pub max: usize,
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
    OutOfBounds { given: usize, span: Range<usize> },
    InvalidCharBoundary { given: usize },
}

impl error::Error for LocationError {}

impl fmt::Display for LocationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LocationError::OutOfBounds { given, span } => write!(
                f,
                "Byte index out of bounds - given: {}, span: {}..{}",
                given, span.start, span.end
            ),
            LocationError::InvalidCharBoundary { given } => {
                write!(f, "Byte index within character boundary - given: {}", given)
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct SpanOutOfBoundsError {
    pub given: Range<usize>,
    pub span: Range<usize>,
}

impl error::Error for SpanOutOfBoundsError {}

impl fmt::Display for SpanOutOfBoundsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Span out of bounds - given: {}..{}, span: {}..{}",
            self.given.start, self.given.end, self.span.start, self.span.end
        )
    }
}

impl From<LocationError> for Error {
    fn from(e: LocationError) -> Error {
        Error::Location(e)
    }
}

impl From<LineIndexOutOfBoundsError> for Error {
    fn from(e: LineIndexOutOfBoundsError) -> Error {
        Error::LineIndexOutOfBounds(e)
    }
}

impl From<SpanOutOfBoundsError> for Error {
    fn from(e: SpanOutOfBoundsError) -> Error {
        Error::SpanOutOfBounds(e)
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::ColumnOutOfBounds { .. } | Error::MissingFile => None,
            Error::Location(error) => Some(error),
            Error::LineIndexOutOfBounds(error) => Some(error),
            Error::SpanOutOfBounds(error) => Some(error),
        }
    }
}

fn location_to_position(
    line_str: &str,
    line: usize,
    column: usize,
    byte_index: usize,
) -> Result<LspPosition, Error> {
    if column > line_str.len() {
        let max = line_str.len();
        let given = column;

        Err(Error::ColumnOutOfBounds { given, max })
    } else if !line_str.is_char_boundary(column) {
        let given = byte_index;

        Err(LocationError::InvalidCharBoundary { given }.into())
    } else {
        let line_utf16 = line_str[..column].encode_utf16();
        let character = line_utf16.count() as u64;
        let line = line as u64;

        Ok(LspPosition { line, character })
    }
}

pub fn byte_index_to_position<'a, F>(
    files: &'a F,
    file_id: F::FileId,
    byte_index: usize,
) -> Result<LspPosition, Error>
where
    F: Files<'a> + ?Sized,
{
    let source = files.source(file_id).ok_or_else(|| Error::MissingFile)?;
    let source = source.as_ref();

    let line_index =
        files
            .line_index(file_id, byte_index)
            .ok_or_else(|| LineIndexOutOfBoundsError {
                given: byte_index,
                max: source.lines().count(),
            })?;
    let line_span = files.line_range(file_id, line_index).unwrap();

    let line_str = source
        .get(line_span.clone())
        .ok_or_else(|| SpanOutOfBoundsError {
            given: line_span.clone(),
            span: 0..source.len(),
        })?;
    let column = byte_index - line_span.start;

    location_to_position(line_str, line_index, column, byte_index)
}

pub fn byte_span_to_range<'a, F>(
    files: &'a F,
    file_id: F::FileId,
    span: Range<usize>,
) -> Result<LspRange, Error>
where
    F: Files<'a> + ?Sized,
{
    Ok(LspRange {
        start: byte_index_to_position(files, file_id, span.start)?,
        end: byte_index_to_position(files, file_id, span.end)?,
    })
}

pub fn character_to_line_offset(line: &str, character: u64) -> Result<usize, Error> {
    let line_len = line.len();
    let mut character_offset = 0;

    let mut chars = line.chars();
    while let Some(ch) = chars.next() {
        if character_offset == character {
            let chars_off = chars.as_str().len();
            let ch_off = ch.len_utf8();

            return Ok(line_len - chars_off - ch_off);
        }

        character_offset += ch.len_utf16() as u64;
    }

    // Handle positions after the last character on the line
    if character_offset == character {
        Ok(line_len)
    } else {
        Err(Error::ColumnOutOfBounds {
            given: character_offset as usize,
            max: line.len(),
        })
    }
}

pub fn position_to_byte_index<'a, F>(
    files: &'a F,
    file_id: F::FileId,
    position: &LspPosition,
) -> Result<usize, Error>
where
    F: Files<'a> + ?Sized,
{
    let source = files.source(file_id).ok_or_else(|| Error::MissingFile)?;
    let source = source.as_ref();

    let line_span = files.line_range(file_id, position.line as usize).unwrap();

    let byte_offset = character_to_line_offset(source, position.character)?;

    Ok(line_span.start + byte_offset)
}

pub fn range_to_byte_span<'a, F>(
    files: &'a F,
    file_id: F::FileId,
    range: &LspRange,
) -> Result<Range<usize>, Error>
where
    F: Files<'a> + ?Sized,
{
    Ok(position_to_byte_index(files, file_id, &range.start)?
        ..position_to_byte_index(files, file_id, &range.end)?)
}

#[cfg(test)]
mod tests {
    use codespan_reporting::files::{Location, SimpleFiles};

    use super::*;

    #[test]
    fn position() {
        let text = r#"
let test = 2
let test1 = ""
test
"#;
        let mut files = SimpleFiles::new();
        let file_id = files.add("test", text);
        let pos = position_to_byte_index(
            &files,
            file_id,
            &LspPosition {
                line: 3,
                character: 2,
            },
        )
        .unwrap();
        assert_eq!(
            Location {
                // One-based
                line_number: 3 + 1,
                column_number: 2 + 1,
            },
            files.location(file_id, pos).unwrap()
        );
    }

    // The protocol specifies that each `character` in position is a UTF-16 character.
    // This means that `å` and `ä` here counts as 1 while `𐐀` counts as 2.
    const UNICODE: &str = "åä t𐐀b";

    #[test]
    fn unicode_get_byte_index() {
        let mut files = SimpleFiles::new();
        let file_id = files.add("unicode", UNICODE);

        let result = position_to_byte_index(
            &files,
            file_id,
            &LspPosition {
                line: 0,
                character: 3,
            },
        );
        assert_eq!(result, Ok(5));

        let result = position_to_byte_index(
            &files,
            file_id,
            &LspPosition {
                line: 0,
                character: 6,
            },
        );
        assert_eq!(result, Ok(10));
    }

    #[test]
    fn unicode_get_position() {
        let mut files = SimpleFiles::new();
        let file_id = files.add("unicode", UNICODE);

        let result = byte_index_to_position(&files, file_id, 5);
        assert_eq!(
            result,
            Ok(LspPosition {
                line: 0,
                character: 3,
            })
        );

        let result = byte_index_to_position(&files, file_id, 10);
        assert_eq!(
            result,
            Ok(LspPosition {
                line: 0,
                character: 6,
            })
        );
    }
}
