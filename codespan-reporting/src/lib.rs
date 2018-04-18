extern crate codespan;
pub extern crate termcolor;

use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;
use termcolor::{Color, ColorChoice};

mod diagnostic;
mod emitter;

pub use self::diagnostic::{Diagnostic, Label, LabelStyle};
pub use self::emitter::emit;

/// A severity level for diagnostic messages
///
/// These are ordered in the following way:
///
/// ```rust
/// use codespan_reporting::Severity;
///
/// assert!(Severity::Bug > Severity::Error);
/// assert!(Severity::Error > Severity::Warning);
/// assert!(Severity::Warning > Severity::Note);
/// assert!(Severity::Note > Severity::Help);
/// ```
#[derive(Copy, Clone, PartialEq, Hash, Debug)]
pub enum Severity {
    /// An unexpected bug.
    Bug,
    /// An error.
    Error,
    /// A warning.
    Warning,
    /// A note.
    Note,
    /// A help message.
    Help,
}

impl Severity {
    /// We want bugs to be the maximum severity, errors next, etc...
    fn to_cmp_int(self) -> u8 {
        match self {
            Severity::Bug => 5,
            Severity::Error => 4,
            Severity::Warning => 3,
            Severity::Note => 2,
            Severity::Help => 1,
        }
    }
}

impl PartialOrd for Severity {
    fn partial_cmp(&self, other: &Severity) -> Option<Ordering> {
        u8::partial_cmp(&self.to_cmp_int(), &other.to_cmp_int())
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.to_str().fmt(f)
    }
}

impl Severity {
    /// Return the termcolor to use when rendering messages of this diagnostic severity
    pub fn color(self) -> Color {
        match self {
            Severity::Bug | Severity::Error => Color::Red,
            Severity::Warning => Color::Yellow,
            Severity::Note => Color::Green,
            Severity::Help => Color::Cyan,
        }
    }

    /// A string that explains this diagnostic severity
    pub fn to_str(self) -> &'static str {
        match self {
            Severity::Bug => "error: internal compiler error",
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Note => "note",
            Severity::Help => "help",
        }
    }
}

/// A command line argument that configures the coloring of the output
///
/// This can be used with command line argument parsers like `clap` or `structopt`.
///
/// # Example
///
/// ```rust
/// extern crate codespan_reporting;
/// #[macro_use]
/// extern crate structopt;
///
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
    /// Allowed values the argument
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
