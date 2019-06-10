use codespan::{ByteIndex, Files, LineIndex, Location, Span};
use std::io;
use termcolor::{Color, ColorSpec, WriteColor};

use crate::emitter::Config;
use crate::{Diagnostic, Label, Severity};

use super::{BorderLeft, BorderTop, BorderTopLeft, Gutter, Locus, NewLine, Note};

enum MarkStyle {
    Primary(Severity),
    Secondary,
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
    label: &'a Label,
    mark_style: MarkStyle,
    notes: &'a [String],
}

impl<'a> SourceSnippet<'a> {
    pub fn new_primary(files: &'a Files, diagnostic: &'a Diagnostic) -> SourceSnippet<'a> {
        SourceSnippet {
            files,
            label: &diagnostic.primary_label,
            mark_style: MarkStyle::Primary(diagnostic.severity),
            notes: &diagnostic.notes,
        }
    }

    pub fn new_secondary(files: &'a Files, label: &'a Label) -> SourceSnippet<'a> {
        SourceSnippet {
            files,
            label,
            mark_style: MarkStyle::Secondary,
            notes: &[],
        }
    }

    fn span(&self) -> Span {
        self.label.span
    }

    fn start(&self) -> ByteIndex {
        self.span().start()
    }

    fn end(&self) -> ByteIndex {
        self.span().end()
    }

    fn file_name(&self) -> &'a str {
        self.files.name(self.label.file_id)
    }

    fn location(&self, byte_index: ByteIndex) -> Result<Location, impl std::error::Error> {
        self.files.location(self.label.file_id, byte_index)
    }

    fn source_slice(&self, span: Span, tab: &'a str) -> Result<String, impl std::error::Error> {
        // NOTE: Not sure if we can do this more efficiently? Perhaps a custom
        // writer might be better?
        self.files
            .source_slice(self.label.file_id, span)
            .map(|s| s.replace('\t', tab))
    }

    fn line_span(&self, line_index: LineIndex) -> Result<Span, impl std::error::Error> {
        self.files.line_span(self.label.file_id, line_index)
    }

    fn label_color(&self, config: &Config) -> Color {
        match self.mark_style {
            MarkStyle::Primary(severity) => config.severity_color(severity),
            MarkStyle::Secondary => config.secondary_color,
        }
    }

    fn underline_char(&self, config: &Config) -> char {
        match self.mark_style {
            MarkStyle::Primary(_) => config.primary_underline_char,
            MarkStyle::Secondary => config.secondary_underline_char,
        }
    }

    pub fn emit(&self, writer: &mut impl WriteColor, config: &Config) -> io::Result<()> {
        let label_spec = ColorSpec::new()
            .set_fg(Some(self.label_color(config)))
            .clone();

        let start = self.location(self.start()).expect("location_start");
        let end = self.location(self.end()).expect("location_end");
        let start_line_span = self.line_span(start.line).expect("start_line_span");
        let end_line_span = self.line_span(end.line).expect("end_line_span");

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

        // SourceSnippet code snippet
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

        // Write line number and border
        Gutter::new(start.line.number(), gutter_padding).emit(writer, config)?;
        BorderLeft::new().emit(writer, config)?;

        let line_trimmer = |ch: char| ch == '\r' || ch == '\n';

        // Write source prefix before marked section
        let prefix_span = start_line_span.with_end(self.start());
        let source_prefix = self.source_slice(prefix_span, &tab).expect("source_prefix");
        write!(writer, "{}", source_prefix)?;

        // Write marked section
        let mark_len = if start.line == end.line {
            // Single line

            // Write marked source section
            let marked_source = self.source_slice(self.span(), &tab).expect("marked_source");
            writer.set_color(&label_spec)?;
            write!(writer, "{}", marked_source)?;
            writer.reset()?;
            config.width(&marked_source)
        } else {
            // Multiple lines

            // Write marked source section
            let marked_span = start_line_span.with_start(self.start());
            let marked_source = self
                .source_slice(marked_span, &tab)
                .expect("marked_source_1");
            writer.set_color(&label_spec)?;
            write!(writer, "{}", marked_source)?;

            for line_index in ((start.line.to_usize() + 1)..end.line.to_usize())
                .map(|i| LineIndex::from(i as u32))
            {
                // Write line number and border
                Gutter::new(line_index.number(), gutter_padding).emit(writer, config)?;
                BorderLeft::new().emit(writer, config)?;

                // Write marked source section
                let marked_span = self.line_span(line_index).expect("marked_span");
                let marked_source = self
                    .source_slice(marked_span, &tab)
                    .expect("marked_source_2");
                writer.set_color(&label_spec)?;
                write!(writer, "{}", marked_source.trim_end_matches(line_trimmer))?;
                NewLine::new().emit(writer, config)?;
            }

            // Write line number and border
            Gutter::new(end.line.number(), gutter_padding).emit(writer, config)?;
            BorderLeft::new().emit(writer, config)?;

            // Write marked source section
            let marked_span = end_line_span.with_end(self.end());
            let marked_source = self
                .source_slice(marked_span, &tab)
                .expect("marked_source_3");
            writer.set_color(&label_spec)?;
            write!(writer, "{}", marked_source)?;
            writer.reset()?;
            config.width(&marked_source)
        };

        // Write source suffix after marked section
        let suffix_span = end_line_span.with_start(self.end());
        let source_suffix = self.source_slice(suffix_span, &tab).expect("source_suffix");
        write!(writer, "{}", source_suffix.trim_end_matches(line_trimmer))?;
        NewLine::new().emit(writer, config)?;

        // Write underline border
        Gutter::new(None, gutter_padding).emit(writer, config)?;
        BorderLeft::new().emit(writer, config)?;

        // Write underline and label
        writer.set_color(&label_spec)?;
        write!(
            writer,
            "{space: >width$}",
            space = "",
            width = config.width(&source_prefix),
        )?;
        for _ in 0..mark_len {
            write!(writer, "{}", self.underline_char(config))?;
        }
        if !self.label.message.is_empty() {
            write!(writer, " {}", self.label.message)?;
        }
        NewLine::new().emit(writer, config)?;
        writer.reset()?;

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
