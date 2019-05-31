//! Utilities for translating from codespan types into Language Server Protocol (LSP) types

use codespan::{
    ByteIndex, ByteIndexError, ByteOffset, CodeMap, ColumnIndex, File, LineIndex, LineIndexError,
    LocationError, RawIndex, RawOffset, Span,
};
use codespan_reporting::{Diagnostic, Severity};
use lsp_types as lsp;
use std::error;
use std::fmt;
use url::Url;

#[derive(Debug, PartialEq)]
pub enum Error {
    SpanOutsideCodeMap(ByteIndex),
    UnableToCorrelateFilename(String),
    ByteIndexError(ByteIndexError),
    LocationError(LocationError),
    LineIndexError(LineIndexError),
}

impl From<ByteIndexError> for Error {
    fn from(e: ByteIndexError) -> Error {
        Error::ByteIndexError(e)
    }
}

impl From<LocationError> for Error {
    fn from(e: LocationError) -> Error {
        Error::LocationError(e)
    }
}

impl From<LineIndexError> for Error {
    fn from(e: LineIndexError) -> Error {
        Error::LineIndexError(e)
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::SpanOutsideCodeMap(_) | Error::UnableToCorrelateFilename(_) => None,
            Error::ByteIndexError(error) => Some(error),
            Error::LocationError(error) => Some(error),
            Error::LineIndexError(error) => Some(error),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::SpanOutsideCodeMap(index) => {
                write!(f, "Position is outside of codemap {}", index)
            },
            Error::UnableToCorrelateFilename(name) => {
                write!(f, "Unable to correlate filename `{}` to url", name)
            },
            Error::ByteIndexError(error) => write!(f, "{}", error),
            Error::LocationError(error) => write!(f, "{}", error),
            Error::LineIndexError(error) => write!(f, "{}", error),
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

        Err(LocationError::ColumnOutOfBounds { given, max }.into())
    } else if !line_str.is_char_boundary(column.to_usize()) {
        let given = byte_index;

        Err(ByteIndexError::InvalidCharBoundary { given }.into())
    } else {
        let line_utf16 = line_str[..column.to_usize()].encode_utf16();
        let character = line_utf16.count() as u64;
        let line = line.to_usize() as u64;

        Ok(lsp::Position { line, character })
    }
}

pub fn byte_index_to_position<S>(file: &File<S>, pos: ByteIndex) -> Result<lsp::Position, Error>
where
    S: AsRef<str>,
{
    let line = file.find_line(pos)?;
    let line_span = file.line_span(line).unwrap();
    let line_str = file.src_slice(line_span).unwrap();
    let column = ColumnIndex::from((pos - line_span.start()).0 as RawIndex);

    location_to_position(line_str, line, column, pos)
}

pub fn byte_span_to_range<S>(file: &File<S>, span: Span<ByteIndex>) -> Result<lsp::Range, Error>
where
    S: AsRef<str>,
{
    Ok(lsp::Range {
        start: byte_index_to_position(file, span.start())?,
        end: byte_index_to_position(file, span.end())?,
    })
}

pub fn character_to_line_offset(line: &str, character: u64) -> Result<ByteOffset, Error> {
    let line_len = ByteOffset::from(line.len() as RawOffset);
    let mut character_offset = 0;

    let mut chars = line.chars();
    while let Some(ch) = chars.next() {
        if character_offset == character {
            let chars_off = ByteOffset::from_str(chars.as_str());
            let ch_off = ByteOffset::from_char_utf8(ch);

            return Ok(line_len - chars_off - ch_off);
        }

        character_offset += ch.len_utf16() as u64;
    }

    // Handle positions after the last character on the line
    if character_offset == character {
        Ok(line_len)
    } else {
        Err(LocationError::ColumnOutOfBounds {
            given: ColumnIndex(character_offset as RawIndex),
            max: ColumnIndex(line.len() as RawIndex),
        }
        .into())
    }
}

pub fn position_to_byte_index<S>(
    file: &File<S>,
    position: &lsp::Position,
) -> Result<ByteIndex, Error>
where
    S: AsRef<str>,
{
    let line_span = file.line_span(LineIndex::from(position.line as RawIndex))?;
    let src_slice = file.src_slice(line_span).unwrap();
    let byte_offset = character_to_line_offset(src_slice, position.character)?;

    Ok(line_span.start() + byte_offset)
}

pub fn range_to_byte_span<S>(file: &File<S>, range: &lsp::Range) -> Result<Span<ByteIndex>, Error>
where
    S: AsRef<str>,
{
    Ok(Span::new(
        position_to_byte_index(file, &range.start)?,
        position_to_byte_index(file, &range.end)?,
    ))
}

pub fn make_lsp_severity(severity: Severity) -> lsp::DiagnosticSeverity {
    match severity {
        Severity::Error | Severity::Bug => lsp::DiagnosticSeverity::Error,
        Severity::Warning => lsp::DiagnosticSeverity::Warning,
        Severity::Note => lsp::DiagnosticSeverity::Information,
        Severity::Help => lsp::DiagnosticSeverity::Hint,
    }
}

const UNKNOWN_POS: lsp::Position = lsp::Position {
    character: 0,
    line: 0,
};

const UNKNOWN_RANGE: lsp::Range = lsp::Range {
    start: UNKNOWN_POS,
    end: UNKNOWN_POS,
};

/// Translates a `codespan_reporting::Diagnostic` to a `languageserver_types::Diagnostic`.
///
/// Since the language client requires `Url`s to locate the errors `codespan_name_to_file` is
/// necessary to resolve codespan `FileName`s
///
/// `code` and `file` are left empty by this function
pub fn make_lsp_diagnostic<F>(
    code_map: &CodeMap,
    diagnostic: Diagnostic,
    mut codespan_name_to_file: F,
) -> Result<lsp::Diagnostic, Error>
where
    F: FnMut(&str) -> Result<Url, ()>,
{
    let find_file = |index| {
        code_map
            .find_file(index)
            .ok_or_else(|| Error::SpanOutsideCodeMap(index))
    };

    // We need a position for the primary error so take the span from the first primary label
    let primary_file = find_file(diagnostic.primary_label.span.start())?;
    let primary_label_range = byte_span_to_range(&primary_file, diagnostic.primary_label.span)?;

    let related_information = diagnostic
        .secondary_labels
        .into_iter()
        .map(|label| {
            // If the label's span does not point anywhere, assume it comes from the same file
            // as the primary label
            let (file, range) = if label.span.start() == ByteIndex::none() {
                (primary_file, UNKNOWN_RANGE)
            } else {
                let file = find_file(label.span.start())?;
                let range = byte_span_to_range(file, label.span)?;

                (file, range)
            };

            let uri = codespan_name_to_file(file.name())
                .map_err(|()| Error::UnableToCorrelateFilename(file.name().to_owned()))?;

            Ok(lsp::DiagnosticRelatedInformation {
                location: lsp::Location { uri, range },
                message: label.message,
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;

    Ok(lsp::Diagnostic {
        message: diagnostic.message,
        range: primary_label_range,
        severity: Some(make_lsp_severity(diagnostic.severity)),
        related_information: if related_information.is_empty() {
            None
        } else {
            Some(related_information)
        },
        ..lsp::Diagnostic::default()
    })
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
te
"#;
        let file = File::new("".into(), text);
        let pos = position_to_byte_index(
            &file,
            &lsp::Position {
                line: 3,
                character: 2,
            },
        )
        .unwrap();
        assert_eq!(Location::new(3, 2), file.location(pos).unwrap());
    }

    // The protocol specifies that each `character` in position is a UTF-16 character.
    // This means that `å` and `ä` here counts as 1 while `𐐀` counts as 2.
    const UNICODE: &str = "åä t𐐀b";

    #[test]
    fn unicode_get_byte_index() {
        let file = File::new("".into(), UNICODE);

        let result = position_to_byte_index(
            &file,
            &lsp::Position {
                line: 0,
                character: 3,
            },
        );
        assert_eq!(result, Ok(ByteIndex::from(6)));

        let result = position_to_byte_index(
            &file,
            &lsp::Position {
                line: 0,
                character: 6,
            },
        );
        assert_eq!(result, Ok(ByteIndex::from(11)));
    }

    #[test]
    fn unicode_get_position() {
        let file = File::new("".into(), UNICODE);

        let result = byte_index_to_position(&file, ByteIndex::from(6));
        assert_eq!(
            result,
            Ok(lsp::Position {
                line: 0,
                character: 3,
            })
        );

        let result = byte_index_to_position(&file, ByteIndex::from(11));
        assert_eq!(
            result,
            Ok(lsp::Position {
                line: 0,
                character: 6,
            })
        );
    }
}
