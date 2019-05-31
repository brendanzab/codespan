use structopt::StructOpt;

use codespan::{Files, Span};
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
    let mut files = Files::new();

    let source = r##"
(define test 123)
() (+ test
      "" 2
      3) ()
()
"##;
    let file = files.add_file("test".into(), source.to_string());

    let str_start = file.byte_index(3.into(), 6.into()).unwrap();
    let error = Diagnostic::new_error(
        "Unexpected type in `+` application",
        Label::new(
            Span::from_offset(str_start, 2.into()),
            "Expected integer but got string",
        ),
    )
    .with_code("E0001")
    .with_secondary_labels(vec![Label::new(
        Span::from_offset(str_start, 2.into()),
        "Expected integer but got string",
    )]);

    let line_start = file.byte_index(2.into(), 3.into()).unwrap();
    let warning = Diagnostic::new_warning(
        "`+` function has no effect unless its result is used",
        Label::new(Span::from_offset(line_start, 27.into()), "Value discarded"),
    );

    let diagnostics = [error, warning];

    let writer = StandardStream::stderr(opts.color.into());
    let config = codespan_reporting::Config::default();
    for diagnostic in &diagnostics {
        emit(&mut writer.lock(), &config, &files, &diagnostic).unwrap();
        println!();
    }
}
