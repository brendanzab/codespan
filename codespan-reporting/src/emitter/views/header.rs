use std::io;
use termcolor::{ColorSpec, WriteColor};

use crate::emitter::Config;
use crate::{Diagnostic, Severity};

use super::NewLine;

/// Diagnostic header.
///
/// ```text
/// error[E0001]: unexpected type in `+` application
/// ```
#[derive(Copy, Clone, Debug)]
pub struct Header<'a> {
    severity: Severity,
    code: Option<&'a str>,
    message: &'a str,
}

impl<'a> Header<'a> {
    pub fn new(diagnostic: &'a Diagnostic) -> Header<'a> {
        Header {
            severity: diagnostic.severity,
            code: diagnostic.code.as_ref().map(String::as_str),
            message: &diagnostic.message,
        }
    }

    fn severity_name(&self) -> &'static str {
        match self.severity {
            Severity::Bug => "bug",
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Help => "help",
            Severity::Note => "note",
        }
    }

    pub fn emit(&self, writer: &mut impl WriteColor, config: &Config) -> io::Result<()> {
        let message_spec = ColorSpec::new().set_bold(true).set_intense(true).clone();
        let primary_spec = ColorSpec::new()
            .set_bold(true)
            .set_intense(true)
            .set_fg(Some(config.severity_color(self.severity)))
            .clone();

        // Write severity name
        //
        // ```text
        // error
        // ```
        writer.set_color(&primary_spec)?;
        write!(writer, "{}", self.severity_name())?;
        if let Some(code) = &self.code {
            // Write error code
            //
            // ```text
            // [E0001]
            // ```
            write!(writer, "[{}]", code)?;
        }

        // Write diagnostic message
        //
        // ```text
        // : unexpected type in `+` application
        // ```
        writer.set_color(&message_spec)?;
        write!(writer, ": {}", self.message)?;
        writer.reset()?;

        NewLine::new().emit(writer, config)?;

        Ok(())
    }
}
