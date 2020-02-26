use std::io;
use std::ops::Range;
use termcolor::WriteColor;

use crate::diagnostic::Label;
use crate::term::Config;
use crate::Files;

use super::{Locus, NewLine};

mod border;
mod gutter;
mod note;
mod underline;

use self::border::{BorderLeft, BorderLeftBreak, BorderTop, BorderTopLeft};
use self::gutter::Gutter;
use self::note::Note;
use self::underline::{Underline, UnderlineBottom, UnderlineLeft, UnderlineTop, UnderlineTopLeft};

pub use self::underline::MarkStyle;

/// Count the number of digits in `n`.
fn count_digits(mut n: usize) -> usize {
    let mut count = 0;
    while n != 0 {
        count += 1;
        n /= 10; // remove last digit
    }
    count
}

/// An underlined snippet of source code.
///
/// ```text
///   ┌── test:2:9 ───
///   │
/// 2 │ (+ test "")
///   │         ^^ expected `Int` but found `String`
///   │
///   = expected type `Int`
///        found type `String`
/// ```
pub struct SourceSnippet<'a, F: Files> {
    file_id: F::FileId,
    ranges: Vec<(&'a Label<F::FileId>, MarkStyle)>,
    notes: &'a [String],
}

impl<'a, F: Files> SourceSnippet<'a, F> {
    pub fn new(
        file_id: F::FileId,
        ranges: Vec<(&'a Label<F::FileId>, MarkStyle)>,
        notes: &'a [String],
    ) -> SourceSnippet<'a, F> {
        SourceSnippet {
            file_id,
            ranges,
            notes,
        }
    }

    fn source_locus_ranges(&self) -> (Range<usize>, Range<usize>) {
        fn merge(range0: Range<usize>, range1: Range<usize>) -> Range<usize> {
            let start = std::cmp::min(range0.start, range1.start);
            let end = std::cmp::max(range0.end, range1.end);
            start..end
        }

        let mut source_range = None;
        let mut locus_range = None;

        for (label, mark_style) in &self.ranges {
            source_range = Some(source_range.map_or(label.range.clone(), |range| {
                merge(range, label.range.clone())
            }));
            if let MarkStyle::Primary(_) = mark_style {
                locus_range = Some(locus_range.map_or(label.range.clone(), |range| {
                    merge(range, label.range.clone())
                }));
            }
        }

        let source_range = source_range.unwrap_or(0..0);
        let locus_range = locus_range.unwrap_or(source_range.clone());

        (source_range, locus_range)
    }

    pub fn emit(
        &self,
        files: &F,
        writer: &mut (impl WriteColor + ?Sized),
        config: &Config,
    ) -> io::Result<()> {
        use std::io::Write;

        let origin = files.origin(self.file_id).expect("origin");
        let location = |byte_index| files.location(self.file_id, byte_index);
        let line_index = |byte_index| files.line_index(self.file_id, byte_index);
        let line = |line_index| files.line(self.file_id, line_index);

        let (source_range, locus_range) = self.source_locus_ranges();

        // Use the length of the last line number as the gutter padding
        let gutter_padding = {
            let line_index = line_index(source_range.end).expect("source_end_line_index");
            let line = line(line_index).expect("source_end_line_number");
            count_digits(line.number)
        };

        // Top left border and locus.
        //
        // ```text
        // ┌── test:2:9 ───
        // ```

        Gutter::new(None, gutter_padding).emit(writer, config)?;
        BorderTopLeft::new().emit(writer, config)?;
        BorderTop::new(2).emit(writer, config)?;
        write!(writer, " ")?;

        let locus_location = location(locus_range.start).expect("locus_location");
        Locus::new(origin, locus_location).emit(writer, config)?;

        write!(writer, " ")?;
        BorderTop::new(3).emit(writer, config)?;
        NewLine::new().emit(writer, config)?;

        // TODO: Better grouping
        for (i, (label, mark_style)) in self.ranges.iter().enumerate() {
            let start_line_index = line_index(label.range.start).expect("start_line_index");
            let end_line_index = line_index(label.range.end).expect("end_line_index");
            let start_line = line(start_line_index).expect("start_line");
            let end_line = line(end_line_index).expect("end_line");

            let start_source = start_line.source.as_ref();
            let end_source = end_line.source.as_ref();

            // Code snippet
            //
            // ```text
            //   │
            // 2 │ (+ test "")
            //   │         ^^ expected `Int` but found `String`
            //   │
            // ```

            // Write initial border
            Gutter::new(None, gutter_padding).emit(writer, config)?;
            match i {
                0 => BorderLeft::new().emit(writer, config)?,
                _ => BorderLeftBreak::new().emit(writer, config)?,
            }
            NewLine::new().emit(writer, config)?;

            // Write underlined source section
            if start_line_index == end_line_index {
                // Single line
                //
                // ```text
                // 2 │ (+ test "")
                //   │         ^^ expected `Int` but found `String`
                // ```

                let mark_start = label.range.start - start_line.start;
                let mark_end = label.range.end - start_line.start;
                let prefix_source = &start_source[..mark_start];
                let marked_source = &start_source[mark_start..mark_end];

                // Write line number and border
                Gutter::new(start_line.number, gutter_padding).emit(writer, config)?;
                BorderLeft::new().emit(writer, config)?;

                // Write line source
                write!(config.source(writer), " {}", start_source.trim_end())?;
                NewLine::new().emit(writer, config)?;

                // Write border, underline, and label
                Gutter::new(None, gutter_padding).emit(writer, config)?;
                BorderLeft::new().emit(writer, config)?;
                Underline::new(*mark_style, &prefix_source, &marked_source, &label.message)
                    .emit(writer, config)?;
                NewLine::new().emit(writer, config)?;
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

                let mark_start = label.range.start - start_line.start;
                let prefix_source = &start_source[..mark_start];

                if prefix_source.trim().is_empty() {
                    // Section is prefixed by empty space, so we don't need to take
                    // up a new line.
                    //
                    // ```text
                    // 4 │ ╭     case (mod num 5) (mod num 3) of
                    // ```

                    // Write line number, border, and underline
                    Gutter::new(start_line.number, gutter_padding).emit(writer, config)?;
                    BorderLeft::new().emit(writer, config)?;
                    UnderlineTopLeft::new(*mark_style).emit(writer, config)?;

                    // Write source line
                    write!(config.source(writer), " {}", start_source.trim_end())?;
                    NewLine::new().emit(writer, config)?;
                } else {
                    // There's source code in the prefix, so run an underline
                    // underneath it to get to the start of the range.
                    //
                    // ```text
                    // 4 │   fizz₁ num = case (mod num 5) (mod num 3) of
                    //   │ ╭─────────────^
                    // ```

                    // Write line number and border
                    Gutter::new(start_line.number, gutter_padding).emit(writer, config)?;
                    BorderLeft::new().emit(writer, config)?;

                    // Write source line
                    write!(config.source(writer), "   {}", start_source.trim_end())?;
                    NewLine::new().emit(writer, config)?;

                    // Write border and underline
                    Gutter::new(None, gutter_padding).emit(writer, config)?;
                    BorderLeft::new().emit(writer, config)?;
                    UnderlineTop::new(*mark_style, &prefix_source).emit(writer, config)?;
                    NewLine::new().emit(writer, config)?;
                }

                // Write marked lines
                //
                // ```text
                // 5 │ │     0 0 => "FizzBuzz"
                // 6 │ │     0 _ => "Fizz"
                // 7 │ │     _ 0 => "Buzz"
                // ```

                for line_index in (start_line_index + 1)..end_line_index {
                    let marked_line = line(line_index).expect("marked_line");

                    // Write line number, border, and underline
                    Gutter::new(marked_line.number, gutter_padding).emit(writer, config)?;
                    BorderLeft::new().emit(writer, config)?;
                    UnderlineLeft::new(*mark_style).emit(writer, config)?;

                    // Write marked source
                    write!(writer, " {}", marked_line.source.as_ref().trim_end())?;
                    NewLine::new().emit(writer, config)?;
                }

                // Write last marked line
                //
                // ```text
                // 8 │ │     _ _ => num
                //   │ ╰──────────────^ `case` clauses have incompatible types
                // ```

                let mark_end = label.range.end - end_line.start;
                let marked_source = &end_source[..mark_end];

                // Write line number, border, and underline
                Gutter::new(end_line.number, gutter_padding).emit(writer, config)?;
                BorderLeft::new().emit(writer, config)?;
                UnderlineLeft::new(*mark_style).emit(writer, config)?;

                // Write line source
                write!(config.source(writer), " {}", end_source.trim_end())?;
                NewLine::new().emit(writer, config)?;

                // Write border, underline, and label
                Gutter::new(None, gutter_padding).emit(writer, config)?;
                BorderLeft::new().emit(writer, config)?;
                UnderlineBottom::new(*mark_style, &marked_source, &label.message)
                    .emit(writer, config)?;
                NewLine::new().emit(writer, config)?;
            }
        }

        // Write final border
        Gutter::new(None, gutter_padding).emit(writer, config)?;
        BorderLeft::new().emit(writer, config)?;
        NewLine::new().emit(writer, config)?;

        // Additional notes
        //
        // ```text
        // = expected type `Int`
        //      found type `String`
        // ```

        for note in self.notes {
            Note::new(gutter_padding, note).emit(writer, config)?;
        }

        Ok(())
    }
}
