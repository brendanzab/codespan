use codespan::{CodeMap, LineIndex, RawIndex};
use std::io;
use termcolor::{Color, ColorSpec, WriteColor};

use crate::{Diagnostic, LabelStyle, Severity};

#[derive(Clone, Debug)]
pub struct Config {
    pub bug_color: Color,
    pub error_color: Color,
    pub warning_color: Color,
    pub note_color: Color,
    pub help_color: Color,
    pub secondary_color: Color,
    pub gutter_color: Color,
    pub primary_mark: char,
    pub secondary_mark: char,
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
            gutter_color: BLUE,
            primary_mark: '^',
            secondary_mark: '-',
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

    /// The style used to mark the labelled section of code.
    pub fn label_color(&self, severity: Severity, label_style: LabelStyle) -> Color {
        match label_style {
            LabelStyle::Primary => self.severity_color(severity),
            LabelStyle::Secondary => self.secondary_color,
        }
    }

    /// The character used for the underlined section of code.
    fn underline_char(&self, label_style: LabelStyle) -> char {
        match label_style {
            LabelStyle::Primary => self.primary_mark,
            LabelStyle::Secondary => self.secondary_mark,
        }
    }
}

pub fn emit(
    mut writer: impl WriteColor,
    config: &Config,
    codemap: &CodeMap<impl AsRef<str>>,
    diagnostic: &Diagnostic,
) -> io::Result<()> {
    Header::new(diagnostic).emit(&mut writer, config)?;

    for label in &diagnostic.labels {
        match codemap.find_file(label.span.start()) {
            None => {
                if !label.message.is_empty() {
                    write!(writer, "- {}", label.message)?;
                    write!(writer, "\n")?;
                }
            },
            Some(file) => {
                let gutter_spec = ColorSpec::new().set_fg(Some(config.gutter_color)).clone();
                let (start_line, start_column) =
                    file.location(label.span.start()).expect("location_start");
                let (end_line, _) = file.location(label.span.end()).expect("location_end");
                let start_line_span = file.line_span(start_line).expect("line_span");
                let end_line_span = file.line_span(end_line).expect("line_span");

                // Use the length of the last line number as the gutter padding
                let gutter_padding = format!("{}", end_line.number()).len();

                // File name
                //
                // ```
                // ┌╴ <test>:2:9
                // ```

                // Write gutter
                writer.set_color(&gutter_spec)?;
                write!(writer, "{: >width$} ┌╴ ", "", width = gutter_padding)?;
                writer.reset()?;

                // Write file name
                write!(
                    writer,
                    "{file}:{line}:{column}",
                    file = file.name(),
                    line = start_line.number(),
                    column = start_column.number(),
                )?;
                write!(writer, "\n")?;

                // Source code snippet
                //
                // ```
                //   │
                // 2 │ (+ test "")
                //   │         ^^ Expected integer but got string
                //   ╵
                // ```

                // Write line number and gutter
                writer.set_color(&gutter_spec)?;
                write!(writer, "{: >width$} │ ", "", width = gutter_padding)?;
                write!(writer, "\n")?;
                write!(
                    writer,
                    "{: >width$} │ ",
                    start_line.number(),
                    width = gutter_padding,
                )?;
                writer.reset()?;

                // Write source prefix before marked section
                let source_prefix = file
                    .src_slice(start_line_span.with_end(label.span.start()))
                    .expect("prefix");
                write!(writer, "{}", source_prefix)?;

                let label_color = config.label_color(diagnostic.severity, label.style);
                let label_spec = ColorSpec::new().set_fg(Some(label_color)).clone();

                // Write marked section
                let mark_len = if start_line == end_line {
                    // Single line

                    // Write marked source section
                    let marked_source = file.src_slice(label.span).expect("marked_source");
                    writer.set_color(&label_spec)?;
                    write!(writer, "{}", marked_source)?;
                    writer.reset()?;
                    marked_source.len()
                } else {
                    // Multiple lines

                    // Write marked source section
                    let marked_source = file
                        .src_slice(start_line_span.with_start(label.span.start()))
                        .expect("start_of_marked");
                    writer.set_color(&label_spec)?;
                    write!(writer, "{}", marked_source)?;

                    for line_index in ((start_line.to_usize() + 1)..end_line.to_usize())
                        .map(|i| LineIndex::from(i as RawIndex))
                    {
                        // Write line number and gutter
                        writer.set_color(&gutter_spec)?;
                        write!(
                            writer,
                            "{: >width$} │ ",
                            line_index.number(),
                            width = gutter_padding,
                        )?;

                        // Write marked source section
                        let line_span = file.line_span(line_index).expect("marked_line_span");
                        let marked_source = file
                            .src_slice(line_span)
                            .expect("marked_source")
                            .trim_end_matches(|ch: char| ch == '\r' || ch == '\n');
                        writer.set_color(&label_spec)?;
                        write!(writer, "{}", marked_source)?;
                        write!(writer, "\n")?;
                    }

                    // Write line number and gutter
                    writer.set_color(&gutter_spec)?;
                    write!(
                        writer,
                        "{: >width$} │ ",
                        end_line.number(),
                        width = gutter_padding,
                    )?;

                    // Write marked source section
                    let marked_source = file
                        .src_slice(end_line_span.with_end(label.span.end()))
                        .expect("marked_source");
                    writer.set_color(&label_spec)?;
                    write!(writer, "{}", marked_source)?;
                    writer.reset()?;
                    marked_source.len()
                };

                // Write source suffix after marked section
                let source = file
                    .src_slice(end_line_span.with_start(label.span.end()))
                    .expect("suffix")
                    .trim_end_matches(|ch: char| ch == '\r' || ch == '\n');
                write!(writer, "{}", source)?;
                write!(writer, "\n")?;

                // Write underline gutter
                writer.set_color(&gutter_spec)?;
                write!(writer, "{: >width$} │ ", "", width = gutter_padding)?;
                writer.reset()?;

                // Write underline and label
                writer.set_color(&label_spec)?;
                write!(writer, "{: >width$}", "", width = source_prefix.len())?;
                for _ in 0..mark_len {
                    write!(writer, "{}", config.underline_char(label.style))?;
                }
                if !label.message.is_empty() {
                    write!(writer, " {}", label.message)?;
                    write!(writer, "\n")?;
                }
                writer.reset()?;

                // Write final gutter
                writer.set_color(&gutter_spec)?;
                write!(writer, "{: >width$} ╵", "", width = gutter_padding)?;
                write!(writer, "\n")?;
                writer.reset()?;
            },
        }
    }
    Ok(())
}

/// Diagnostic header
///
/// ```
/// error[E0001]: Unexpected type in `+` application
/// ```
#[derive(Copy, Clone, Debug)]
struct Header<'doc> {
    severity: Severity,
    code: Option<&'doc str>,
    message: &'doc str,
}

impl<'doc> Header<'doc> {
    fn new(diagnostic: &'doc Diagnostic) -> Header<'doc> {
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
        // ```
        // error
        // ```
        writer.set_color(&primary_spec)?;
        write!(writer, "{}", self.severity_name())?;
        if let Some(code) = &self.code {
            // Write error code
            //
            // ```
            // [E0001]
            // ```
            write!(writer, "[{}]", code)?;
        }

        // Write diagnostic message
        //
        // ```
        // : Unexpected type in `+` application
        // ```
        writer.set_color(&message_spec)?;
        write!(writer, ": {}", self.message)?;
        write!(writer, "\n")?;
        writer.reset()?;

        Ok(())
    }
}
