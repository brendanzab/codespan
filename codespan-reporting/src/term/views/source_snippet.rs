use codespan::{FileId, Files, LineIndex, Span};
use std::io;
use termcolor::WriteColor;

use crate::diagnostic::Label;
use crate::term::Config;

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
pub struct SourceSnippet<'a> {
    file_id: FileId,
    spans: Vec<(&'a Label, MarkStyle)>,
    notes: &'a [String],
}

impl<'a> SourceSnippet<'a> {
    pub fn new(file_id: FileId, spans: Vec<(&'a Label, MarkStyle)>, notes: &'a [String]) -> Self {
        SourceSnippet {
            file_id,
            spans,
            notes,
        }
    }

    fn source_locus_spans(&self) -> (Span, Span) {
        let mut source_span = None;
        let mut locus_span = None;

        for (label, mark_style) in &self.spans {
            source_span =
                Some(source_span.map_or(label.span, |span| Span::merge(span, label.span)));
            if let MarkStyle::Primary(_) = mark_style {
                locus_span =
                    Some(locus_span.map_or(label.span, |span| Span::merge(span, label.span)));
            }
        }

        let source_span = source_span.unwrap_or(Span::initial());
        let locus_span = locus_span.unwrap_or(source_span);

        (source_span, locus_span)
    }

    pub fn emit(
        &self,
        files: &'a Files<impl AsRef<str>>,
        writer: &mut (impl WriteColor + ?Sized),
        config: &Config,
    ) -> io::Result<()> {
        use std::io::Write;

        // NOTE: All of the things we need with `files` is done here. Could this
        // help us decide on a decent trait for the file provider?
        let file_name = files.name(self.file_id);
        let location = |byte_index| files.location(self.file_id, byte_index);
        let get_line = |line_index| {
            // NOTE: We could simplify this into a single `get_line` method
            let span = files.line_span(self.file_id, line_index).ok()?;
            let source = files.source_slice(self.file_id, span).ok()?;
            Some((span, source))
        };

        let (source_span, locus_span) = self.source_locus_spans();
        let locus_start = location(locus_span.start()).expect("locus_span_start");
        let source_end = location(source_span.end()).expect("source_span_end");

        // Use the length of the last line number as the gutter padding
        let gutter_padding = count_digits(source_end.line.number().to_usize());

        // Top left border and locus.
        //
        // ```text
        // ┌── test:2:9 ───
        // ```

        Gutter::new(None, gutter_padding).emit(writer, config)?;
        BorderTopLeft::new().emit(writer, config)?;
        BorderTop::new(2).emit(writer, config)?;
        write!(writer, " ")?;

        Locus::new(file_name, locus_start).emit(writer, config)?;

        write!(writer, " ")?;
        BorderTop::new(3).emit(writer, config)?;
        NewLine::new().emit(writer, config)?;

        // TODO: Better grouping
        for (i, (label, mark_style)) in self.spans.iter().enumerate() {
            let start = location(label.span.start()).expect("location_start");
            let end = location(label.span.end()).expect("location_end");
            let (start_line_span, start_line) = get_line(start.line).expect("start_line_span");
            let (end_line_span, end_line) = get_line(end.line).expect("end_line_span");

            let label_style = mark_style.label_style(config);

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
            if start.line == end.line {
                // Single line
                //
                // ```text
                // 2 │ (+ test "")
                //   │         ^^ expected `Int` but found `String`
                // ```

                let highlight_start = (label.span.start() - start_line_span.start()).to_usize();
                let highlight_end = (label.span.end() - start_line_span.start()).to_usize();
                let prefix_source = &start_line[..highlight_start];
                let highlighted_source = &start_line[highlight_start..highlight_end];
                let suffix_source = &start_line[highlight_end..];

                // Write line number and border
                Gutter::new(start.line.number(), gutter_padding).emit(writer, config)?;
                BorderLeft::new().emit(writer, config)?;

                // Write line source
                write!(config.source(writer), " {}", prefix_source)?;
                writer.set_color(label_style)?;
                write!(config.source(writer), "{}", highlighted_source)?;
                writer.reset()?;
                write!(config.source(writer), "{}", suffix_source.trim_end())?;
                NewLine::new().emit(writer, config)?;

                // Write border, underline, and label
                Gutter::new(None, gutter_padding).emit(writer, config)?;
                BorderLeft::new().emit(writer, config)?;
                Underline::new(
                    *mark_style,
                    &prefix_source,
                    &highlighted_source,
                    &label.message,
                )
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

                let highlight_start = (label.span.start() - start_line_span.start()).to_usize();
                let prefix_source = &start_line[..highlight_start];
                let highlighted_source = &start_line[highlight_start..];

                if prefix_source.trim().is_empty() {
                    // Section is prefixed by empty space, so we don't need to take
                    // up a new line.
                    //
                    // ```text
                    // 4 │ ╭     case (mod num 5) (mod num 3) of
                    // ```

                    // Write line number, border, and underline
                    Gutter::new(start.line.number(), gutter_padding).emit(writer, config)?;
                    BorderLeft::new().emit(writer, config)?;
                    UnderlineTopLeft::new(*mark_style).emit(writer, config)?;

                    // Write source line
                    write!(config.source(writer), " {}", prefix_source)?;
                    writer.set_color(&label_style)?;
                    write!(config.source(writer), "{}", highlighted_source.trim_end())?;
                    writer.reset()?;
                    NewLine::new().emit(writer, config)?;
                } else {
                    // There's source code in the prefix, so run an underline
                    // underneath it to get to the start of the span.
                    //
                    // ```text
                    // 4 │   fizz₁ num = case (mod num 5) (mod num 3) of
                    //   │ ╭─────────────^
                    // ```

                    // Write line number and border
                    Gutter::new(start.line.number(), gutter_padding).emit(writer, config)?;
                    BorderLeft::new().emit(writer, config)?;

                    // Write source line
                    write!(config.source(writer), "   {}", prefix_source)?;
                    writer.set_color(&label_style)?;
                    write!(config.source(writer), "{}", highlighted_source.trim_end())?;
                    writer.reset()?;
                    NewLine::new().emit(writer, config)?;

                    // Write border and underline
                    Gutter::new(None, gutter_padding).emit(writer, config)?;
                    BorderLeft::new().emit(writer, config)?;
                    UnderlineTop::new(*mark_style, &prefix_source).emit(writer, config)?;
                    NewLine::new().emit(writer, config)?;
                }

                // Write highlighted lines
                //
                // ```text
                // 5 │ │     0 0 => "FizzBuzz"
                // 6 │ │     0 _ => "Fizz"
                // 7 │ │     _ 0 => "Buzz"
                // ```

                for line_index in ((start.line.to_usize() + 1)..end.line.to_usize())
                    .map(|i| LineIndex::from(i as u32))
                {
                    let (_, highlighted_source) =
                        get_line(line_index).expect("highlighted_source_2");

                    // Write line number, border, and underline
                    Gutter::new(line_index.number(), gutter_padding).emit(writer, config)?;
                    BorderLeft::new().emit(writer, config)?;
                    UnderlineLeft::new(*mark_style).emit(writer, config)?;

                    // Write highlighted source
                    writer.set_color(label_style)?;
                    write!(writer, " {}", highlighted_source.trim_end())?;
                    writer.reset()?;
                    NewLine::new().emit(writer, config)?;
                }

                // Write last highlighted line
                //
                // ```text
                // 8 │ │     _ _ => num
                //   │ ╰──────────────^ `case` clauses have incompatible types
                // ```

                let highlight_end = (label.span.end() - end_line_span.start()).to_usize();
                let highlighted_source = &end_line[..highlight_end];
                let suffix_source = &end_line[highlight_end..];

                // Write line number, border, and underline
                Gutter::new(end.line.number(), gutter_padding).emit(writer, config)?;
                BorderLeft::new().emit(writer, config)?;
                UnderlineLeft::new(*mark_style).emit(writer, config)?;

                // Write line source
                writer.set_color(label_style)?;
                write!(config.source(writer), " {}", highlighted_source)?;
                writer.reset()?;
                write!(config.source(writer), "{}", suffix_source.trim_end())?;
                NewLine::new().emit(writer, config)?;

                // Write border, underline, and label
                Gutter::new(None, gutter_padding).emit(writer, config)?;
                BorderLeft::new().emit(writer, config)?;
                UnderlineBottom::new(*mark_style, &highlighted_source, &label.message)
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
