//! Utilities for translating from codespan types into Language Server Protocol (LSP) types

extern crate codespan;
extern crate codespan_reporting;

extern crate failure;
#[macro_use]
extern crate failure_derive;

extern crate languageserver_types;
extern crate url;

use codespan::{
    ByteIndex, ByteIndexError, ByteOffset, CodeMap, ColumnIndex, FileMap, FileName, LineIndex,
    LineIndexError, LocationError, RawIndex, RawOffset, Span,
};
use codespan_reporting::{Diagnostic, Severity};
use languageserver_types as lsp;
use url::Url;

#[derive(Debug, Fail, PartialEq)]
pub enum Error {
    #[fail(display = "Position is outside of codemap {}", _0)]
    SpanOutsideCodeMap(ByteIndex),
    #[fail(display = "Unable to correlate filename `{}` to url", _0)]
    UnableToCorrelateFilename(FileName),
    #[fail(display = "{}", _0)]
    ByteIndexError(#[cause] ByteIndexError),
    #[fail(display = "{}", _0)]
    LocationError(#[cause] LocationError),
    #[fail(display = "{}", _0)]
    LineIndexError(#[cause] LineIndexError),
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

pub fn byte_index_to_position<S>(
    source: &FileMap<S>,
    pos: ByteIndex,
) -> Result<lsp::Position, Error>
where
    S: AsRef<str>,
{
    let line = source.find_line(pos)?;
    let line_span = source.line_span(line).unwrap();
    let line_str = source.src_slice(line_span).unwrap();
    let column = ColumnIndex::from((pos - line_span.start()).0 as RawIndex);

    location_to_position(line_str, line, column, pos)
}

pub fn byte_span_to_range<S>(
    source: &FileMap<S>,
    span: Span<ByteIndex>,
) -> Result<lsp::Range, Error>
where
    S: AsRef<str>,
{
    Ok(lsp::Range {
        start: byte_index_to_position(source, span.start())?,
        end: byte_index_to_position(source, span.end())?,
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
        }.into())
    }
}

pub fn position_to_byte_index<S>(
    source: &FileMap<S>,
    position: &lsp::Position,
) -> Result<ByteIndex, Error>
where
    S: AsRef<str>,
{
    let line_span = source.line_span(LineIndex::from(position.line as RawIndex))?;
    let src_slice = source.src_slice(line_span).unwrap();
    let byte_offset = character_to_line_offset(src_slice, position.character)?;

    Ok(line_span.start() + byte_offset)
}

pub fn range_to_byte_span<S>(
    source: &FileMap<S>,
    range: &lsp::Range,
) -> Result<Span<ByteIndex>, Error>
where
    S: AsRef<str>,
{
    Ok(Span::new(
        position_to_byte_index(source, &range.start)?,
        position_to_byte_index(source, &range.end)?,
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
/// `code` and `source` are left empty by this function
pub fn make_lsp_diagnostic<F>(
    code_map: &CodeMap,
    diagnostic: Diagnostic,
    mut codespan_name_to_file: F,
) -> Result<lsp::Diagnostic, Error>
where
    F: FnMut(&FileName) -> Result<Url, ()>,
{
    use codespan_reporting::LabelStyle;

    let find_file = |index| {
        code_map
            .find_file(index)
            .ok_or_else(|| Error::SpanOutsideCodeMap(index))
    };

    // We need a position for the primary error so take the span from the first primary label
    let (primary_file_map, primary_label_range) = {
        let first_primary_label = diagnostic
            .labels
            .iter()
            .find(|label| label.style == LabelStyle::Primary);

        match first_primary_label {
            Some(label) => {
                let file_map = find_file(label.span.start())?;
                (Some(file_map), byte_span_to_range(&file_map, label.span)?)
            },
            None => (None, UNKNOWN_RANGE),
        }
    };

    let related_information = diagnostic
        .labels
        .into_iter()
        .map(|label| {
            let (file_map, range) = match primary_file_map {
                // If the label's span does not point anywhere, assume it comes from the same file
                // as the primary label
                Some(file_map) if label.span.start() == ByteIndex::none() => {
                    (file_map, UNKNOWN_RANGE)
                },
                Some(_) | None => {
                    let file_map = find_file(label.span.start())?;
                    let range = byte_span_to_range(file_map, label.span)?;

                    (file_map, range)
                },
            };

            let uri = codespan_name_to_file(file_map.name())
                .map_err(|()| Error::UnableToCorrelateFilename(file_map.name().clone()))?;

            Ok(lsp::DiagnosticRelatedInformation {
                location: lsp::Location { uri, range },
                message: label.message.unwrap_or(String::new()),
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
    use super::*;

    #[test]
    fn position() {
        let text = r#"
let test = 2
let test1 = ""
te
"#;
        let source = FileMap::new("".into(), text);
        let pos = position_to_byte_index(
            &source,
            &lsp::Position {
                line: 3,
                character: 2,
            },
        ).unwrap();
        assert_eq!((3.into(), 2.into()), source.location(pos).unwrap());
    }

    // The protocol specifies that each `character` in position is a UTF-16 character.
    // This means that `å` and `ä` here counts as 1 while `𐐀` counts as 2.
    const UNICODE: &str = "åä t𐐀b";

    #[test]
    fn unicode_get_byte_index() {
        let source = FileMap::new("".into(), UNICODE);

        let result = position_to_byte_index(
            &source,
            &lsp::Position {
                line: 0,
                character: 3,
            },
        );
        assert_eq!(result, Ok(ByteIndex::from(6)));

        let result = position_to_byte_index(
            &source,
            &lsp::Position {
                line: 0,
                character: 6,
            },
        );
        assert_eq!(result, Ok(ByteIndex::from(11)));
    }

    #[test]
    fn unicode_get_position() {
        let source = FileMap::new("".into(), UNICODE);

        let result = byte_index_to_position(&source, ByteIndex::from(6));
        assert_eq!(
            result,
            Ok(lsp::Position {
                line: 0,
                character: 3,
            })
        );

        let result = byte_index_to_position(&source, ByteIndex::from(11));
        assert_eq!(
            result,
            Ok(lsp::Position {
                line: 0,
                character: 6,
            })
        );
    }
}
