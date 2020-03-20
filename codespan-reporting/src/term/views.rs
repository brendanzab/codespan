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
            let start_line_index = files.line_index(label.file_id, label.range.start).unwrap();
            let end_line_index = files.line_index(label.file_id, label.range.end).unwrap();
            let end_line_number = files.line_number(label.file_id, end_line_index).unwrap();

            // The label spans over multiple lines if if the line indices of the
            // start and end differ.
            let is_multiline = start_line_index != end_line_index;
            // Update the outer padding based on the last line number
            outer_padding = std::cmp::max(outer_padding, count_digits(end_line_number));

            // TODO(#100): Group contiguous line index ranges using some sort of interval set algorithm.
            // TODO(#100): Flatten mark groups to overlapping underlines that can be easily rendered.
            // TODO(#125): If start line and end line are too far apart, we should add a source break.

            // NOTE: This could be made more efficient by using an associative
            // data structure like a hashmap or B-tree,  but we use a vector to
            // preserve the order that unique files appear in the list of labels.
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
                        // to piggyback off its lexicographic comparison implementation.
                        (other.range.start, other.range.end)
                            .cmp(&(label.range.start, label.range.end))
                    }) {
                        Ok(index) | Err(index) => labels.insert(index, label),
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
        for (file_id, seen_multiline, labels) in file_ids_to_labels {
            let source = files.source(file_id).unwrap();
            let source = source.as_ref();

            // Top left border and locus.
            //
            // ```text
            // ┌── test:2:9 ───
            // ```
            if let Some(label) = labels.first() {
                renderer.render_source_start(
                    outer_padding,
                    &Locus {
                        name: files.name(file_id).unwrap().to_string(),
                        location: files.location(file_id, label.range.start).unwrap(),
                    },
                )?;
                renderer.render_source_empty(outer_padding, &[])?;
            }

            let mut labels = labels.into_iter().peekable();
            let mut previous_end_line_index = None;

            while let Some(label) = labels.next() {
                let severity = match label.style {
                    LabelStyle::Primary => Some(self.diagnostic.severity),
                    LabelStyle::Secondary => None,
                };

                let start_line_index = files.line_index(file_id, label.range.start).unwrap();
                let start_line_number = files.line_number(file_id, start_line_index).unwrap();
                let start_line_range = files.line_range(file_id, start_line_index).unwrap();
                let end_line_index = files.line_index(file_id, label.range.end).unwrap();
                let end_line_number = files.line_number(file_id, end_line_index).unwrap();
                let end_line_range = files.line_range(file_id, end_line_index).unwrap();

                // Check to see if we need to render any intermediate stuff
                // before rendering the current mark.
                if let Some(previous_end_line_index) = previous_end_line_index {
                    match start_line_index.checked_sub(previous_end_line_index) {
                        // Current mark is on the same line as the previous mark
                        Some(0) => {
                            // TODO: Accumulate marks here
                            renderer.render_source_break(outer_padding, &[])?;
                        }
                        // Current mark is on the next consecutive line
                        Some(1) => {}
                        // One line between the current mark and the previous mark
                        Some(2) => {
                            // Write a source line
                            let next_index = previous_end_line_index + 1;
                            renderer.render_source_line(
                                outer_padding,
                                files.line_number(file_id, next_index).unwrap(),
                                &source[files.line_range(file_id, next_index).unwrap()],
                                match seen_multiline {
                                    true => &[None],
                                    false => &[],
                                },
                            )?;
                        }
                        // More than one line between the current mark and the
                        // previous mark - or the marks are out of order.
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

                // Render the current label.
                if start_line_index == end_line_index {
                    // Single line
                    //
                    // ```text
                    // 2 │ (+ test "")
                    //   │         ^^ expected `Int` but found `String`
                    // ```
                    let mark_start = label.range.start - start_line_range.start;
                    let mark_end = label.range.end - start_line_range.start;

                    let mark = Mark::Single(severity, mark_start..mark_end, &label.message);
                    let mut marks = match seen_multiline {
                        true => vec![None, Some(mark)],
                        false => vec![Some(mark)],
                    };

                    // Accumulate consecutive single marks
                    // TODO: it feels like this could be merged with the check
                    // of `previous_end_index` above?
                    while let Some(next_label) = labels.peek() {
                        let severity = match next_label.style {
                            LabelStyle::Primary => Some(self.diagnostic.severity),
                            LabelStyle::Secondary => None,
                        };

                        let range = next_label.range.clone();
                        let next_start_line_index = files.line_index(file_id, range.start).unwrap();
                        let next_end_line_index = files.line_index(file_id, range.end).unwrap();

                        if start_line_index == next_start_line_index
                            && next_start_line_index == next_end_line_index
                        {
                            let mark_start = next_label.range.start - start_line_range.start;
                            let mark_end = next_label.range.end - start_line_range.start;

                            marks.push(Some(Mark::Single(
                                severity,
                                mark_start..mark_end,
                                &next_label.message,
                            )));
                        } else {
                            break;
                        }

                        labels.next();
                    }

                    renderer.render_source_line(
                        outer_padding,
                        start_line_number,
                        &source[start_line_range],
                        &marks,
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
                    let mark_start = label.range.start - start_line_range.start;
                    let prefix_source = &source[start_line_range.start..label.range.start];

                    if prefix_source.trim().is_empty() {
                        // Section is prefixed by empty space, so we don't need to take
                        // up a new line.
                        //
                        // ```text
                        // 4 │ ╭     case (mod num 5) (mod num 3) of
                        // ```
                        renderer.render_source_line(
                            outer_padding,
                            start_line_number,
                            &source[start_line_range],
                            &[Some(Mark::MultiTopLeft(severity))],
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
                            start_line_number,
                            &source[start_line_range],
                            &[Some(Mark::MultiTop(severity, ..mark_start))],
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
                        renderer.render_source_line(
                            outer_padding,
                            files.line_number(file_id, marked_line_index).unwrap(),
                            &source[files.line_range(file_id, marked_line_index).unwrap()],
                            &[Some(Mark::MultiLeft(severity))],
                        )?;
                    }

                    // Write last marked line
                    //
                    // ```text
                    // 8 │ │     _ _ => num
                    //   │ ╰──────────────^ `case` clauses have incompatible types
                    // ```
                    let mark_end = label.range.end - end_line_range.start;

                    renderer.render_source_line(
                        outer_padding,
                        end_line_number,
                        &source[end_line_range],
                        &[Some(Mark::MultiBottom(
                            severity,
                            ..mark_end,
                            &label.message,
                        ))],
                    )?;
                }

                previous_end_line_index = Some(end_line_index);
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

            renderer.render_header(
                Some(&Locus {
                    name: files.name(label.file_id).unwrap().to_string(),
                    location: files.location(label.file_id, label.range.start).unwrap(),
                }),
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
