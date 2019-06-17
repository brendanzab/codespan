//! Diagnostic reporting support for the codespan crate.

mod diagnostic;
mod emitter;

pub use termcolor;

pub use self::diagnostic::{Diagnostic, Label, Severity};
pub use self::emitter::{emit, ColorArg, Config, DisplayStyle};
