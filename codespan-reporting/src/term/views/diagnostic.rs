use std::io;
use std::ops::Range;
use termcolor::WriteColor;

use crate::diagnostic::{Diagnostic, LabelStyle};
use crate::files::Files;
use crate::term::Config;

use super::{Header, Locus, Note};

/// Count the number of decimal digits in `n`.
fn count_digits(mut n: usize) -> usize {
    let mut count = 0;
    while n != 0 {
        count += 1;
        n /= 10; // remove last digit
    }
    count
}

/// Merge two ranges.
fn merge(range0: &Range<usize>, range1: &Range<usize>) -> Range<usize> {
    let start = std::cmp::min(range0.start, range1.start);
    let end = std::cmp::max(range0.end, range1.end);
    start..end
}

/// Output a richly formatted diagnostic, with source code previews.
pub struct RichDiagnostic<'a, FileId> {
    diagnostic: &'a Diagnostic<FileId>,
}

impl<'a, FileId> RichDiagnostic<'a, FileId>
where
    FileId: Copy + Ord,
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

        use super::{NewLine, SourceSnippet};

        // Group marks by file

        let mut mark_groups = BTreeMap::new();
        let mut gutter_padding = 0;

        for label in &self.diagnostic.labels {
            use std::collections::btree_map::Entry;

            use super::{Mark, MarkGroup, MarkStyle};

            let mark_style = match label.style {
                LabelStyle::Primary => MarkStyle::Primary(self.diagnostic.severity),
                LabelStyle::Secondary => MarkStyle::Secondary,
            };

            // Compute the width of the gutter for the following source snippets and notes
            let end_line = files
                .line_index(label.file_id, label.range.end)
                .and_then(|index| files.line(label.file_id, index))
                .expect("end_line");
            gutter_padding = std::cmp::max(gutter_padding, count_digits(end_line.number));

            let mark = Mark {
                style: mark_style,
                range: label.range.clone(),
                message: label.message.as_str(),
            };

            // TODO: Sort snippets by the mark group origin
            // TODO: Group contiguous line index ranges using some sort of interval set algorithm
            // TODO: Flatten mark groups to overlapping underlines that can be easily rendered.
            match mark_groups.entry(label.file_id) {
                Entry::Vacant(entry) => {
                    entry.insert(MarkGroup {
                        origin: files.origin(label.file_id).expect("origin"),
                        range: label.range.clone(),
                        marks: vec![mark],
                    });
                },
                Entry::Occupied(mut entry) => {
                    let mark_group = entry.get_mut();
                    mark_group.range = merge(&mark_group.range, &mark.range);
                    mark_group.marks.push(mark);
                },
            }
        }

        // Sort marks lexicographically by the range of source code they cover.
        for (_, mark_group) in mark_groups.iter_mut() {
            mark_group.marks.sort_by_key(|mark| {
                // `Range<usize>` doesn't implement `Ord`, so convert to `(usize, usize)`
                // to piggyback off its lexicographic sorting implementation.
                (mark.range.start, mark.range.end)
            });
        }

        // Emit the title
        //
        // ```text
        // error[E0001]: unexpected type in `+` application
        // ```
        Header::new(self.diagnostic).emit(writer, config)?;
        if !mark_groups.is_empty() {
            NewLine::new().emit(writer, config)?;
        }

        // Emit the source snippets
        //
        // ```text
        //   ┌── test:2:9 ───
        //   │
        // 2 │ (+ test "")
        //   │         ^^ expected `Int` but found `String`
        //   │
        // ```
        for (file_id, mark_group) in mark_groups {
            SourceSnippet::new(gutter_padding, file_id, mark_group).emit(files, writer, config)?;
        }

        // Additional notes
        //
        // ```text
        // = expected type `Int`
        //      found type `String`
        // ```
        for note in &self.diagnostic.notes {
            Note::new(gutter_padding, &note).emit(writer, config)?;
        }
        NewLine::new().emit(writer, config)?;

        Ok(())
    }
}

/// Output a short diagnostic, with a line number, severity, and message.
pub struct ShortDiagnostic<'a, FileId> {
    diagnostic: &'a Diagnostic<FileId>,
}

impl<'a, FileId> ShortDiagnostic<'a, FileId>
where
    FileId: Copy + Ord,
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
        let mut primary_labels = 0;

        let labels = self.diagnostic.labels.iter();
        for label in labels.filter(|label| label.style == LabelStyle::Primary) {
            primary_labels += 1;

            let origin = files.origin(label.file_id).expect("origin");
            let start = label.range.start;
            let line_index = files.line_index(label.file_id, start).expect("line_index");
            let line = files.line(label.file_id, line_index).expect("line");

            Locus::new(origin, line.number, line.column_number(start)).emit(writer, config)?;
            write!(writer, ": ")?;
            Header::new(self.diagnostic).emit(writer, config)?;
        }

        // Fallback to printing a non-located header if no primary labels were encountered
        if primary_labels == 0 {
            Header::new(self.diagnostic).emit(writer, config)?;
        }

        Ok(())
    }
}
