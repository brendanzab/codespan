use codespan::{ByteIndex, FileId, Files, LineIndex, Location, Span};
use std::io;
use termcolor::{ColorSpec, WriteColor};

use crate::diagnostic::{Diagnostic, Label};
use crate::term::Config;

use super::{Locus, NewLine};

mod border;
mod gutter;
mod note;
mod underline;

use self::border::{BorderLeft, BorderTop, BorderTopLeft};
use self::gutter::Gutter;
use self::note::Note;
use self::underline::{
    MarkStyle, Underline, UnderlineBottom, UnderlineLeft, UnderlineTop, UnderlineTopLeft,
};

impl MarkStyle {
    fn label_style<'config>(self, config: &'config Config) -> &'config ColorSpec {
        match self {
            MarkStyle::Primary(severity) => config.styles.primary_label(severity),
            MarkStyle::Secondary => &config.styles.secondary_label,
        }
    }

    fn caret_char(self, config: &Config) -> char {
        match self {
            MarkStyle::Primary(_) => config.primary_caret_char,
            MarkStyle::Secondary => config.secondary_caret_char,
        }
    }

    fn multiline_caret_char(self, config: &Config) -> char {
        match self {
            MarkStyle::Primary(_) => config.multiline_primary_caret_char,
            MarkStyle::Secondary => config.multiline_secondary_caret_char,
        }
    }
}

/// A marked section of source code.
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
    span: Span,
    message: &'a str,
    mark_style: MarkStyle,
    notes: &'a [String],
}

impl<'a> SourceSnippet<'a> {
    pub fn new_primary(files: &'a Files, diagnostic: &'a Diagnostic) -> SourceSnippet<'a> {
        SourceSnippet {
            files,
            file_id: diagnostic.primary_label.file_id,
            span: diagnostic.primary_label.span,
            message: &diagnostic.primary_label.message,
            mark_style: MarkStyle::Primary(diagnostic.severity),
            notes: &diagnostic.notes,
        }
    }

    pub fn new_secondary(files: &'a Files, label: &'a Label) -> SourceSnippet<'a> {
        SourceSnippet {
            files,
            file_id: label.file_id,
            span: label.span,
            message: &label.message,
            mark_style: MarkStyle::Secondary,
            notes: &[],
        }
    }

    fn file_name(&self) -> &'a str {
        self.files.name(self.file_id)
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

    fn label_style<'config>(&self, config: &'config Config) -> &'config ColorSpec {
        self.mark_style.label_style(config)
    }

    pub fn emit(&self, writer: &mut impl WriteColor, config: &Config) -> io::Result<()> {
        let start = self.location(self.span.start()).expect("location_start");
        let end = self.location(self.span.end()).expect("location_end");
        let start_line_span = self.line_span(start.line).expect("start_line_span");
        let end_line_span = self.line_span(end.line).expect("end_line_span");
        let is_multiline = start.line != end.line;

        let label_style = self.label_style(config);
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
        BorderLeft::new().emit(writer, config)?;
        NewLine::new().emit(writer, config)?;

        let line_trimmer = |ch: char| ch == '\r' || ch == '\n';

        // Write marked section
        if !is_multiline {
            // Single line
            //
            // ```text
            // 2 │ (+ test "")
            //   │         ^^ expected `Int` but found `String`
            // ```

            // Write line number and border
            Gutter::new(start.line.number(), gutter_padding).emit(writer, config)?;
            BorderLeft::new().emit(writer, config)?;
            write!(writer, " ")?;

            // Write source prefix before marked section
            let prefix_span = start_line_span.with_end(self.span.start());
            let source_prefix = self.source_slice(prefix_span, &tab).expect("source_prefix");
            write!(writer, "{}", source_prefix)?;

            // Write marked source section
            let marked_source = self.source_slice(self.span, &tab).expect("marked_source");
            writer.set_color(label_style)?;
            write!(writer, "{}", marked_source)?;
            writer.reset()?;

            // Write source suffix after marked section
            let suffix_span = end_line_span.with_start(self.span.end());
            let source_suffix = self.source_slice(suffix_span, &tab).expect("source_suffix");
            write!(writer, "{}", source_suffix.trim_end_matches(line_trimmer))?;
            NewLine::new().emit(writer, config)?;

            // Write border, underline, and label
            Gutter::new(None, gutter_padding).emit(writer, config)?;
            BorderLeft::new().emit(writer, config)?;
            Underline::new(
                self.mark_style,
                &source_prefix,
                &marked_source,
                self.message,
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

            // Write line number and border
            Gutter::new(start.line.number(), gutter_padding).emit(writer, config)?;
            BorderLeft::new().emit(writer, config)?;

            let prefix_span = start_line_span.with_end(self.span.start());
            let source_prefix = self.source_slice(prefix_span, &tab).expect("source_prefix");
            let marked_span = start_line_span.with_start(self.span.start());
            let marked_source = self
                .source_slice(marked_span, &tab)
                .expect("marked_source_1");

            if source_prefix.trim().is_empty() {
                // Section is prefixed by empty space, so we don't need to take
                // up a new line.
                //
                // ```text
                // 4 │ ╭     case (mod num 5) (mod num 3) of
                // ```

                // Write underline
                UnderlineTopLeft::new(self.mark_style).emit(writer, config)?;

                // Write source prefix before marked section
                write!(writer, " {}", source_prefix)?;

                // Write marked source section
                writer.set_color(&label_style)?;
                write!(writer, "{}", marked_source.trim_end_matches(line_trimmer))?;
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

                // Write source prefix before marked section
                write!(writer, "   {}", source_prefix)?;

                // Write marked source section
                writer.set_color(&label_style)?;
                write!(writer, "{}", marked_source.trim_end_matches(line_trimmer))?;
                writer.reset()?;
                NewLine::new().emit(writer, config)?;

                // Write border and underline
                Gutter::new(None, gutter_padding).emit(writer, config)?;
                BorderLeft::new().emit(writer, config)?;
                UnderlineTop::new(self.mark_style, &source_prefix).emit(writer, config)?;
                NewLine::new().emit(writer, config)?;
            }

            // Write marked lines
            //
            // ```text
            // 5 │ │     0 0 => "FizzBuzz"
            // 6 │ │     0 _ => "Fizz"
            // 7 │ │     _ 0 => "Buzz"
            // ```
            for line_index in ((start.line.to_usize() + 1)..end.line.to_usize())
                .map(|i| LineIndex::from(i as u32))
            {
                // Write line number, border, and underline
                Gutter::new(line_index.number(), gutter_padding).emit(writer, config)?;
                BorderLeft::new().emit(writer, config)?;
                UnderlineLeft::new(self.mark_style).emit(writer, config)?;

                // Write marked source section
                let marked_span = self.line_span(line_index).expect("marked_span");
                let marked_source = self
                    .source_slice(marked_span, &tab)
                    .expect("marked_source_2");
                writer.set_color(label_style)?;
                write!(writer, " {}", marked_source.trim_end_matches(line_trimmer))?;
                writer.reset()?;
                NewLine::new().emit(writer, config)?;
            }

            // Write last marked line
            //
            // ```text
            // 8 │ │     _ _ => num
            //   │ ╰──────────────^ `case` clauses have incompatible types
            // ```

            // Write line number, border, and underline
            Gutter::new(end.line.number(), gutter_padding).emit(writer, config)?;
            BorderLeft::new().emit(writer, config)?;
            UnderlineLeft::new(self.mark_style).emit(writer, config)?;

            // Write marked source section
            let marked_span = end_line_span.with_end(self.span.end());
            let marked_source = self
                .source_slice(marked_span, &tab)
                .expect("marked_source_3");
            writer.set_color(label_style)?;
            write!(writer, " {}", marked_source)?;
            writer.reset()?;

            // Write source suffix after marked section
            let suffix_span = end_line_span.with_start(self.span.end());
            let source_suffix = self.source_slice(suffix_span, &tab).expect("source_suffix");
            write!(writer, "{}", source_suffix.trim_end_matches(line_trimmer))?;
            NewLine::new().emit(writer, config)?;

            // Write border, underline, and label
            Gutter::new(None, gutter_padding).emit(writer, config)?;
            BorderLeft::new().emit(writer, config)?;
            UnderlineBottom::new(self.mark_style, &marked_source, self.message)
                .emit(writer, config)?;
            NewLine::new().emit(writer, config)?;
        };

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
