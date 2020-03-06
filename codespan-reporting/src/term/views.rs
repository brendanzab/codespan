use std::io;

use crate::diagnostic::{Diagnostic, LabelStyle};
use crate::files::Files;
use crate::term::renderer::{Locus, Mark, Renderer};

/// Count the number of decimal digits in `n`.
fn count_digits(mut n: usize) -> usize {
    let mut count = 0;
    while n != 0 {
        count += 1;
        n /= 10; // remove last digit
    }
    count
}

/// Output a richly formatted diagnostic, with source code previews.
pub struct RichDiagnostic<'diagnostic, FileId> {
    diagnostic: &'diagnostic Diagnostic<FileId>,
}

impl<'diagnostic, FileId> RichDiagnostic<'diagnostic, FileId>
where
    FileId: Copy + PartialEq,
{
    pub fn new(diagnostic: &'diagnostic Diagnostic<FileId>) -> RichDiagnostic<'diagnostic, FileId> {
        RichDiagnostic { diagnostic }
    }

    pub fn render<'files>(
        &self,
        files: &'files impl Files<'files, FileId = FileId>,
        renderer: &mut Renderer<'_, '_>,
    ) -> io::Result<()>
    where
        FileId: 'files,
    {
        // TODO: Make this data structure external, to allow for allocation reuse
        let mut file_ids_to_labels = Vec::new();
        let mut outer_padding = 0;
        // Group marks by file
        for label in &self.diagnostic.labels {
            let start_line_index = files
                .line_index(label.file_id, label.range.start)
                .expect("start_line_index");
            let end_line_index = files
                .line_index(label.file_id, label.range.end)
                .expect("end_line_index");
            let end_line = files.line(label.file_id, end_line_index).expect("end_line");
            // The label spans over multiple lines if if the line indices of the
            // start and end differ.
            let is_multiline = start_line_index != end_line_index;
            // Update the outer padding based on the last lin number
            outer_padding = std::cmp::max(outer_padding, count_digits(end_line.number));

            // TODO: Group contiguous line index ranges using some sort of interval set algorithm.
            // TODO: Flatten mark groups to overlapping underlines that can be easily rendered.
            // TODO: If start line and end line are too far apart, we should add a source break.
            match file_ids_to_labels
                .iter_mut()
                .find(|(file_id, _, _)| label.file_id == *file_id)
            {
                None => file_ids_to_labels.push((label.file_id, is_multiline, vec![label])),
                Some((_, seen_multiline, labels)) => {
                    // Keep track of if we've sen a multiline label. This helps
                    // us to figure out the inner padding of the source snippet.
                    // TODO: This will need to be more complicated once we allow multiline labels to overlap.
                    *seen_multiline |= is_multiline;
                    // Ensure that the vector of labels is sorted
                    // lexicographically by the range of source code they cover.
                    // This should make our job easier later on.
                    match labels.binary_search_by(|other| {
                        // `Range<usize>` doesn't implement `Ord`, so convert to `(usize, usize)`
                        // to piggyback off its lexicographic sorting implementation.
                        (other.range.start, other.range.end)
                            .cmp(&(label.range.start, label.range.end))
                    }) {
                        Ok(i) | Err(i) => labels.insert(i, label),
                    }
                }
            }
        }

        // Header and message
        //
        // ```text
        // error[E0001]: unexpected type in `+` application
        // ```
        renderer.render_header(
            None,
            self.diagnostic.severity,
            self.diagnostic.code.as_ref().map(String::as_str),
            self.diagnostic.message.as_str(),
        )?;
        if !file_ids_to_labels.is_empty() {
            renderer.render_empty()?;
        }

        // Source snippets
        //
        // ```text
        //
        //   ┌── test:2:9 ───
        //   │
        // 2 │ (+ test "")
        //   │         ^^ expected `Int` but found `String`
        //   │
        // ```
        for (file_id, seen_multiline, labels) in &file_ids_to_labels {
            let source = files.source(*file_id).expect("source");
            let mut labels = labels
                .iter()
                .map(|label| {
                    let start_line_index = files
                        .line_index(label.file_id, label.range.start)
                        .expect("start_line_index");
                    let start_line = files
                        .line(label.file_id, start_line_index)
                        .expect("start_line");
                    let end_line_index = files
                        .line_index(label.file_id, label.range.end)
                        .expect("end_line_index");
                    let end_line = files.line(label.file_id, end_line_index).expect("end_line");
                    (
                        label,
                        (start_line_index, start_line),
                        (end_line_index, end_line),
                    )
                })
                .peekable();

            // Top left border and locus.
            //
            // ```text
            // ┌── test:2:9 ───
            // ```
            if let Some((label, (_, start_line), _)) = labels.peek() {
                let start_source = &source.as_ref()[start_line.range.clone()];
                renderer.render_source_start(
                    outer_padding,
                    &Locus {
                        origin: files.origin(*file_id).expect("origin").to_string(),
                        line_number: start_line.number,
                        column_number: start_line.column_number(start_source, label.range.start),
                    },
                )?;
                renderer.render_source_empty(outer_padding, &[])?;
            }

            while let Some((label, (start_line_index, start_line), (end_line_index, end_line))) =
                labels.next()
            {
                let severity = match label.style {
                    LabelStyle::Primary => Some(self.diagnostic.severity),
                    LabelStyle::Secondary => None,
                };

                let start_source = &source.as_ref()[start_line.range.clone()];
                let end_source = &source.as_ref()[end_line.range.clone()];

                // Attempt to split off the last line.
                if start_line_index == end_line_index {
                    // Single line
                    //
                    // ```text
                    // 2 │ (+ test "")
                    //   │         ^^ expected `Int` but found `String`
                    // ```
                    let mark_start = label.range.start - start_line.range.start;
                    let mark_end = label.range.end - start_line.range.start;

                    let mark = || (severity, Mark::Single(mark_start..mark_end, &label.message));
                    let multi_marks = [None, Some(mark())];
                    let single_marks = [Some(mark())];

                    renderer.render_source_line(
                        outer_padding,
                        start_line.number,
                        start_source.as_ref(),
                        match seen_multiline {
                            true => &multi_marks,
                            false => &single_marks,
                        },
                    )?;
                } else {
                    // Multiple lines
                    //
                    // ```text
                    // 4 │   fizz₁ num = case (mod num 5) (mod num 3) of
                    //   │ ╭─────────────^
                    // 5 │ │     0 0 => "FizzBuzz"
                    // 6 │ │     0 _ => "Fizz"
                    // 7 │ │     _ 0 => "Buzz"
                    // 8 │ │     _ _ => num
                    //   │ ╰──────────────^ `case` clauses have incompatible types
                    // ```
                    let mark_start = label.range.start - start_line.range.start;
                    let prefix_source = &start_source[..mark_start];

                    if prefix_source.trim().is_empty() {
                        // Section is prefixed by empty space, so we don't need to take
                        // up a new line.
                        //
                        // ```text
                        // 4 │ ╭     case (mod num 5) (mod num 3) of
                        // ```
                        renderer.render_source_line(
                            outer_padding,
                            start_line.number,
                            start_source.as_ref(),
                            &[Some((severity, Mark::MultiTopLeft))],
                        )?;
                    } else {
                        // There's source code in the prefix, so run an underline
                        // underneath it to get to the start of the range.
                        //
                        // ```text
                        // 4 │   fizz₁ num = case (mod num 5) (mod num 3) of
                        //   │ ╭─────────────^
                        // ```
                        renderer.render_source_line(
                            outer_padding,
                            start_line.number,
                            start_source.as_ref(),
                            &[Some((severity, Mark::MultiTop(..mark_start)))],
                        )?;
                    }

                    // Write marked lines
                    //
                    // ```text
                    // 5 │ │     0 0 => "FizzBuzz"
                    // 6 │ │     0 _ => "Fizz"
                    // 7 │ │     _ 0 => "Buzz"
                    // ```
                    for marked_line_index in (start_line_index + 1)..end_line_index {
                        let marked_line = files
                            .line(label.file_id, marked_line_index)
                            .expect("marked_line");
                        let marked_source = &source.as_ref()[marked_line.range.clone()];
                        renderer.render_source_line(
                            outer_padding,
                            marked_line.number,
                            marked_source,
                            &[Some((severity, Mark::MultiLeft))],
                        )?;
                    }

                    // Write last marked line
                    //
                    // ```text
                    // 8 │ │     _ _ => num
                    //   │ ╰──────────────^ `case` clauses have incompatible types
                    // ```
                    let mark_end = label.range.end - end_line.range.start;

                    renderer.render_source_line(
                        outer_padding,
                        end_line.number,
                        end_source.as_ref(),
                        &[Some((
                            severity,
                            Mark::MultiBottom(..mark_end, &label.message),
                        ))],
                    )?;
                }

                if let Some((_, (next_start_line_index, _), _)) = labels.peek() {
                    match next_start_line_index.checked_sub(end_line_index) {
                        // Same line
                        Some(0) => {
                            // TODO: Accumulate marks!
                            renderer.render_source_break(outer_padding, &[])?;
                        }
                        // Consecutive lines
                        Some(1) => {}
                        // Only one line between us and the next label
                        Some(2) => {
                            // Write a source line
                            let next_line = files
                                .line(label.file_id, end_line_index + 1)
                                .expect("next_line");
                            let next_source = &source.as_ref()[next_line.range.clone()];
                            renderer.render_source_line(
                                outer_padding,
                                next_line.number,
                                next_source,
                                match seen_multiline {
                                    true => &[None],
                                    false => &[],
                                },
                            )?;
                        }
                        // Either:
                        // - one line between us and the next label
                        // - labels are out of order
                        Some(_) | None => {
                            // Source break
                            //
                            // ```text
                            // ·
                            // ```
                            renderer.render_source_break(outer_padding, &[])?;
                        }
                    }
                }
            }
            renderer.render_source_empty(outer_padding, &[])?;
        }

        // Additional notes
        //
        // ```text
        // = expected type `Int`
        //      found type `String`
        // ```
        for note in &self.diagnostic.notes {
            renderer.render_source_note(outer_padding, note)?;
        }
        renderer.render_empty()?;

        Ok(())
    }
}

/// Output a short diagnostic, with a line number, severity, and message.
pub struct ShortDiagnostic<'diagnostic, FileId> {
    diagnostic: &'diagnostic Diagnostic<FileId>,
}

impl<'diagnostic, FileId> ShortDiagnostic<'diagnostic, FileId>
where
    FileId: Copy + PartialEq,
{
    pub fn new(
        diagnostic: &'diagnostic Diagnostic<FileId>,
    ) -> ShortDiagnostic<'diagnostic, FileId> {
        ShortDiagnostic { diagnostic }
    }

    pub fn render<'files>(
        &self,
        files: &'files impl Files<'files, FileId = FileId>,
        renderer: &mut Renderer<'_, '_>,
    ) -> io::Result<()>
    where
        FileId: 'files,
    {
        // Located headers
        //
        // ```text
        // test:2:9: error[E0001]: unexpected type in `+` application
        // ```
        let mut primary_labels_encountered = 0;
        let labels = self.diagnostic.labels.iter();
        for label in labels.filter(|label| label.style == LabelStyle::Primary) {
            primary_labels_encountered += 1;

            let locus = {
                let start = label.range.start;
                let source = files.source(label.file_id).expect("source");
                let line_index = files.line_index(label.file_id, start).expect("line_index");
                let line = files.line(label.file_id, line_index).expect("line");
                let line_source = &source.as_ref()[line.range.clone()];

                Locus {
                    origin: files.origin(label.file_id).expect("origin").to_string(),
                    line_number: line.number,
                    column_number: line.column_number(line_source, start),
                }
            };

            renderer.render_header(
                Some(&locus),
                self.diagnostic.severity,
                self.diagnostic.code.as_ref().map(String::as_str),
                self.diagnostic.message.as_str(),
            )?;
        }

        // Fallback to printing a non-located header if no primary labels were encountered
        //
        // ```text
        // error[E0002]: Bad config found
        // ```
        if primary_labels_encountered == 0 {
            renderer.render_header(
                None,
                self.diagnostic.severity,
                self.diagnostic.code.as_ref().map(String::as_str),
                self.diagnostic.message.as_str(),
            )?;
        }

        Ok(())
    }
}
