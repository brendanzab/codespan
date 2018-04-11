extern crate codespan;
extern crate codespan_reporting;

use codespan::CodeMap;
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

    let diagnostic = Diagnostic::new(Severity::Error, "Unexpected type in `+` application")
        .with_label(
            Label::new_primary(file_map.line_span(2.into()).unwrap())
                .with_message("Expected integer but got string"),
        );
    emit(&mut writer.lock(), &code_map, &diagnostic).unwrap();
}
