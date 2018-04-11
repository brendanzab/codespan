extern crate codespan;
extern crate codespan_reporting;

use codespan::{CodeMap, Span};
use codespan_reporting::termcolor::{ColorChoice, StandardStream};
use codespan_reporting::{emit, Diagnostic, Label, Severity};

fn main() {
    let writer = StandardStream::stdout(ColorChoice::Auto);

    let mut code_map = CodeMap::new();

    let source = r##"
(define test 123)
(+ test "")
"##;
    let file_map = code_map.add_filemap("test".into(), source.to_string());

    let str_start = file_map.byte_index(2.into(), 8.into()).unwrap();
    let diagnostic = Diagnostic::new(Severity::Error, "Unexpected type in `+` application")
        .with_label(
            Label::new_primary(Span::from_offset(str_start, 2.into()))
                .with_message("Expected integer but got string"),
        );
    emit(&mut writer.lock(), &code_map, &diagnostic).unwrap();
}
