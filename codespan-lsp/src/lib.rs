//! Utilities for translating from codespan types into Language Server Protocol (LSP) types

extern crate codespan;
extern crate codespan_reporting;

extern crate failure;
#[macro_use]
extern crate failure_derive;

extern crate languageserver_types;
extern crate url;

use codespan::{ByteIndex, ByteIndexError, ByteOffset, ColumnIndex, FileName, LineIndex,
               LineIndexError, LocationError, RawIndex, RawOffset, Span};

use url::Url;

use languageserver_types as lsp;

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
        Err(LocationError::ColumnOutOfBounds {
            given: column,
            max: ColumnIndex(line_str.len() as RawIndex),
        }.into())
    } else if !line_str.is_char_boundary(column.to_usize()) {
        Err(ByteIndexError::InvalidCharBoundary { given: byte_index }.into())
    } else {
        let character = line_str[..column.to_usize()].encode_utf16().count() as u64;
        Ok(lsp::Position {
            line: line.to_usize() as u64,
            character,
        })
    }
}

pub fn byte_index_to_position<S>(
    source: &codespan::FileMap<S>,
    pos: ByteIndex,
) -> Result<lsp::Position, Error>
where
    S: AsRef<str>,
{
    let (line_str, line, column) = source.find_line(pos).map(|line| {
        let line_span = source.line_span(line).unwrap();
        let line_src = source.src_slice(line_span).unwrap();
        (
            line_src,
            line,
            ColumnIndex::from((pos - line_span.start()).0 as RawIndex),
        )
    })?;
    location_to_position(line_str, line, column, pos)
}

pub fn byte_span_to_range<S>(
    source: &codespan::FileMap<S>,
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
    let mut character_offset = 0;
    let mut found = None;

    let mut chars = line.chars();
    while let Some(c) = chars.next() {
        if character_offset == character {
            found = Some(line.len() - chars.as_str().len() - c.len_utf8());
            break;
        }
        character_offset += c.len_utf16() as u64;
    }

    found
        .or_else(|| {
            // Handle positions after the last character on the line
            if character_offset == character {
                Some(line.len())
            } else {
                None
            }
        })
        .map(|i| ByteOffset::from(i as RawOffset))
        .ok_or_else(|| {
            LocationError::ColumnOutOfBounds {
                given: ColumnIndex(character_offset as RawIndex),
                max: ColumnIndex(line.len() as RawIndex),
            }.into()
        })
}

pub fn position_to_byte_index<S>(
    source: &codespan::FileMap<S>,
    position: &lsp::Position,
) -> Result<ByteIndex, Error>
where
    S: AsRef<str>,
{
    let line_span = source.line_span(LineIndex::from(position.line as RawIndex))?;
    character_to_line_offset(source.src_slice(line_span).unwrap(), position.character)
        .map(|byte_offset| line_span.start() + byte_offset)
}

pub fn range_to_byte_span<S>(
    source: &codespan::FileMap<S>,
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

/// Translates a `codespan_reporting::Diagnostic` to a `languageserver_types::Diagnostic`.
///
/// Since the language client requires `Url`s to locate the errors `codespan_name_to_file` is
/// necessary to resolve codespan `FileName`s
///
/// `code` and `source` are left empty by this function
pub fn make_lsp_diagnostic<F>(
    mut codespan_name_to_file: F,
    code_map: &codespan::CodeMap,
    diagnostic: codespan_reporting::Diagnostic,
) -> Result<lsp::Diagnostic, Error>
where
    F: FnMut(&codespan::FileName) -> Result<Url, ()>,
{
    make_lsp_diagnostic_(&mut codespan_name_to_file, code_map, diagnostic)
}

fn make_lsp_diagnostic_(
    codespan_name_to_file: &mut FnMut(&codespan::FileName) -> Result<Url, ()>,
    code_map: &codespan::CodeMap,
    diagnostic: codespan_reporting::Diagnostic,
) -> Result<lsp::Diagnostic, Error> {
    let unknown = lsp::Position {
        character: 0,
        line: 0,
    };
    let unknown_range = lsp::Range {
        start: unknown,
        end: unknown,
    };

    // We need a position for the primary error so take the span from the first primary label
    let mut primary_file_map = None;
    let range = match diagnostic
        .labels
        .iter()
        .find(|label| label.style == codespan_reporting::LabelStyle::Primary)
    {
        Some(label) => {
            let file_map = code_map
                .find_file(label.span.start())
                .ok_or_else(|| Error::SpanOutsideCodeMap(label.span.start()))?;
            primary_file_map = Some(file_map);
            byte_span_to_range(&file_map, label.span)?
        }
        None => unknown_range,
    };

    let related_information = diagnostic
        .labels
        .into_iter()
        .map(|label| {
            let location = match primary_file_map {
                // If the label's span does not point anywhere, assume it comes from the same file
                // as the primary label
                Some(file_map) if label.span.start() == ByteIndex::none() => lsp::Location {
                    uri: codespan_name_to_file(file_map.name())
                        .map_err(|()| Error::UnableToCorrelateFilename(file_map.name().clone()))?,
                    range: unknown_range,
                },
                _ => {
                    let file_map = code_map
                        .find_file(label.span.start())
                        .ok_or_else(|| Error::SpanOutsideCodeMap(label.span.start()))?;
                    lsp::Location {
                        uri: codespan_name_to_file(file_map.name()).map_err(|()| {
                            Error::UnableToCorrelateFilename(file_map.name().clone())
                        })?,
                        range: byte_span_to_range(file_map, label.span)?,
                    }
                }
            };
            Ok(lsp::DiagnosticRelatedInformation {
                location,
                message: label.message.unwrap_or(String::new()),
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;
    Ok(lsp::Diagnostic {
        message: diagnostic.message,
        range,
        severity: Some(match diagnostic.severity {
            codespan_reporting::Severity::Error | codespan_reporting::Severity::Bug => {
                lsp::DiagnosticSeverity::Error
            }
            codespan_reporting::Severity::Warning => lsp::DiagnosticSeverity::Warning,
            codespan_reporting::Severity::Note => lsp::DiagnosticSeverity::Information,
            codespan_reporting::Severity::Help => lsp::DiagnosticSeverity::Hint,
        }),
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
        let source = codespan::FileMap::new("".into(), text);
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
        let source = codespan::FileMap::new("".into(), UNICODE);

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
        let source = codespan::FileMap::new("".into(), UNICODE);

        let result = byte_index_to_position(&source, ByteIndex::from(6));
        assert_eq!(
            result,
            Ok(lsp::Position {
                line: 0,
                character: 3,
            },)
        );

        let result = byte_index_to_position(&source, ByteIndex::from(11));
        assert_eq!(
            result,
            Ok(lsp::Position {
                line: 0,
                character: 6,
            },)
        );
    }
}
