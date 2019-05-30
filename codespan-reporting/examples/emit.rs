use structopt::StructOpt;

use codespan::{CodeMap, Span};
use codespan_reporting::termcolor::StandardStream;
use codespan_reporting::{emit, ColorArg, Diagnostic, Label, Severity};

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
() (+ test
      "" 2
      3) ()
()
"##;
    let file_map = code_map.add_filemap("test".into(), source.to_string());

    let str_start = file_map.byte_index(3.into(), 6.into()).unwrap();
    let error = Diagnostic::new(Severity::Error, "Unexpected type in `+` application")
        .with_label(
            Label::new_primary(Span::from_offset(str_start, 2.into()))
                .with_message("Expected integer but got string"),
        )
        .with_label(
            Label::new_secondary(Span::from_offset(str_start, 2.into()))
                .with_message("Expected integer but got string"),
        )
        .with_code("E0001");

    let line_start = file_map.byte_index(2.into(), 3.into()).unwrap();
    let warning = Diagnostic::new(
        Severity::Warning,
        "`+` function has no effect unless its result is used",
    )
    .with_label(
        Label::new_primary(Span::from_offset(line_start, 27.into()))
            .with_message("Value discarded"),
    );

    let diagnostics = [error, warning];

    let writer = StandardStream::stderr(opts.color.into());
    let config = codespan_reporting::Config::default();
    for diagnostic in &diagnostics {
        emit(&mut writer.lock(), &config, &code_map, &diagnostic).unwrap();
        println!();
    }
}
