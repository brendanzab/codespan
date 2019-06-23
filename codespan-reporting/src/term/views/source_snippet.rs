use codespan::{ByteIndex, FileId, Files, LineIndex, Location, Span};
use std::io;
use termcolor::{ColorSpec, WriteColor};

use crate::diagnostic::{Diagnostic, Label, Severity};
use crate::term::Config;

use super::{Gutter, Locus, NewLine, Note};

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
        match self.mark_style {
            MarkStyle::Primary(severity) => config.styles.primary_label(severity),
            MarkStyle::Secondary => &config.styles.secondary_label,
        }
    }

    fn underline_char(&self, config: &Config) -> char {
        match self.mark_style {
            MarkStyle::Primary(_) => config.primary_underline_char,
            MarkStyle::Secondary => config.secondary_underline_char,
        }
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

            // Write source prefix before marked section
            let prefix_span = start_line_span.with_end(self.span.start());
            let source_prefix = self.source_slice(prefix_span, &tab).expect("source_prefix");
            write!(writer, "{}", source_prefix)?;

            // Write marked source section
            let marked_source = self.source_slice(self.span, &tab).expect("marked_source");
            writer.set_color(&label_style)?;
            write!(writer, "{}", marked_source)?;
            writer.reset()?;

            // Write source suffix after marked section
            let suffix_span = end_line_span.with_start(self.span.end());
            let source_suffix = self.source_slice(suffix_span, &tab).expect("source_suffix");
            write!(writer, "{}", source_suffix.trim_end_matches(line_trimmer))?;
            NewLine::new().emit(writer, config)?;

            // Write underline border
            Gutter::new(None, gutter_padding).emit(writer, config)?;
            BorderLeft::new().emit(writer, config)?;

            // Write underline and label
            let prefix_len = config.width(&source_prefix);
            write!(writer, "{space: >width$}", space = "", width = prefix_len)?;
            writer.set_color(&label_style)?;
            // We use `usize::max` here to ensure that we print at least one
            // underline character - even when we have a zero-length span.
            let underline_len = usize::max(config.width(&marked_source), 1);
            for _ in 0..underline_len {
                write!(writer, "{}", self.underline_char(config))?;
            }
            if !self.message.is_empty() {
                write!(writer, " {}", self.message)?;
            }
            writer.reset()?;
            NewLine::new().emit(writer, config)?;
        } else {
            // Multiple lines

            // Write line number and border
            Gutter::new(start.line.number(), gutter_padding).emit(writer, config)?;
            BorderLeft::new().emit(writer, config)?;

            // Write source prefix before marked section
            let prefix_span = start_line_span.with_end(self.span.start());
            let source_prefix = self.source_slice(prefix_span, &tab).expect("source_prefix");
            write!(writer, "{}", source_prefix)?;

            // Write marked source section
            let marked_span = start_line_span.with_start(self.span.start());
            let marked_source = self
                .source_slice(marked_span, &tab)
                .expect("marked_source_1");
            writer.set_color(&label_style)?;
            write!(writer, "{}", marked_source.trim_end_matches(line_trimmer))?;
            writer.reset()?;
            NewLine::new().emit(writer, config)?;

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
                writer.set_color(&label_style)?;
                write!(writer, "{}", marked_source.trim_end_matches(line_trimmer))?;
                writer.reset()?;
                NewLine::new().emit(writer, config)?;
            }

            // Write line number and border
            Gutter::new(end.line.number(), gutter_padding).emit(writer, config)?;
            BorderLeft::new().emit(writer, config)?;

            // Write marked source section
            let marked_span = end_line_span.with_end(self.span.end());
            let marked_source = self
                .source_slice(marked_span, &tab)
                .expect("marked_source_3");
            writer.set_color(&label_style)?;
            write!(writer, "{}", marked_source)?;
            writer.reset()?;

            // Write source suffix after marked section
            let suffix_span = end_line_span.with_start(self.span.end());
            let source_suffix = self.source_slice(suffix_span, &tab).expect("source_suffix");
            write!(writer, "{}", source_suffix.trim_end_matches(line_trimmer))?;
            NewLine::new().emit(writer, config)?;

            // Write underline border
            Gutter::new(None, gutter_padding).emit(writer, config)?;
            BorderLeft::new().emit(writer, config)?;

            // Write underline and label
            writer.set_color(&label_style)?;
            // We use `usize::max` here to ensure that we print at least one
            // underline character - even when we have a zero-length span.
            let underline_len = config.width(&marked_source) + config.width(&source_suffix);
            let underline_len = usize::max(underline_len, 1);
            for _ in 0..underline_len {
                write!(writer, "{}", self.underline_char(config))?;
            }
            if !self.message.is_empty() {
                write!(writer, " {}", self.message)?;
            }
            writer.reset()?;
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

/// The top-left corner of a source line.
struct BorderTopLeft {}

impl BorderTopLeft {
    fn new() -> BorderTopLeft {
        BorderTopLeft {}
    }

    fn emit(&self, writer: &mut impl WriteColor, config: &Config) -> io::Result<()> {
        writer.set_color(&config.styles.border)?;
        write!(writer, "{top_left}", top_left = config.border_top_left_char)?;
        writer.reset()?;

        Ok(())
    }
}

/// The top border of a source line.
struct BorderTop {
    width: usize,
}

impl BorderTop {
    fn new(width: usize) -> BorderTop {
        BorderTop { width }
    }

    fn emit(&self, writer: &mut impl WriteColor, config: &Config) -> io::Result<()> {
        writer.set_color(&config.styles.border)?;
        for _ in 0..self.width {
            write!(writer, "{top}", top = config.border_top_char)?
        }
        writer.reset()?;

        Ok(())
    }
}

/// The left-hand border of a source line.
struct BorderLeft {}

impl BorderLeft {
    fn new() -> BorderLeft {
        BorderLeft {}
    }

    fn emit(&self, writer: &mut impl WriteColor, config: &Config) -> io::Result<()> {
        writer.set_color(&config.styles.border)?;
        write!(writer, "{left} ", left = config.border_left_char)?;
        writer.reset()?;

        Ok(())
    }
}
