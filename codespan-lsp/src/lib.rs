//! Utilities for translating from codespan types into Language Server Protocol (LSP) types

use codespan::{
    ByteIndex, ByteOffset, ColumnIndex, FileId, Files, LineIndex, LineIndexOutOfBoundsError,
    LocationError, RawIndex, RawOffset, Span, SpanOutOfBoundsError,
};
use lsp_types as lsp;
use std::ffi::OsString;
use std::path::PathBuf;
use std::{error, fmt};

#[derive(Debug, PartialEq)]
pub enum Error {
    UnableToCorrelateFilename(OsString),
    ColumnOutOfBounds {
        given: ColumnIndex,
        max: ColumnIndex,
    },
    Location(LocationError),
    LineIndexOutOfBounds(LineIndexOutOfBoundsError),
    SpanOutOfBounds(SpanOutOfBoundsError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::UnableToCorrelateFilename(s) => {
                let p = PathBuf::from(s);
                write!(f, "Unable to correlate filename `{}` to url", p.display())
            },
            Error::ColumnOutOfBounds { given, max } => {
                write!(f, "Column out of bounds - given: {}, max: {}", given, max)
            },
            Error::Location(e) => e.fmt(f),
            Error::LineIndexOutOfBounds(e) => e.fmt(f),
            Error::SpanOutOfBounds(e) => e.fmt(f),
        }
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
            Error::UnableToCorrelateFilename(_) | Error::ColumnOutOfBounds { .. } => None,
            Error::Location(error) => Some(error),
            Error::LineIndexOutOfBounds(error) => Some(error),
            Error::SpanOutOfBounds(error) => Some(error),
        }
    }
}

fn location_to_position(
    line_str: &str,
    line: LineIndex,
    column: ColumnIndex,
    byte_index: ByteIndex,
) -> Result<lsp::Position, Error> {
    if column.to_usize() > line_str.len() {
        let max = ColumnIndex(line_str.len() as RawIndex);
        let given = column;

        Err(Error::ColumnOutOfBounds { given, max })
    } else if !line_str.is_char_boundary(column.to_usize()) {
        let given = byte_index;

        Err(LocationError::InvalidCharBoundary { given }.into())
    } else {
        let line_utf16 = line_str[..column.to_usize()].encode_utf16();
        let character = line_utf16.count() as u64;
        let line = line.to_usize() as u64;

        Ok(lsp::Position { line, character })
    }
}

pub fn byte_index_to_position<Source: AsRef<str>>(
    files: &Files<Source>,
    file_id: FileId,
    byte_index: ByteIndex,
) -> Result<lsp::Position, Error> {
    let location = files.location(file_id, byte_index)?;
    let line_span = files.line_span(file_id, location.line)?;
    let line_str = files.source_slice(file_id, line_span)?;
    let column = ColumnIndex::from((byte_index - line_span.start()).0 as RawIndex);

    location_to_position(line_str, location.line, column, byte_index)
}

pub fn byte_span_to_range<Source: AsRef<str>>(
    files: &Files<Source>,
    file_id: FileId,
    span: Span,
) -> Result<lsp::Range, Error> {
    Ok(lsp::Range {
        start: byte_index_to_position(files, file_id, span.start())?,
        end: byte_index_to_position(files, file_id, span.end())?,
    })
}

pub fn character_to_line_offset(line: &str, character: u64) -> Result<ByteOffset, Error> {
    let line_len = ByteOffset::from(line.len() as RawOffset);
    let mut character_offset = 0;

    let mut chars = line.chars();
    while let Some(ch) = chars.next() {
        if character_offset == character {
            let chars_off = ByteOffset::from_str_len(chars.as_str());
            let ch_off = ByteOffset::from_char_len(ch);

            return Ok(line_len - chars_off - ch_off);
        }

        character_offset += ch.len_utf16() as u64;
    }

    // Handle positions after the last character on the line
    if character_offset == character {
        Ok(line_len)
    } else {
        Err(Error::ColumnOutOfBounds {
            given: ColumnIndex(character_offset as RawIndex),
            max: ColumnIndex(line.len() as RawIndex),
        })
    }
}

pub fn position_to_byte_index<Source: AsRef<str>>(
    files: &Files<Source>,
    file_id: FileId,
    position: &lsp::Position,
) -> Result<ByteIndex, Error> {
    let line_span = files.line_span(file_id, position.line as RawIndex)?;
    let source = files.source_slice(file_id, line_span)?;
    let byte_offset = character_to_line_offset(source, position.character)?;

    Ok(line_span.start() + byte_offset)
}

pub fn range_to_byte_span<Source: AsRef<str>>(
    files: &Files<Source>,
    file_id: FileId,
    range: &lsp::Range,
) -> Result<Span, Error> {
    Ok(Span::new(
        position_to_byte_index(files, file_id, &range.start)?,
        position_to_byte_index(files, file_id, &range.end)?,
    ))
}

#[cfg(test)]
mod tests {
    use codespan::Location;

    use super::*;

    #[test]
    fn position() {
        let text = r#"
let test = 2
let test1 = ""
test
"#;
        let mut files = Files::new();
        let file_id = files.add("test", text);
        let pos = position_to_byte_index(
            &files,
            file_id,
            &lsp::Position {
                line: 3,
                character: 2,
            },
        )
        .unwrap();
        assert_eq!(Location::new(3, 2), files.location(file_id, pos).unwrap());
    }

    // The protocol specifies that each `character` in position is a UTF-16 character.
    // This means that `å` and `ä` here counts as 1 while `𐐀` counts as 2.
    const UNICODE: &str = "åä t𐐀b";

    #[test]
    fn unicode_get_byte_index() {
        let mut files = Files::new();
        let file_id = files.add("unicode", UNICODE);

        let result = position_to_byte_index(
            &files,
            file_id,
            &lsp::Position {
                line: 0,
                character: 3,
            },
        );
        assert_eq!(result, Ok(ByteIndex::from(5)));

        let result = position_to_byte_index(
            &files,
            file_id,
            &lsp::Position {
                line: 0,
                character: 6,
            },
        );
        assert_eq!(result, Ok(ByteIndex::from(10)));
    }

    #[test]
    fn unicode_get_position() {
        let mut files = Files::new();
        let file_id = files.add("unicode", UNICODE);

        let result = byte_index_to_position(&files, file_id, ByteIndex::from(5));
        assert_eq!(
            result,
            Ok(lsp::Position {
                line: 0,
                character: 3,
            })
        );

        let result = byte_index_to_position(&files, file_id, ByteIndex::from(10));
        assert_eq!(
            result,
            Ok(lsp::Position {
                line: 0,
                character: 6,
            })
        );
    }
}
