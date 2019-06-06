use codespan::Files;
use std::io;
use termcolor::WriteColor;

use crate::emitter::Config;
use crate::Diagnostic;

use super::{Header, NewLine, SourceSnippet};

/// Output a richly formatted diagnostic, with source code previews.
pub struct RichDiagnostic<'a> {
    files: &'a Files,
    diagnostic: &'a Diagnostic,
}

impl<'a> RichDiagnostic<'a> {
    pub fn new(files: &'a Files, diagnostic: &'a Diagnostic) -> RichDiagnostic<'a> {
        RichDiagnostic { files, diagnostic }
    }

    pub fn emit(&self, writer: &mut impl WriteColor, config: &Config) -> io::Result<()> {
        Header::new(self.diagnostic).emit(writer, config)?;
        NewLine::new().emit(writer, config)?;

        SourceSnippet::new_primary(self.files, &self.diagnostic).emit(writer, config)?;
        NewLine::new().emit(writer, config)?;

        for label in &self.diagnostic.secondary_labels {
            SourceSnippet::new_secondary(self.files, &label).emit(writer, config)?;
            NewLine::new().emit(writer, config)?;
        }

        Ok(())
    }
}
