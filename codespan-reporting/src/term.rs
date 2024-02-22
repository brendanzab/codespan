//! Terminal back-end for emitting diagnostics.

use std::str::FromStr;
use termcolor::{ColorChoice, WriteColor};

use crate::diagnostic::Diagnostic;
use crate::files::Files;

mod config;
mod renderer;
mod views;

pub use termcolor;

pub use self::config::{Chars, Config, DisplayStyle, Styles};

/// A command line argument that configures the coloring of the output.
///
/// This can be used with command line argument parsers like [`clap`] or [`Parser`].
///
/// [`clap`]: https://crates.io/crates/clap
/// [`Parser`]: https://crates.io/crates/Parser
///
/// # Example
///
/// ```rust
/// use clap::{builder::TypedValueParser as _, Parser};
/// use codespan_reporting::term::ColorArg;
/// use codespan_reporting::term::termcolor::StandardStream;
/// use std::str::FromStr as _;
///
/// #[derive(Debug, Parser)]
/// #[clap(name = "groovey-app")]
/// pub struct Opts {
///     /// Configure coloring of output
///     #[clap(
///         long = "color",
///         default_value = "auto",
///         value_parser = clap::builder::PossibleValuesParser::new(ColorArg::VARIANTS)
///             .map(|s| ColorArg::from_str(&s)).map(Result::unwrap),
///     )]
///     pub color: ColorArg,
/// }
///
/// let opts = Opts::parse();
/// let writer = StandardStream::stderr(opts.color.into());
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ColorArg(pub ColorChoice);

impl ColorArg {
    /// Allowed values the argument.
    ///
    /// This is useful for generating documentation via [`clap`]'s
    /// `value_parser = clap::builder::PossibleValuesParser::new` configuration.
    ///
    /// [`clap`]: https://crates.io/crates/clap
    pub const VARIANTS: &'static [&'static str] = &["auto", "always", "ansi", "never"];
}

impl FromStr for ColorArg {
    type Err = &'static str;

    fn from_str(src: &str) -> Result<ColorArg, &'static str> {
        match src {
            _ if src.eq_ignore_ascii_case("auto") => Ok(ColorArg(ColorChoice::Auto)),
            _ if src.eq_ignore_ascii_case("always") => Ok(ColorArg(ColorChoice::Always)),
            _ if src.eq_ignore_ascii_case("ansi") => Ok(ColorArg(ColorChoice::AlwaysAnsi)),
            _ if src.eq_ignore_ascii_case("never") => Ok(ColorArg(ColorChoice::Never)),
            _ => Err("valid values: auto, always, ansi, never"),
        }
    }
}

impl From<ColorArg> for ColorChoice {
    fn from(x: ColorArg) -> Self {
        x.0
    }
}

/// Emit a diagnostic using the given writer, context, config, and files.
///
/// The return value covers all error cases. These error case can arise if:
/// * a file was removed from the file database.
/// * a file was changed so that it is too small to have an index
/// * IO fails
pub fn emit<'files, F: Files<'files>>(
    writer: &mut dyn WriteColor,
    config: &Config,
    files: &'files F,
    diagnostic: &Diagnostic<F::FileId>,
) -> Result<(), super::files::Error> {
    use self::renderer::Renderer;
    use self::views::{RichDiagnostic, ShortDiagnostic};

    let mut renderer = Renderer::new(writer, config);
    match config.display_style {
        DisplayStyle::Rich => RichDiagnostic::new(diagnostic, config).render(files, &mut renderer),
        DisplayStyle::Medium => ShortDiagnostic::new(diagnostic, true).render(files, &mut renderer),
        DisplayStyle::Short => ShortDiagnostic::new(diagnostic, false).render(files, &mut renderer),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::diagnostic::Label;
    use crate::files::SimpleFiles;

    #[test]
    fn unsized_emit() {
        let mut files = SimpleFiles::new();

        let id = files.add("test", "");
        let mut writer = termcolor::NoColor::new(Vec::<u8>::new());
        let diagnostic = Diagnostic::bug().with_labels(vec![Label::primary(id, 0..0)]);

        emit(&mut writer, &Config::default(), &files, &diagnostic).unwrap();
    }
}
