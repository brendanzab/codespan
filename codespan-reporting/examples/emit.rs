use structopt::StructOpt;

use codespan::{FileSpan, Files, Span};
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
        "unexpected type in `+` application",
        Label::new(
            FileSpan::new(file_id, Span::new(36, 38)),
            // Use the unindent crate to format this string as:
            //
            // ```
            // "expected: `Int`\n   found: `String`"
            // ```
            unindent::unindent("
                expected: `Int`
                   found: `String`
            "),
        ),
    )
    .with_code("E0001")
    .with_secondary_labels(vec![Label::new(
        FileSpan::new(file_id, Span::new(36, 38)),
        unindent::unindent("
            expected: `Int`
               found: `String`
        "),
    )]);

    let warning = Diagnostic::new_warning(
        "`+` function has no effect unless its result is used",
        Label::new(FileSpan::new(file_id, Span::new(22, 49)), "value discarded"),
    );

    let diagnostics = [error, warning];

    let writer = StandardStream::stderr(opts.color.into());
    let config = codespan_reporting::Config::default();
    for diagnostic in &diagnostics {
        emit(&mut writer.lock(), &config, &files, &diagnostic).unwrap();
        println!();
    }
}
