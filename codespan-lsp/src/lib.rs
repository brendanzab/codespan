//! Utilities for translating from codespan types into Language Server Protocol (LSP) types

use codespan::{ByteIndex, ByteSize, FileId, Files, Location, Span};
use codespan_reporting::{Diagnostic, Severity};
use lsp_types as lsp;
use url::Url;

fn location_to_position(line_str: &str, location: Location) -> Result<lsp::Position, ()> {
    if location.column.to_usize() > line_str.len() {
        Err(())
    } else if !line_str.is_char_boundary(location.column.to_usize()) {
        Err(())
    } else {
        let line_utf16 = line_str[..location.column.to_usize()].encode_utf16();
        let character = line_utf16.count() as u64;
        let line = location.line.to_usize() as u64;

        Ok(lsp::Position { line, character })
    }
}

pub fn byte_index_to_position(
    files: &Files,
    file_id: FileId,
    byte_index: ByteIndex,
) -> Result<lsp::Position, ()> {
    let location = files.location(file_id, byte_index).unwrap();
    let line_span = files.line_span(file_id, location.line).unwrap();
    let line_str = files.source(file_id, line_span).unwrap();

    location_to_position(&line_str, location)
}

pub fn byte_span_to_range(files: &Files, file_id: FileId, span: Span) -> Result<lsp::Range, ()> {
    Ok(lsp::Range {
        start: byte_index_to_position(files, file_id, span.start())?,
        end: byte_index_to_position(files, file_id, span.end())?,
    })
}

pub fn character_to_line_offset(line: &str, character: u64) -> ByteSize {
    let mut line_offset = ByteSize::from(0);

    for (i, ch) in line.chars().enumerate() {
        if character == i as u64 {
            break;
        }
        line_offset += ByteSize::from_char_len_utf8(ch);
    }

    line_offset
}

pub fn position_to_byte_index(
    files: &Files,
    file_id: FileId,
    position: &lsp::Position,
) -> ByteIndex {
    let line_span = files.line_span(file_id, position.line as u32).unwrap();
    let source = files.source(file_id, line_span).unwrap();
    let byte_offset = character_to_line_offset(source, position.character);

    line_span.start() + byte_offset
}

pub fn range_to_byte_span(files: &Files, file_id: FileId, range: &lsp::Range) -> Span {
    Span::new(
        position_to_byte_index(files, file_id, &range.start),
        position_to_byte_index(files, file_id, &range.end),
    )
}

pub fn make_lsp_severity(severity: Severity) -> lsp::DiagnosticSeverity {
    match severity {
        Severity::Error | Severity::Bug => lsp::DiagnosticSeverity::Error,
        Severity::Warning => lsp::DiagnosticSeverity::Warning,
        Severity::Note => lsp::DiagnosticSeverity::Information,
        Severity::Help => lsp::DiagnosticSeverity::Hint,
    }
}

/// Translates a `codespan_reporting::Diagnostic` to a `languageserver_types::Diagnostic`.
///
/// Since the language client requires `Url`s to locate the errors `codespan_name_to_file` is
/// necessary to resolve codespan `FileName`s
///
/// `code` and `file` are left empty by this function
pub fn make_lsp_diagnostic(
    files: &Files,
    diagnostic: Diagnostic,
    mut codespan_name_to_file: impl FnMut(&str) -> Result<Url, ()>,
) -> Result<lsp::Diagnostic, ()> {
    // We need a position for the primary error so take the span from the first primary label
    let primary_file_id = diagnostic.primary_label.file_span.id;
    let primary_span = diagnostic.primary_label.file_span.span;
    let primary_label_range = byte_span_to_range(files, primary_file_id, primary_span)?;

    let related_information = diagnostic
        .secondary_labels
        .into_iter()
        .map(|label| {
            let file_id = label.file_span.id;
            let range = byte_span_to_range(files, file_id, label.file_span.span)?;
            let uri = codespan_name_to_file(files.name(file_id))?;

            Ok(lsp::DiagnosticRelatedInformation {
                location: lsp::Location { uri, range },
                message: label.message,
            })
        })
        .collect::<Result<Vec<_>, ()>>()?;

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
        let mut files = Files::new();
        let file_id = files.add("", text);
        let pos = position_to_byte_index(
            &files,
            file_id,
            &lsp::Position {
                line: 3,
                character: 2,
            },
        );
        assert_eq!(Location::new(3, 2), files.location(file_id, pos).unwrap());
    }

    // The protocol specifies that each `character` in position is a UTF-16 character.
    // This means that `å` and `ä` here counts as 1 while `𐐀` counts as 2.
    const UNICODE: &str = "åä t𐐀b";

    #[test]
    fn unicode_get_byte_index() {
        let mut files = Files::new();
        let file_id = files.add("", UNICODE);

        let result = position_to_byte_index(
            &files,
            file_id,
            &lsp::Position {
                line: 0,
                character: 3,
            },
        );
        assert_eq!(result, ByteIndex::from(6));

        let result = position_to_byte_index(
            &files,
            file_id,
            &lsp::Position {
                line: 0,
                character: 6,
            },
        );
        assert_eq!(result, ByteIndex::from(11));
    }

    #[test]
    fn unicode_get_position() {
        let mut files = Files::new();
        let file_id = files.add("", UNICODE);

        let result = byte_index_to_position(&files, file_id, ByteIndex::from(6));
        assert_eq!(
            result,
            Ok(lsp::Position {
                line: 0,
                character: 3,
            })
        );

        let result = byte_index_to_position(&files, file_id, ByteIndex::from(11));
        assert_eq!(
            result,
            Ok(lsp::Position {
                line: 0,
                character: 6,
            })
        );
    }
}
