use std::io;
use std::ops::Range;

use crate::diagnostic::{Diagnostic, LabelStyle};
use crate::files::{Files, Location};
use crate::term::renderer::{Locus, MultiMark, Renderer, SingleMark};

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

        struct MarkedFile<'diagnostic, FileId> {
            file_id: FileId,
            start: usize,
            name: String,
            location: Location,
            num_multi_marks: usize,
            lines: BTreeMap<usize, Line<'diagnostic>>,
        }

        impl<'diagnostic, FileId> MarkedFile<'diagnostic, FileId> {
            fn get_or_insert_line(
                &mut self,
                line_index: usize,
                line_range: Range<usize>,
                line_number: usize,
            ) -> &mut Line<'diagnostic> {
                self.lines.entry(line_index).or_insert_with(|| Line {
                    range: line_range,
                    number: line_number,
                    single_marks: vec![],
                    multi_marks: vec![],
                })
            }
        }

        struct Line<'diagnostic> {
            number: usize,
            range: std::ops::Range<usize>,
            // TODO: How do we reuse these allocations?
            single_marks: Vec<SingleMark<'diagnostic>>,
            multi_marks: Vec<(usize, MultiMark<'diagnostic>)>,
        }

        // TODO: Make this data structure external, to allow for allocation reuse
        let mut marked_files = Vec::<MarkedFile<'_, _>>::new();
        // Keep track of the outer padding to use when rendering the
        // snippets of source code.
        let mut outer_padding = 0;

        // Group marks by file
        for label in &self.diagnostic.labels {
            let severity = match label.style {
                LabelStyle::Primary => Some(self.diagnostic.severity),
                LabelStyle::Secondary => None,
            };

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
            let marked_file = match marked_files
                .iter_mut()
                .find(|marked_file| label.file_id == marked_file.file_id)
            {
                Some(marked_file) => {
                    if marked_file.start > label.range.start {
                        marked_file.start = label.range.start;
                        marked_file.location =
                            files.location(label.file_id, label.range.start).unwrap();
                    }
                    marked_file
                }
                None => {
                    marked_files.push(MarkedFile {
                        file_id: label.file_id,
                        start: label.range.start,
                        name: files.name(label.file_id).unwrap().to_string(),
                        location: files.location(label.file_id, label.range.start).unwrap(),
                        num_multi_marks: 0,
                        lines: BTreeMap::new(),
                    });
                    marked_files.last_mut().unwrap()
                }
            };

            if start_line_index == end_line_index {
                // Single line
                //
                // ```text
                // 2 │ (+ test "")
                //   │         ^^ expected `Int` but found `String`
                // ```
                let mark_start = label.range.start - start_line_range.start;
                let mark_end = label.range.end - start_line_range.start;

                let line = marked_file.get_or_insert_line(
                    start_line_index,
                    start_line_range,
                    start_line_number,
                );

                // Ensure that the single line labels are lexicographically
                // sorted by the range of source code that they cover.
                let index = match line.single_marks.binary_search_by(|(_, range, _)| {
                    // `Range<usize>` doesn't implement `Ord`, so convert to `(usize, usize)`
                    // to piggyback off its lexicographic comparison implementation.
                    (range.start, range.end).cmp(&(mark_start, mark_end))
                }) {
                    Ok(index) | Err(index) => index,
                };

                line.single_marks
                    .insert(index, (severity, mark_start..mark_end, &label.message));
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

                let mark_index = marked_file.num_multi_marks;
                marked_file.num_multi_marks += 1;

                // First marked line
                let mark_start = label.range.start - start_line_range.start;
                let prefix_source = &source[start_line_range.start..label.range.start];

                marked_file
                    .get_or_insert_line(start_line_index, start_line_range, start_line_number)
                    .multi_marks
                    // TODO: Do this in the `Renderer`?
                    .push(match prefix_source.trim() {
                        // Section is prefixed by empty space, so we don't need to take
                        // up a new line.
                        //
                        // ```text
                        // 4 │ ╭     case (mod num 5) (mod num 3) of
                        // ```
                        "" => (mark_index, MultiMark::TopLeft(severity)),
                        // There's source code in the prefix, so run a mark
                        // underneath it to get to the start of the range.
                        //
                        // ```text
                        // 4 │   fizz₁ num = case (mod num 5) (mod num 3) of
                        //   │ ╭─────────────^
                        // ```
                        _ => (mark_index, MultiMark::Top(severity, ..mark_start)),
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

                    marked_file
                        .get_or_insert_line(line_index, line_range, line_number)
                        .multi_marks
                        .push((mark_index, MultiMark::Left(severity)));
                }

                // Last marked line
                //
                // ```text
                // 8 │ │     _ _ => num
                //   │ ╰──────────────^ `case` clauses have incompatible types
                // ```
                let mark_end = label.range.end - end_line_range.start;

                marked_file
                    .get_or_insert_line(end_line_index, end_line_range, end_line_number)
                    .multi_marks
                    .push((
                        mark_index,
                        MultiMark::Bottom(severity, ..mark_end, &label.message),
                    ));
            }
        }

        // TODO: Insert `None` spaces in `marked_files`

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
        if !marked_files.is_empty() {
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
        for marked_file in marked_files {
            let source = files.source(marked_file.file_id).unwrap();
            let source = source.as_ref();

            // Top left border and locus.
            //
            // ```text
            // ┌── test:2:9 ───
            // ```
            if !marked_file.lines.is_empty() {
                renderer.render_source_start(
                    outer_padding,
                    &Locus {
                        name: marked_file.name,
                        location: marked_file.location,
                    },
                )?;
                renderer.render_source_empty(outer_padding, marked_file.num_multi_marks, &[])?;
            }

            let mut lines = marked_file.lines.into_iter().peekable();
            let current_marks = Vec::new();

            while let Some((line_index, line)) = lines.next() {
                renderer.render_source_line(
                    outer_padding,
                    line.number,
                    &source[line.range.clone()],
                    &line.single_marks,
                    marked_file.num_multi_marks,
                    &line.multi_marks,
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
                            let file_id = marked_file.file_id;
                            renderer.render_source_line(
                                outer_padding,
                                files.line_number(file_id, line_index + 1).unwrap(),
                                &source[files.line_range(file_id, line_index + 1).unwrap()],
                                &[],
                                marked_file.num_multi_marks,
                                &current_marks,
                            )?;
                        }
                        // More than one line between the current line and the next line.
                        Some(_) | None => {
                            // Source break
                            //
                            // ```text
                            // ·
                            // ```
                            renderer.render_source_break(
                                outer_padding,
                                marked_file.num_multi_marks,
                                &current_marks,
                            )?;
                        }
                    }
                }
            }
            renderer.render_source_empty(
                outer_padding,
                marked_file.num_multi_marks,
                &current_marks,
            )?;
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
