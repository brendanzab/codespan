//! Diagnostic reporting support for the codespan crate.

use std::str::FromStr;
use termcolor::ColorChoice;

mod diagnostic;
mod emitter;

pub use termcolor;

pub use self::diagnostic::{Diagnostic, Label, Severity};
pub use self::emitter::{emit, Config};

/// A command line argument that configures the coloring of the output.
///
/// This can be used with command line argument parsers like `clap` or `structopt`.
///
/// # Example
///
/// ```rust
/// use structopt::StructOpt;
/// use codespan_reporting::termcolor::StandardStream;
/// use codespan_reporting::ColorArg;
///
/// #[derive(Debug, StructOpt)]
/// #[structopt(name = "groovey-app")]
/// pub struct Opts {
///     /// Configure coloring of output
///     #[structopt(
///         long = "color",
///         parse(try_from_str),
///         default_value = "auto",
///         raw(possible_values = "ColorArg::VARIANTS", case_insensitive = "true")
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
