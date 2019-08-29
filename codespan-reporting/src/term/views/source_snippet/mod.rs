use codespan::{ByteIndex, FileId, Files, LineIndex, Location, Span};
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
    files: &'a Files,
    file_id: FileId,
    spans: Vec<(&'a Label, MarkStyle)>,
    notes: &'a [String],
}

impl<'a> SourceSnippet<'a> {
    pub fn new(
        files: &'a Files,
        file_id: FileId,
        spans: Vec<(&'a Label, MarkStyle)>,
        notes: &'a [String],
    ) -> SourceSnippet<'a> {
        SourceSnippet {
            files,
            file_id,
            spans,
            notes,
        }
    }

    fn file_name(&self) -> &'a str {
        self.files.name(self.file_id)
    }

    fn span(&self) -> Span {
        let span = self.spans.iter().fold(None::<Span>, |acc, (label, _)| {
            Some(acc.map_or(label.span, |acc| {
                let start = std::cmp::min(acc.start(), label.span.start());
                let end = std::cmp::max(acc.end(), label.span.end());
                Span::new(start, end)
            }))
        });
        span.unwrap_or(Span::initial())
    }

    fn location(&self, byte_index: ByteIndex) -> Result<Location, impl std::error::Error> {
        self.files.location(self.file_id, byte_index)
    }

    fn source_slice(&self, span: Span, tab: &'a str) -> Result<String, impl std::error::Error> {
        // NOTE: Not sure if we can do this more efficiently? Perhaps a custom
        // writer might be better?
        self.files
            .source_slice(self.file_id, span)
            .map(|s| s.replace('\t', tab))
    }

    fn line_span(&self, line_index: LineIndex) -> Result<Span, impl std::error::Error> {
        self.files.line_span(self.file_id, line_index)
    }

    pub fn emit(&self, writer: &mut impl WriteColor, config: &Config) -> io::Result<()> {
        let span = self.span();
        let start = self.location(span.start()).expect("location_start");
        let end = self.location(span.end()).expect("location_end");

        // Use the length of the last line number as the gutter padding
        let gutter_padding = format!("{}", end.line.number()).len();
        // Cache the tabs we'll be using to pad the source strings.
        let tab = config.tab_padding();

        // Top left border and locus.
        //
        // ```text
        // ┌── test:2:9 ───
        // ```

        Gutter::new(None, gutter_padding).emit(writer, config)?;
        BorderTopLeft::new().emit(writer, config)?;
        BorderTop::new(2).emit(writer, config)?;
        write!(writer, " ")?;

        Locus::new(self.file_name(), start).emit(writer, config)?;

        write!(writer, " ")?;
        BorderTop::new(3).emit(writer, config)?;
        NewLine::new().emit(writer, config)?;

        // TODO: Better grouping
        for (i, (label, mark_style)) in self.spans.iter().enumerate() {
            let start = self.location(label.span.start()).expect("location_start");
            let end = self.location(label.span.end()).expect("location_end");
            let start_line_span = self.line_span(start.line).expect("start_line_span");
            let end_line_span = self.line_span(end.line).expect("end_line_span");

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

                let prefix_source = {
                    let span = Span::new(start_line_span.start(), label.span.start());
                    self.source_slice(span, &tab).expect("prefix_source")
                };
                let highlighted_source = {
                    let span = label.span;
                    self.source_slice(span, &tab).expect("highlighted_source")
                };
                let suffix_source = {
                    let span = Span::new(label.span.end(), end_line_span.end());
                    self.source_slice(span, &tab).expect("suffix_source")
                };

                // Write line number and border
                Gutter::new(start.line.number(), gutter_padding).emit(writer, config)?;
                BorderLeft::new().emit(writer, config)?;

                // Write line source
                write!(writer, " {}", prefix_source)?;
                writer.set_color(label_style)?;
                write!(writer, "{}", highlighted_source)?;
                writer.reset()?;
                write!(writer, "{}", suffix_source.trim_end())?;
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

                let prefix_source = {
                    let span = Span::new(start_line_span.start(), label.span.start());
                    self.source_slice(span, &tab).expect("prefix_source")
                };
                let highlighted_source = {
                    let span = Span::new(label.span.start(), start_line_span.end());
                    self.source_slice(span, &tab).expect("highlighted_source_1")
                };

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
                    write!(writer, " {}", prefix_source)?;
                    writer.set_color(&label_style)?;
                    write!(writer, "{}", highlighted_source.trim_end())?;
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
                    write!(writer, "   {}", prefix_source)?;
                    writer.set_color(&label_style)?;
                    write!(writer, "{}", highlighted_source.trim_end())?;
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
                    let highlighted_source = {
                        let span = self.line_span(line_index).expect("highlighted_span");
                        self.source_slice(span, &tab).expect("highlighted_source_2")
                    };

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
                let highlighted_source = {
                    let span = Span::new(end_line_span.start(), label.span.end());
                    self.source_slice(span, &tab).expect("highlighted_source_3")
                };
                let suffix_source = {
                    let span = Span::new(label.span.end(), end_line_span.end());
                    self.source_slice(span, &tab).expect("suffix_source")
                };

                // Write line number, border, and underline
                Gutter::new(end.line.number(), gutter_padding).emit(writer, config)?;
                BorderLeft::new().emit(writer, config)?;
                UnderlineLeft::new(*mark_style).emit(writer, config)?;

                // Write line source
                writer.set_color(label_style)?;
                write!(writer, " {}", highlighted_source)?;
                writer.reset()?;
                write!(writer, "{}", suffix_source.trim_end())?;
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
