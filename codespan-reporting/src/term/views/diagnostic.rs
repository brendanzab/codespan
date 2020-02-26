use std::io;
use termcolor::WriteColor;

use crate::diagnostic::Diagnostic;
use crate::files::Files;
use crate::term::Config;

use super::{Header, Locus, NewLine, SourceSnippet};

/// Output a richly formatted diagnostic, with source code previews.
pub struct RichDiagnostic<'a, FileId> {
    diagnostic: &'a Diagnostic<FileId>,
}

impl<'a, FileId> RichDiagnostic<'a, FileId>
where
    FileId: Copy + PartialEq + PartialOrd + Eq + Ord + std::hash::Hash,
{
    pub fn new(diagnostic: &'a Diagnostic<FileId>) -> RichDiagnostic<'a, FileId> {
        RichDiagnostic { diagnostic }
    }

    pub fn emit<'files>(
        &self,
        files: &'files impl Files<'files, FileId = FileId>,
        writer: &mut (impl WriteColor + ?Sized),
        config: &Config,
    ) -> io::Result<()>
    where
        FileId: 'files,
    {
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
pub struct ShortDiagnostic<'a, FileId> {
    diagnostic: &'a Diagnostic<FileId>,
}

impl<'a, FileId> ShortDiagnostic<'a, FileId>
where
    FileId: Copy + PartialEq + PartialOrd + Eq + Ord + std::hash::Hash,
{
    pub fn new(diagnostic: &'a Diagnostic<FileId>) -> ShortDiagnostic<'a, FileId> {
        ShortDiagnostic { diagnostic }
    }

    pub fn emit<'files>(
        &self,
        files: &'files impl Files<'files, FileId = FileId>,
        writer: &mut (impl WriteColor + ?Sized),
        config: &Config,
    ) -> io::Result<()>
    where
        FileId: 'files,
    {
        let label = &self.diagnostic.primary_label;
        let start = label.range.start;

        let origin = files.origin(label.file_id).expect("origin");
        let line_index = files.line_index(label.file_id, start).expect("line_index");
        let line = files.line(label.file_id, line_index).expect("line");

        Locus::new(origin, line.number, line.column_number(start)).emit(writer, config)?;
        write!(writer, ": ")?;
        Header::new(self.diagnostic).emit(writer, config)?;

        Ok(())
    }
}
