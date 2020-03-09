use std::io;
use std::ops::Range;
use termcolor::WriteColor;

use crate::files::Files;
use crate::term::Config;

use super::{
    BorderLeft, BorderLeftBreak, BorderTop, BorderTopLeft, Gutter, Locus, MarkStyle, NewLine,
    Underline, UnderlineBottom, UnderlineLeft, UnderlineTop, UnderlineTopLeft,
};

pub struct MarkGroup<'a, Origin> {
    pub origin: Origin,
    pub range: Range<usize>,
    pub marks: Vec<Mark<'a>>,
}

pub struct Mark<'a> {
    pub style: MarkStyle,
    pub range: Range<usize>,
    pub message: &'a str,
}

/// An underlined snippet of source code.
///
/// ```text
///   ┌── test:2:9 ───
///   │
/// 2 │ (+ test "")
///   │         ^^ expected `Int` but found `String`
///   │
/// ```
pub struct SourceSnippet<'a, 'files, F: Files<'files>> {
    gutter_padding: usize,
    file_id: F::FileId,
    mark_group: MarkGroup<'a, F::Origin>,
}

impl<'a, 'files: 'a, F: Files<'files>> SourceSnippet<'a, 'files, F> {
    pub fn new(
        gutter_padding: usize,
        file_id: F::FileId,
        mark_group: MarkGroup<'a, F::Origin>,
    ) -> SourceSnippet<'a, 'files, F> {
        SourceSnippet {
            gutter_padding,
            file_id,
            mark_group,
        }
    }

    pub fn emit(
        &self,
        files: &'files F,
        writer: &mut (impl WriteColor + ?Sized),
        config: &Config,
    ) -> io::Result<()> {
        use std::io::Write;

        let source = files.source(self.file_id).expect("source");
        let line_index = |byte_index| files.line_index(self.file_id, byte_index);
        let line = |line_index| files.line(self.file_id, line_index);

        // Top left border and locus.
        //
        // ```text
        // ┌── test:2:9 ───
        // ```

        Gutter::new(None, self.gutter_padding).emit(writer, config)?;
        BorderTopLeft::new().emit(writer, config)?;
        BorderTop::new(2).emit(writer, config)?;
        write!(writer, " ")?;

        {
            let origin = &self.mark_group.origin;
            let start = self.mark_group.range.start;
            let line_index = line_index(start).expect("locus_line_index");
            let line = line(line_index).expect("locus_line");
            let line_source = &source.as_ref()[line.range.clone()];

            Locus::new(&origin, line.number, line.column_number(line_source, start))
                .emit(writer, config)?;
        }

        write!(writer, " ")?;
        BorderTop::new(3).emit(writer, config)?;
        NewLine::new().emit(writer, config)?;

        for (i, mark) in self.mark_group.marks.iter().enumerate() {
            let start_line_index = line_index(mark.range.start).expect("start_line_index");
            let end_line_index = line_index(mark.range.end).expect("end_line_index");
            let start_line = line(start_line_index).expect("start_line");
            let end_line = line(end_line_index).expect("end_line");

            let start_source = &source.as_ref()[start_line.range.clone()];
            let end_source = &source.as_ref()[end_line.range.clone()];

            // Code snippet
            //
            // ```text
            //   │
            // 2 │ (+ test "")
            //   │         ^^ expected `Int` but found `String`
            //   │
            // ```

            // Write initial border
            Gutter::new(None, self.gutter_padding).emit(writer, config)?;
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

                let mark_start = mark.range.start - start_line.range.start;
                let mark_end = mark.range.end - start_line.range.start;
                let prefix_source = &start_source[..mark_start];
                let marked_source = &start_source[mark_start..mark_end];

                // Write line number and border
                Gutter::new(start_line.number, self.gutter_padding).emit(writer, config)?;
                BorderLeft::new().emit(writer, config)?;

                // Write line source
                write!(config.source(writer), " {}", start_source.trim_end())?;
                NewLine::new().emit(writer, config)?;

                // Write border, underline, and label
                Gutter::new(None, self.gutter_padding).emit(writer, config)?;
                BorderLeft::new().emit(writer, config)?;
                Underline::new(mark.style, &prefix_source, &marked_source, &mark.message)
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

                let mark_start = mark.range.start - start_line.range.start;
                let prefix_source = &start_source[..mark_start];

                if prefix_source.trim().is_empty() {
                    // Section is prefixed by empty space, so we don't need to take
                    // up a new line.
                    //
                    // ```text
                    // 4 │ ╭     case (mod num 5) (mod num 3) of
                    // ```

                    // Write line number, border, and underline
                    Gutter::new(start_line.number, self.gutter_padding).emit(writer, config)?;
                    BorderLeft::new().emit(writer, config)?;
                    UnderlineTopLeft::new(mark.style).emit(writer, config)?;

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
                    Gutter::new(start_line.number, self.gutter_padding).emit(writer, config)?;
                    BorderLeft::new().emit(writer, config)?;

                    // Write source line
                    write!(config.source(writer), "   {}", start_source.trim_end())?;
                    NewLine::new().emit(writer, config)?;

                    // Write border and underline
                    Gutter::new(None, self.gutter_padding).emit(writer, config)?;
                    BorderLeft::new().emit(writer, config)?;
                    UnderlineTop::new(mark.style, &prefix_source).emit(writer, config)?;
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
                    let marked_source = &source
                        .as_ref()
                        .get(marked_line.range.clone())
                        .expect("marked_source");

                    // Write line number, border, and underline
                    Gutter::new(marked_line.number, self.gutter_padding).emit(writer, config)?;
                    BorderLeft::new().emit(writer, config)?;
                    UnderlineLeft::new(mark.style).emit(writer, config)?;

                    // Write marked source
                    write!(writer, " {}", marked_source.trim_end())?;
                    NewLine::new().emit(writer, config)?;
                }

                // Write last marked line
                //
                // ```text
                // 8 │ │     _ _ => num
                //   │ ╰──────────────^ `case` clauses have incompatible types
                // ```

                let mark_end = mark.range.end - end_line.range.start;
                let marked_source = &end_source[..mark_end];

                // Write line number, border, and underline
                Gutter::new(end_line.number, self.gutter_padding).emit(writer, config)?;
                BorderLeft::new().emit(writer, config)?;
                UnderlineLeft::new(mark.style).emit(writer, config)?;

                // Write line source
                write!(config.source(writer), " {}", end_source.trim_end())?;
                NewLine::new().emit(writer, config)?;

                // Write border, underline, and label
                Gutter::new(None, self.gutter_padding).emit(writer, config)?;
                BorderLeft::new().emit(writer, config)?;
                UnderlineBottom::new(mark.style, &marked_source, &mark.message)
                    .emit(writer, config)?;
                NewLine::new().emit(writer, config)?;
            }
        }

        // Write final border
        Gutter::new(None, self.gutter_padding).emit(writer, config)?;
        BorderLeft::new().emit(writer, config)?;
        NewLine::new().emit(writer, config)?;

        Ok(())
    }
}
