//! Terminal back-end for emitting diagnostics.
//!
//! Choose an emitter based on the output destination:
//!
//! - [`emit_to_write_style`] writes with colors and styles to a [`WriteStyle`] writer.
//!   With the `termcolor` feature, this includes `termcolor::WriteColor` writers.
//! - `emit_to_io_write` writes plain text to a `std::io::Write` writer and requires
//!   the `std` feature.
//! - [`emit_to_string`] appends plain text to an existing [`String`], while
//!   [`emit_into_string`] returns a new string. Both support `no_std` with `alloc`.
//!
//! These functions replace the deprecated [`emit`] function.
//!
//! # Example
//!
//! ```
//! use codespan_reporting::diagnostic::{Diagnostic, Label};
//! use codespan_reporting::files::SimpleFile;
//! use codespan_reporting::term::{self, Config};
//!
//! let file = SimpleFile::new("example", "let x = ;");
//! let diagnostic = Diagnostic::error()
//!     .with_message("expected an expression")
//!     .with_label(Label::primary((), 8..9));
//! let config = Config::default();
//!
//! let rendered = term::emit_into_string(&config, &file, &diagnostic)?;
//! assert!(rendered.contains("expected an expression"));
//!
//! let mut output = String::from("Compiler output:\n");
//! term::emit_to_string(&mut output, &config, &file, &diagnostic)?;
//! assert_eq!(output, format!("Compiler output:\n{rendered}"));
//!
//! # #[cfg(feature = "std")]
//! # {
//! let mut bytes = Vec::new();
//! term::emit_to_io_write(&mut bytes, &config, &file, &diagnostic)?;
//! assert_eq!(bytes, rendered.as_bytes());
//! # }
//!
//! # #[cfg(feature = "termcolor")]
//! # {
//! let mut colored = term::termcolor::Buffer::ansi();
//! term::emit_to_write_style(&mut colored, &config, &file, &diagnostic)?;
//! assert!(colored.as_slice().windows(2).any(|bytes| bytes == b"\x1b["));
//! # }
//! # Ok::<(), codespan_reporting::files::Error>(())
//! ```

use crate::diagnostic::Diagnostic;
use crate::files::Files;

mod config;
mod renderer;
mod views;

#[cfg(not(feature = "std"))]
use alloc::string::String;

pub use config::{Chars, Config, DisplayStyle};

// re-export
#[cfg(feature = "termcolor")]
pub use config::styles::{termcolor, Styles, StylesWriter};

pub use renderer::{GeneralWrite, GeneralWriteResult, Renderer, WriteStyle};
pub use views::{RichDiagnostic, ShortDiagnostic};

/// Renders a diagnostic as plain text in a new string.
///
/// Returns an error if the file database cannot resolve a file or source location.
pub fn emit_into_string<'files, F: Files<'files> + ?Sized>(
    config: &Config,
    files: &'files F,
    diagnostic: &Diagnostic<F::FileId>,
) -> Result<String, super::files::Error> {
    let mut writer = String::new();
    emit_to_string(&mut writer, config, files, diagnostic)?;
    Ok(writer)
}

/// Appends a diagnostic as plain text to an existing string.
///
/// This function supports `no_std` with `alloc` and does not clear the string first.
/// Returns an error if the file database cannot resolve a file or source location.
pub fn emit_to_string<'files, F: Files<'files> + ?Sized>(
    writer: &mut String,
    config: &Config,
    files: &'files F,
    diagnostic: &Diagnostic<F::FileId>,
) -> Result<(), super::files::Error> {
    // std::io::Write used under `feature = "std"`
    #[cfg(feature = "std")]
    {
        let mut buffer = Vec::new();
        emit_with_style(
            renderer::PlainWriter::new(&mut buffer),
            config,
            files,
            diagnostic,
        )?;
        let buffer_str: &str = core::str::from_utf8(&buffer).expect("buffer not utf8");
        writer.push_str(buffer_str);
        Ok(())
    }

    // core::fmt::Write used not `not(feature = "std")`
    #[cfg(not(feature = "std"))]
    {
        emit_with_style(
            renderer::PlainWriter::new(writer),
            config,
            files,
            diagnostic,
        )
    }
}

/// Writes a diagnostic as plain text to an I/O writer.
///
/// Requires the `std` feature. To preserve colors and styles, use
/// [`emit_to_write_style`] instead.
/// Returns an error if writing fails or the file database cannot resolve a file
/// or source location.
#[cfg(feature = "std")]
pub fn emit_to_io_write<'files, F: Files<'files> + ?Sized, W: std::io::Write + ?Sized>(
    writer: &mut W,
    config: &Config,
    files: &'files F,
    diagnostic: &Diagnostic<F::FileId>,
) -> Result<(), super::files::Error> {
    emit_with_style(
        renderer::PlainWriter::new(writer),
        config,
        files,
        diagnostic,
    )
}

/// Writes a diagnostic with colors and styles to a [`WriteStyle`] writer.
///
/// With the `termcolor` feature, this accepts `termcolor::WriteColor` writers.
/// Use `StylesWriter` to customize their styles.
/// Returns an error if writing fails or the file database cannot resolve a file
/// or source location.
pub fn emit_to_write_style<'files, F: Files<'files> + ?Sized, W: WriteStyle + ?Sized>(
    writer: &mut W,
    config: &Config,
    files: &'files F,
    diagnostic: &Diagnostic<F::FileId>,
) -> Result<(), super::files::Error> {
    emit_with_style(
        renderer::WriteStyleByRef::new(writer),
        config,
        files,
        diagnostic,
    )
}

#[deprecated(
    since = "0.13.0",
    note = "Use `emit_to_write_style` instead or depending on the writer use `emit_to_io_write` or `emit_to_string`"
)]
/// Emit a diagnostic using the given writer, context, config, and files.
///
/// The return value covers all error cases. These error case can arise if:
/// * a file was removed from the file database.
/// * a file was changed so that it is too small to have an index
/// * IO fails
pub fn emit<'files, F: Files<'files> + ?Sized, W: renderer::GeneralWrite + ?Sized>(
    writer: &mut W,
    config: &Config,
    files: &'files F,
    diagnostic: &Diagnostic<F::FileId>,
) -> Result<(), super::files::Error> {
    emit_with_style(
        renderer::PlainWriter::new(writer),
        config,
        files,
        diagnostic,
    )
}

fn emit_with_style<'files, F: Files<'files> + ?Sized, W: WriteStyle>(
    mut writer: W,
    config: &Config,
    files: &'files F,
    diagnostic: &Diagnostic<F::FileId>,
) -> Result<(), super::files::Error> {
    let mut renderer = Renderer::new(&mut writer, config);
    match config.display_style {
        DisplayStyle::Rich => RichDiagnostic::new(diagnostic, config).render(files, &mut renderer),
        DisplayStyle::Medium => ShortDiagnostic::new(diagnostic, true).render(files, &mut renderer),
        DisplayStyle::Short => ShortDiagnostic::new(diagnostic, false).render(files, &mut renderer),
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use super::*;

    use crate::diagnostic::Label;
    use crate::files::SimpleFiles;

    /// Test range of 0 to 0 and check does not crash
    #[test]
    fn unsized_emit() {
        let mut files = SimpleFiles::new();

        let id = files.add("test", "");
        let zero_range = 0..0;
        let diagnostic = Diagnostic::bug().with_label(Label::primary(id, zero_range));
        emit_into_string(&Config::default(), &files, &diagnostic).unwrap();
    }
}
