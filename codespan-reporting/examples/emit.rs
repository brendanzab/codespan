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

    let file_id = files.add("test", source.to_string());

    let error = Diagnostic::new_error(
        "Unexpected type in `+` application",
        Label::new(
            file_id,
            Span::new(36, 38),
            "Expected integer but got string",
        ),
    )
    .with_code("E0001")
    .with_secondary_labels(vec![Label::new(
        file_id,
        Span::new(36, 38),
        "Expected integer but got string",
    )]);

    let warning = Diagnostic::new_warning(
        "`+` function has no effect unless its result is used",
        Label::new(file_id, Span::new(22, 49), "Value discarded"),
    );

    let diagnostics = [error, warning];

    let writer = StandardStream::stderr(opts.color.into());
    let config = codespan_reporting::Config::default();
    for diagnostic in &diagnostics {
        emit(&mut writer.lock(), &config, &files, &diagnostic).unwrap();
        println!();
    }
}
