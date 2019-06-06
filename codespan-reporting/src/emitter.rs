use codespan::{ByteIndex, Files, LineIndex, LineNumber, Location, Span};
use std::io;
use termcolor::{Color, ColorSpec, WriteColor};
use unicode_width::UnicodeWidthStr;

use crate::{Diagnostic, Label, Severity};

/// Configures how a diagnostic is rendered.
#[derive(Clone, Debug)]
pub struct Config {
    /// The color to use when rendering bugs. Defaults to `Color::Red`.
    pub bug_color: Color,
    /// The color to use when rendering errors. Defaults to `Color::Red`.
    pub error_color: Color,
    /// The color to use when rendering warnings. Defaults to `Color::Yellow`.
    pub warning_color: Color,
    /// The color to use when rendering notes. Defaults to `Color::Green`.
    pub note_color: Color,
    /// The color to use when rendering helps. Defaults to `Color::Cyan`.
    pub help_color: Color,
    /// The color to use when rendering secondary labels. Defaults to
    /// `Color::Blue` (or `Color::Cyan` on windows).
    pub secondary_color: Color,
    /// The color to use when rendering the line numbers. Defaults to
    /// `Color::Blue` (or `Color::Cyan` on windows).
    pub line_number_color: Color,
    /// The color to use when rendering the source code borders. Defaults to
    /// `Color::Blue` (or `Color::Cyan` on windows).
    pub border_color: Color,
    /// The color to use when rendering the note bullets. Defaults to
    /// `Color::Blue` (or `Color::Cyan` on windows).
    pub note_bullet_color: Color,
    /// The character to use when marking the top-left corner of the source.
    /// Defaults to: `┌`.
    pub border_top_left_char: char,
    /// The character to use when marking the top border of the source.
    /// Defaults to: `─`.
    pub border_top_char: char,
    /// The character to use when marking the left border of the source.
    /// Defaults to: `│`.
    pub border_left_char: char,
    /// The character to use when underlining a primary label. Defaults to: `^`.
    pub primary_underline_char: char,
    /// The character to use when underlining a secondary label. Defaults to: `-`.
    pub secondary_underline_char: char,
    /// The character to use for the note bullet. Defaults to: `=`.
    pub note_bullet_char: char,
}

impl Default for Config {
    fn default() -> Config {
        // Blue is really difficult to see on the standard windows command line
        #[cfg(windows)]
        const BLUE: Color = Color::Cyan;
        #[cfg(not(windows))]
        const BLUE: Color = Color::Blue;

        Config {
            bug_color: Color::Red,
            error_color: Color::Red,
            warning_color: Color::Yellow,
            note_color: Color::Green,
            help_color: Color::Cyan,
            secondary_color: BLUE,
            line_number_color: BLUE,
            border_color: BLUE,
            note_bullet_color: BLUE,
            border_top_left_char: '┌',
            border_top_char: '─',
            border_left_char: '│',
            primary_underline_char: '^',
            secondary_underline_char: '-',
            note_bullet_char: '=',
        }
    }
}

impl Config {
    /// The color used to mark a given severity.
    pub fn severity_color(&self, severity: Severity) -> Color {
        match severity {
            Severity::Bug => self.bug_color,
            Severity::Error => self.error_color,
            Severity::Warning => self.warning_color,
            Severity::Note => self.note_color,
            Severity::Help => self.help_color,
        }
    }
}

pub fn emit(
    mut writer: impl WriteColor,
    config: &Config,
    files: &Files,
    diagnostic: &Diagnostic,
) -> io::Result<()> {
    Header::new(diagnostic).emit(&mut writer, config)?;
    NewLine::new().emit(&mut writer, config)?;

    MarkedSource::new_primary(files, &diagnostic).emit(&mut writer, config)?;
    NewLine::new().emit(&mut writer, config)?;

    for label in &diagnostic.secondary_labels {
        MarkedSource::new_secondary(files, &label).emit(&mut writer, config)?;
        NewLine::new().emit(&mut writer, config)?;
    }

    Ok(())
}

/// Diagnostic header.
///
/// ```text
/// error[E0001]: Unexpected type in `+` application
/// ```
#[derive(Copy, Clone, Debug)]
struct Header<'a> {
    severity: Severity,
    code: Option<&'a str>,
    message: &'a str,
}

impl<'a> Header<'a> {
    fn new(diagnostic: &'a Diagnostic) -> Header<'a> {
        Header {
            severity: diagnostic.severity,
            code: diagnostic.code.as_ref().map(String::as_str),
            message: &diagnostic.message,
        }
    }

    fn severity_name(&self) -> &'static str {
        match self.severity {
            Severity::Bug => "bug",
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Help => "help",
            Severity::Note => "note",
        }
    }

    fn emit(&self, writer: &mut impl WriteColor, config: &Config) -> io::Result<()> {
        let message_spec = ColorSpec::new().set_bold(true).set_intense(true).clone();
        let primary_spec = ColorSpec::new()
            .set_bold(true)
            .set_intense(true)
            .set_fg(Some(config.severity_color(self.severity)))
            .clone();

        // Write severity name
        //
        // ```text
        // error
        // ```
        writer.set_color(&primary_spec)?;
        write!(writer, "{}", self.severity_name())?;
        if let Some(code) = &self.code {
            // Write error code
            //
            // ```text
            // [E0001]
            // ```
            write!(writer, "[{}]", code)?;
        }

        // Write diagnostic message
        //
        // ```text
        // : Unexpected type in `+` application
        // ```
        writer.set_color(&message_spec)?;
        write!(writer, ": {}", self.message)?;
        writer.reset()?;

        NewLine::new().emit(writer, config)?;

        Ok(())
    }
}

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
struct MarkedSource<'a> {
    files: &'a Files,
    label: &'a Label,
    mark_style: MarkStyle,
    notes: &'a [String],
}

impl<'a> MarkedSource<'a> {
    fn new_primary(files: &'a Files, diagnostic: &'a Diagnostic) -> MarkedSource<'a> {
        MarkedSource {
            files,
            label: &diagnostic.primary_label,
            mark_style: MarkStyle::Primary(diagnostic.severity),
            notes: &diagnostic.notes,
        }
    }

    fn new_secondary(files: &'a Files, label: &'a Label) -> MarkedSource<'a> {
        MarkedSource {
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

    fn source_slice(&self, span: Span) -> Result<&'a str, impl std::error::Error> {
        self.files.source_slice(self.label.file_id, span)
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

    fn emit(&self, writer: &mut impl WriteColor, config: &Config) -> io::Result<()> {
        let label_spec = ColorSpec::new()
            .set_fg(Some(self.label_color(config)))
            .clone();

        let start = self.location(self.start()).expect("location_start");
        let end = self.location(self.end()).expect("location_end");
        let start_line_span = self.line_span(start.line).expect("start_line_span");
        let end_line_span = self.line_span(end.line).expect("end_line_span");

        // Use the length of the last line number as the gutter padding
        let gutter_padding = format!("{}", end.line.number()).len();

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

        // Source code snippet
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
        let source_prefix = self.source_slice(prefix_span).expect("source_prefix");
        write!(writer, "{}", source_prefix)?;

        // Write marked section
        let mark_len = if start.line == end.line {
            // Single line

            // Write marked source section
            let marked_source = self.source_slice(self.span()).expect("marked_source");
            writer.set_color(&label_spec)?;
            write!(writer, "{}", marked_source)?;
            writer.reset()?;
            marked_source.width()
        } else {
            // Multiple lines

            // Write marked source section
            let marked_span = start_line_span.with_start(self.start());
            let marked_source = self.source_slice(marked_span).expect("marked_source_1");
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
                let marked_source = self.source_slice(marked_span).expect("marked_source_2");
                writer.set_color(&label_spec)?;
                write!(writer, "{}", marked_source.trim_end_matches(line_trimmer))?;
                NewLine::new().emit(writer, config)?;
            }

            // Write line number and border
            Gutter::new(end.line.number(), gutter_padding).emit(writer, config)?;
            BorderLeft::new().emit(writer, config)?;

            // Write marked source section
            let marked_span = end_line_span.with_end(self.end());
            let marked_source = self.source_slice(marked_span).expect("marked_source_3");
            writer.set_color(&label_spec)?;
            write!(writer, "{}", marked_source)?;
            writer.reset()?;
            marked_source.width()
        };

        // Write source suffix after marked section
        let suffix_span = end_line_span.with_start(self.end());
        let source_suffix = self.source_slice(suffix_span).expect("source_suffix");
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
            width = source_prefix.width(),
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

/// The 'location focus' of a source code snippet.
///
/// This is displayed in a way that other tools can understand, for
/// example when command+clicking in iTerm.
///
/// ```text
/// test:2:9
/// ```
struct Locus<'a> {
    file_name: &'a str,
    location: Location,
}

impl<'a> Locus<'a> {
    fn new(file_name: &'a str, location: Location) -> Locus<'a> {
        Locus {
            file_name,
            location,
        }
    }

    fn emit(&self, writer: &mut impl WriteColor, _config: &Config) -> io::Result<()> {
        write!(
            writer,
            "{file}:{line}:{column}",
            file = self.file_name,
            line = self.location.line.number(),
            column = self.location.column.number(),
        )
    }
}

/// The left-hand gutter of a source line.
struct Gutter {
    line_number: Option<LineNumber>,
    gutter_padding: usize,
}

impl<'a> Gutter {
    fn new(line_number: impl Into<Option<LineNumber>>, gutter_padding: usize) -> Gutter {
        Gutter {
            line_number: line_number.into(),
            gutter_padding,
        }
    }

    fn emit(&self, writer: &mut impl WriteColor, config: &Config) -> io::Result<()> {
        write!(writer, " ")?;
        match self.line_number {
            None => {
                write!(
                    writer,
                    "{space: >width$}",
                    space = "",
                    width = self.gutter_padding,
                )?;
            },
            Some(line_number) => {
                let line_number_spec = ColorSpec::new()
                    .set_fg(Some(config.line_number_color))
                    .clone();

                writer.set_color(&line_number_spec)?;
                write!(
                    writer,
                    "{line: >width$}",
                    line = line_number,
                    width = self.gutter_padding,
                )?;
                writer.reset()?;
            },
        }
        write!(writer, " ")?;

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
        let border_spec = ColorSpec::new().set_fg(Some(config.border_color)).clone();

        writer.set_color(&border_spec)?;
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
        let border_spec = ColorSpec::new().set_fg(Some(config.border_color)).clone();

        writer.set_color(&border_spec)?;
        for _ in 0..self.width {
            write!(writer, "{top}", top = config.border_top_char)?
        }
        writer.reset()?;

        Ok(())
    }
}

/// The left-hand border of a source line.
struct BorderLeft {}

impl<'a> BorderLeft {
    fn new() -> BorderLeft {
        BorderLeft {}
    }

    fn emit(&self, writer: &mut impl WriteColor, config: &Config) -> io::Result<()> {
        let border_spec = ColorSpec::new().set_fg(Some(config.border_color)).clone();

        writer.set_color(&border_spec)?;
        write!(writer, "{left} ", left = config.border_left_char)?;
        writer.reset()?;

        Ok(())
    }
}

/// Additional note
///
/// ```text
/// = expected type `Int`
///      found type `String`
/// ```
struct Note<'a> {
    gutter_padding: usize,
    message: &'a str,
}

impl<'a> Note<'a> {
    fn new(gutter_padding: usize, message: &'a str) -> Note<'a> {
        Note {
            gutter_padding,
            message,
        }
    }

    fn emit(&self, writer: &mut impl WriteColor, config: &Config) -> io::Result<()> {
        for (i, line) in self.message.lines().enumerate() {
            Gutter::new(None, self.gutter_padding).emit(writer, config)?;
            match i {
                0 => NoteBullet::new().emit(writer, config)?,
                _ => write!(writer, " ")?,
            }
            // Write line of message
            write!(writer, " {}", line)?;
            NewLine::new().emit(writer, config)?;
        }

        Ok(())
    }
}

/// The bullet that appears before a note.
///
/// ```text
/// =
/// ```
struct NoteBullet {}

impl<'a> NoteBullet {
    fn new() -> NoteBullet {
        NoteBullet {}
    }

    fn emit(&self, writer: &mut impl WriteColor, config: &Config) -> io::Result<()> {
        let note_bullet_spec = ColorSpec::new().set_fg(Some(config.note_bullet_color)).clone();

        writer.set_color(&note_bullet_spec)?;
        write!(writer, "{}", config.note_bullet_char)?;
        writer.reset()?;

        Ok(())
    }
}

/// A new line.
struct NewLine {}

impl<'a> NewLine {
    fn new() -> NewLine {
        NewLine {}
    }

    fn emit(&self, writer: &mut impl WriteColor, _config: &Config) -> io::Result<()> {
        write!(writer, "\n")
    }
}
