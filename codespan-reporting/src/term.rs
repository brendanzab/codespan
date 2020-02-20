//! Terminal back-end for emitting diagnostics.

use std::io;
use std::str::FromStr;
use termcolor::{ColorChoice, WriteColor};

use crate::diagnostic::{Diagnostic, Files};

mod config;
mod views;

pub use termcolor;

pub use self::config::{Chars, Config, DisplayStyle, Styles};

/// Emit a diagnostic using the given writer, context, config, and files.
pub fn emit<F: Files>(
    writer: &mut (impl WriteColor + ?Sized),
    config: &Config,
    files: &F,
    diagnostic: &Diagnostic<F::FileId>,
) -> io::Result<()> {
    use self::views::{RichDiagnostic, ShortDiagnostic};

    match config.display_style {
        DisplayStyle::Rich => RichDiagnostic::new(diagnostic).emit(files, writer, config),
        DisplayStyle::Short => ShortDiagnostic::new(diagnostic).emit(files, writer, config),
    }
}

/// A command line argument that configures the coloring of the output.
///
/// This can be used with command line argument parsers like `clap` or `structopt`.
///
/// # Example
///
/// ```rust
/// use structopt::StructOpt;
/// use codespan_reporting::term::termcolor::StandardStream;
/// use codespan_reporting::term::ColorArg;
///
/// #[derive(Debug, StructOpt)]
/// #[structopt(name = "groovey-app")]
/// pub struct Opts {
///     /// Configure coloring of output
///     #[structopt(
///         long = "color",
///         parse(try_from_str),
///         default_value = "auto",
///         possible_values = ColorArg::VARIANTS,
///         case_insensitive = true
///     )]
///     pub color: ColorArg,
/// }
///
/// fn main() {
///     let opts = Opts::from_args();
///     let writer = StandardStream::stderr(opts.color.into());
/// }
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ColorArg(pub ColorChoice);

impl ColorArg {
    /// Allowed values the argument.
    ///
    /// This is useful for generating documentation via `clap` or `structopt`'s
    /// `possible_values` configuration.
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

impl Into<ColorChoice> for ColorArg {
    fn into(self) -> ColorChoice {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::diagnostic::Label;

    #[test]
    fn unsized_emit() {
        let mut files = codespan::Files::new();

        let id = files.add("test", "");
        emit(
            &mut termcolor::NoColor::new(Vec::<u8>::new()) as &mut dyn WriteColor,
            &Config::default(),
            &files,
            &Diagnostic::new_bug("", Label::new(id, 0..0, "")),
        )
        .unwrap();
    }
}
