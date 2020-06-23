use std::io;
use std::ops::Range;

use crate::diagnostic::{Diagnostic, LabelStyle};
use crate::files::{Files, Location};
use crate::term::renderer::{Locus, MultiLabel, Renderer, SingleLabel};

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
        use std::collections::BTreeMap;

        struct LabeledFile<'diagnostic, FileId> {
            file_id: FileId,
            start: usize,
            name: String,
            location: Location,
            num_multi_labels: usize,
            lines: BTreeMap<usize, Line<'diagnostic>>,
        }

        impl<'diagnostic, FileId> LabeledFile<'diagnostic, FileId> {
            fn get_or_insert_line(
                &mut self,
                line_index: usize,
                line_range: Range<usize>,
                line_number: usize,
            ) -> &mut Line<'diagnostic> {
                self.lines.entry(line_index).or_insert_with(|| Line {
                    range: line_range,
                    number: line_number,
                    single_labels: vec![],
                    multi_labels: vec![],
                })
            }
        }

        struct Line<'diagnostic> {
            number: usize,
            range: std::ops::Range<usize>,
            // TODO: How do we reuse these allocations?
            single_labels: Vec<SingleLabel<'diagnostic>>,
            multi_labels: Vec<(usize, LabelStyle, MultiLabel<'diagnostic>)>,
        }

        // TODO: Make this data structure external, to allow for allocation reuse
        let mut labeled_files = Vec::<LabeledFile<'_, _>>::new();
        // Keep track of the outer padding to use when rendering the
        // snippets of source code.
        let mut outer_padding = 0;

        // Group labels by file
        for label in &self.diagnostic.labels {
            let source = files.source(label.file_id).unwrap();
            let source = source.as_ref();

            let start_line_index = files.line_index(label.file_id, label.range.start).unwrap();
            let start_line_number = files.line_number(label.file_id, start_line_index).unwrap();
            let start_line_range = files.line_range(label.file_id, start_line_index).unwrap();
            let end_line_index = files.line_index(label.file_id, label.range.end).unwrap();
            let end_line_number = files.line_number(label.file_id, end_line_index).unwrap();
            let end_line_range = files.line_range(label.file_id, end_line_index).unwrap();

            outer_padding = std::cmp::max(outer_padding, count_digits(start_line_number));
            outer_padding = std::cmp::max(outer_padding, count_digits(end_line_number));

            // NOTE: This could be made more efficient by using an associative
            // data structure like a hashmap or B-tree,  but we use a vector to
            // preserve the order that unique files appear in the list of labels.
            let labeled_file = match labeled_files
                .iter_mut()
                .find(|labeled_file| label.file_id == labeled_file.file_id)
            {
                Some(labeled_file) => {
                    if labeled_file.start > label.range.start {
                        labeled_file.start = label.range.start;
                        labeled_file.location =
                            files.location(label.file_id, label.range.start).unwrap();
                    }
                    labeled_file
                }
                None => {
                    labeled_files.push(LabeledFile {
                        file_id: label.file_id,
                        start: label.range.start,
                        name: files.name(label.file_id).unwrap().to_string(),
                        location: files.location(label.file_id, label.range.start).unwrap(),
                        num_multi_labels: 0,
                        lines: BTreeMap::new(),
                    });
                    labeled_files.last_mut().unwrap()
                }
            };

            if start_line_index == end_line_index {
                // Single line
                //
                // ```text
                // 2 │ (+ test "")
                //   │         ^^ expected `Int` but found `String`
                // ```
                let label_start = label.range.start - start_line_range.start;
                let label_end = label.range.end - start_line_range.start;

                let line = labeled_file.get_or_insert_line(
                    start_line_index,
                    start_line_range,
                    start_line_number,
                );

                // Ensure that the single line labels are lexicographically
                // sorted by the range of source code that they cover.
                let index = match line.single_labels.binary_search_by(|(_, range, _)| {
                    // `Range<usize>` doesn't implement `Ord`, so convert to `(usize, usize)`
                    // to piggyback off its lexicographic comparison implementation.
                    (range.start, range.end).cmp(&(label_start, label_end))
                }) {
                    // If the ranges are the same, order the labels in reverse
                    // to how they were originally specified in the diagnostic.
                    // This helps with printing in the renderer.
                    Ok(index) | Err(index) => index,
                };

                // Ensure that we print at least one caret, even when we
                // have a zero-length source range.
                let mut label_range = label_start..label_end;
                if label_range.len() == 0 {
                    label_range.end = label_range.start + 1;
                }

                line.single_labels
                    .insert(index, (label.style, label_range, &label.message));
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

                let label_index = labeled_file.num_multi_labels;
                labeled_file.num_multi_labels += 1;

                // First labeled line
                let label_start = label.range.start - start_line_range.start;
                let prefix_source = &source[start_line_range.start..label.range.start];

                labeled_file
                    .get_or_insert_line(start_line_index, start_line_range, start_line_number)
                    .multi_labels
                    // TODO: Do this in the `Renderer`?
                    .push(match prefix_source.trim() {
                        // Section is prefixed by empty space, so we don't need to take
                        // up a new line.
                        //
                        // ```text
                        // 4 │ ╭     case (mod num 5) (mod num 3) of
                        // ```
                        "" => (label_index, label.style, MultiLabel::TopLeft),
                        // There's source code in the prefix, so run a label
                        // underneath it to get to the start of the range.
                        //
                        // ```text
                        // 4 │   fizz₁ num = case (mod num 5) (mod num 3) of
                        //   │ ╭─────────────^
                        // ```
                        _ => (label_index, label.style, MultiLabel::Top(..label_start)),
                    });

                // Marked lines
                //
                // ```text
                // 5 │ │     0 0 => "FizzBuzz"
                // 6 │ │     0 _ => "Fizz"
                // 7 │ │     _ 0 => "Buzz"
                // ```
                // TODO(#125): If start line and end line are too far apart, add a source break.
                for line_index in (start_line_index + 1)..end_line_index {
                    let line_range = files.line_range(label.file_id, line_index).unwrap();
                    let line_number = files.line_number(label.file_id, line_index).unwrap();

                    outer_padding = std::cmp::max(outer_padding, count_digits(line_number));

                    labeled_file
                        .get_or_insert_line(line_index, line_range, line_number)
                        .multi_labels
                        .push((label_index, label.style, MultiLabel::Left));
                }

                // Last labeled line
                //
                // ```text
                // 8 │ │     _ _ => num
                //   │ ╰──────────────^ `case` clauses have incompatible types
                // ```
                let label_end = label.range.end - end_line_range.start;

                labeled_file
                    .get_or_insert_line(end_line_index, end_line_range, end_line_number)
                    .multi_labels
                    .push((
                        label_index,
                        label.style,
                        MultiLabel::Bottom(..label_end, &label.message),
                    ));
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

        // Source snippets
        //
        // ```text
        //   ┌─ test:2:9
        //   │
        // 2 │ (+ test "")
        //   │         ^^ expected `Int` but found `String`
        //   │
        // ```
        let mut labeled_files = labeled_files.into_iter().peekable();
        while let Some(labeled_file) = labeled_files.next() {
            let source = files.source(labeled_file.file_id).unwrap();
            let source = source.as_ref();

            // Top left border and locus.
            //
            // ```text
            // ┌─ test:2:9
            // ```
            if !labeled_file.lines.is_empty() {
                renderer.render_snippet_start(
                    outer_padding,
                    &Locus {
                        name: labeled_file.name,
                        location: labeled_file.location,
                    },
                )?;
                renderer.render_snippet_empty(
                    outer_padding,
                    self.diagnostic.severity,
                    labeled_file.num_multi_labels,
                    &[],
                )?;
            }

            let mut lines = labeled_file.lines.into_iter().peekable();
            let current_labels = Vec::new();

            while let Some((line_index, line)) = lines.next() {
                renderer.render_snippet_source(
                    outer_padding,
                    line.number,
                    &source[line.range.clone()],
                    self.diagnostic.severity,
                    &line.single_labels,
                    labeled_file.num_multi_labels,
                    &line.multi_labels,
                )?;

                // Check to see if we need to render any intermediate stuff
                // before rendering the next line.
                if let Some((next_line_index, _)) = lines.peek() {
                    match next_line_index.checked_sub(line_index) {
                        // Consecutive lines
                        Some(1) => {}
                        // One line between the current line and the next line
                        Some(2) => {
                            // Write a source line
                            let file_id = labeled_file.file_id;
                            renderer.render_snippet_source(
                                outer_padding,
                                files.line_number(file_id, line_index + 1).unwrap(),
                                &source[files.line_range(file_id, line_index + 1).unwrap()],
                                self.diagnostic.severity,
                                &[],
                                labeled_file.num_multi_labels,
                                &current_labels,
                            )?;
                        }
                        // More than one line between the current line and the next line.
                        Some(_) | None => {
                            // Source break
                            //
                            // ```text
                            // ·
                            // ```
                            renderer.render_snippet_break(
                                outer_padding,
                                self.diagnostic.severity,
                                labeled_file.num_multi_labels,
                                &current_labels,
                            )?;
                        }
                    }
                }
            }

            // Check to see if we should render a trailing border after the
            // final line of the snippet.
            if labeled_files.peek().is_none() && self.diagnostic.notes.is_empty() {
                // We don't render a border if we are at the final newline
                // without trailing notes, because it would end up looking too
                // spaced-out in combination with the final new line.
            } else {
                // Render the trailing snippet border.
                renderer.render_snippet_empty(
                    outer_padding,
                    self.diagnostic.severity,
                    labeled_file.num_multi_labels,
                    &current_labels,
                )?;
            }
        }

        // Additional notes
        //
        // ```text
        // = expected type `Int`
        //      found type `String`
        // ```
        for note in &self.diagnostic.notes {
            renderer.render_snippet_note(outer_padding, note)?;
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
