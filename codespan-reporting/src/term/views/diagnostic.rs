use codespan::Files;
use std::io;
use termcolor::WriteColor;

use crate::diagnostic::Diagnostic;
use crate::term::Config;

use super::{Header, Locus, NewLine, SourceSnippet};

/// Output a richly formatted diagnostic, with source code previews.
pub struct RichDiagnostic<'a> {
    diagnostic: &'a Diagnostic,
}

impl<'a> RichDiagnostic<'a> {
    pub fn new(diagnostic: &'a Diagnostic) -> Self {
        RichDiagnostic { diagnostic }
    }

    pub fn emit(
        &self,
        files: &'a Files<impl AsRef<str>>,
        writer: &mut (impl WriteColor + ?Sized),
        config: &Config,
    ) -> io::Result<()> {
        use std::collections::BTreeMap;

        use super::MarkStyle;

        Header::new(self.diagnostic).emit(writer, config)?;
        NewLine::new().emit(writer, config)?;

        let primary_label = &self.diagnostic.primary_label;
        let primary_file_id = self.diagnostic.primary_label.file_id;
        let severity = self.diagnostic.severity;
        let notes = &self.diagnostic.notes;

        // Group labels by file

        let mut label_groups = BTreeMap::new();

        label_groups
            .entry(primary_file_id)
            .or_insert(vec![])
            .push((primary_label, MarkStyle::Primary(severity)));

        for secondary_label in &self.diagnostic.secondary_labels {
            label_groups
                .entry(secondary_label.file_id)
                .or_insert(vec![])
                .push((secondary_label, MarkStyle::Secondary));
        }

        // Emit the snippets, starting with the one that contains the primary label

        let labels = label_groups.remove(&primary_file_id).unwrap_or(vec![]);
        SourceSnippet::new(primary_file_id, labels, notes).emit(files, writer, config)?;
        NewLine::new().emit(writer, config)?;

        for (file_id, labels) in label_groups {
            SourceSnippet::new(file_id, labels, &[]).emit(files, writer, config)?;
            NewLine::new().emit(writer, config)?;
        }

        Ok(())
    }
}

/// Output a short diagnostic, with a line number, severity, and message.
pub struct ShortDiagnostic<'a> {
    diagnostic: &'a Diagnostic,
}

impl<'a> ShortDiagnostic<'a> {
    pub fn new(diagnostic: &'a Diagnostic) -> ShortDiagnostic<'a> {
        ShortDiagnostic { diagnostic }
    }

    pub fn emit(
        &self,
        files: &'a Files<impl AsRef<str>>,
        writer: &mut (impl WriteColor + ?Sized),
        config: &Config,
    ) -> io::Result<()> {
        let label = &self.diagnostic.primary_label;
        let location = files
            .location(label.file_id, label.span.start())
            .expect("location");
        Locus::new(files.name(self.diagnostic.primary_label.file_id), location)
            .emit(writer, config)?;
        write!(writer, ": ")?;
        Header::new(self.diagnostic).emit(writer, config)?;

        Ok(())
    }
}
