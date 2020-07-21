//! Terminal back-end for emitting diagnostics.

use std::io;
use std::str::FromStr;
use termcolor::{ColorChoice, WriteColor};

use crate::diagnostic::Diagnostic;
use crate::files::{Files, Src, RefSrc};

mod config;
mod renderer;
mod views;

pub use termcolor;

pub use self::config::{Chars, Config, DisplayStyle, Styles};

/// A command line argument that configures the coloring of the output.
///
/// This can be used with command line argument parsers like [`clap`] or [`structopt`].
///
/// [`clap`]: https://crates.io/crates/clap
/// [`structopt`]: https://crates.io/crates/structopt
///
/// # Example
///
/// ```rust
/// use codespan_reporting::term::termcolor::StandardStream;
/// use codespan_reporting::term::ColorArg;
/// use structopt::StructOpt;
///
/// #[derive(Debug, StructOpt)]
/// #[structopt(name = "groovey-app")]
/// pub struct Opts {
///     /// Configure coloring of output
///     #[structopt(
///         long = "color",
///         default_value = "auto",
///         possible_values = ColorArg::VARIANTS,
///         case_insensitive = true,
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
    /// This is useful for generating documentation via [`clap`] or `structopt`'s
    /// `possible_values` configuration.
    ///
    /// [`clap`]: https://crates.io/crates/clap
    /// [`structopt`]: https://crates.io/crates/structopt
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

/// Emit a diagnostic using the given writer, context, config, and files.
pub fn emit<'files, FileId: Copy + PartialEq, S: Src, F: FnMut(FileId) -> S>(
    writer: &mut dyn WriteColor,
    config: &Config,
    fetch_file: F,
    diagnostic: &Diagnostic<FileId>,
) -> io::Result<()> {
    use self::renderer::Renderer;
    use self::views::{RichDiagnostic, ShortDiagnostic};

    let mut renderer = Renderer::new(writer, config);
    match config.display_style {
        DisplayStyle::Rich => RichDiagnostic::new(diagnostic).render(fetch_file, &mut renderer),
        DisplayStyle::Short => ShortDiagnostic::new(diagnostic).render(fetch_file, &mut renderer),
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

        emit(&mut writer, &Config::default(), |id| files.get(id).unwrap(), &diagnostic).unwrap();
    }
}
