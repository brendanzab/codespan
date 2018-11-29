extern crate codespan;
extern crate codespan_reporting;
#[macro_use]
extern crate structopt;

use structopt::StructOpt;

use codespan::{ByteOffset, CodeMap, ColumnIndex, LineIndex, Span};
use codespan_reporting::termcolor::StandardStream;
use codespan_reporting::{emit, ColorArg, Diagnostic, Label};

#[derive(Debug, StructOpt)]
#[structopt(name = "emit")]
pub struct Opts {
    /// Configure coloring of output
    #[structopt(
        long = "color",
        parse(try_from_str),
        default_value = "auto",
        raw(possible_values = "ColorArg::VARIANTS", case_insensitive = "true")
    )]
    pub color: ColorArg,
}

fn main() {
    let opts = Opts::from_args();
    let mut code_map = CodeMap::new();

    let source = r##"
(define test 123)
(+ test "")
()

ééééé by x by "héhé"

"##;
    let file_map = code_map.add_filemap("test".into(), source.to_string());

    let str_start = file_map.byte_index(LineIndex(2), ColumnIndex(8)).unwrap();
    let str_span = Span::from_offset(str_start, ByteOffset(2));
    let plus_error = Diagnostic::new_error("Unexpected type in `+` application")
        .with_label(Label::new_primary(str_span).with_message("Expected integer but got string"))
        .with_label(Label::new_secondary(str_span).with_message("Expected integer but got string"))
        .with_code("E0001");

    let plus_call_start = file_map.byte_index(LineIndex(2), ColumnIndex(0)).unwrap();
    let plus_call_span = Span::from_offset(plus_call_start, ByteOffset(1));
    let call_warning =
        Diagnostic::new_warning("`+` function has no effect unless its result is used")
            .with_label(Label::new_primary(plus_call_span));

    let initial_start = file_map.byte_index(LineIndex(5), ColumnIndex(6)).unwrap();
    let initial_span = Span::from_offset(initial_start, ByteOffset(4));
    let duplicate_start = file_map.byte_index(LineIndex(5), ColumnIndex(14)).unwrap();
    let duplicate_span = Span::from_offset(duplicate_start, ByteOffset(6));
    let unicode_error = Diagnostic::new_error("duplicate clause")
        .with_label(Label::new_primary(duplicate_span).with_message("duplicate clause"))
        .with_label(Label::new_secondary(initial_span).with_message("initial clause"));

    let diagnostics = [plus_error, call_warning, unicode_error];

    let writer = StandardStream::stderr(opts.color.into());
    for diagnostic in &diagnostics {
        emit(&mut writer.lock(), &code_map, &diagnostic).unwrap();
        println!();
    }
}
