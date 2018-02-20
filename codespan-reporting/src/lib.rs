extern crate codespan;
extern crate termcolor;

use std::fmt;
use termcolor::Color;

mod diagnostic;
mod emitter;

pub use self::diagnostic::{Diagnostic, Label, LabelStyle};
pub use self::emitter::emit;

/// A severity level for diagnostic messages
#[derive(Copy, PartialEq, Clone, Hash, Debug)]
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
