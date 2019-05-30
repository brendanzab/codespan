use codespan::{CodeMap, LineIndex, RawIndex};
use std::io;
use termcolor::{Color, ColorSpec, WriteColor};

use crate::{Diagnostic, LabelStyle, Severity};

pub fn emit(
    mut writer: impl WriteColor,
    codemap: &CodeMap<impl AsRef<str>>,
    diagnostic: &Diagnostic,
) -> io::Result<()> {
    let severity_color = severity_color(diagnostic.severity);
    let gutter_spec = ColorSpec::new().set_fg(Some(gutter_color())).clone();
    let primary_spec = ColorSpec::new().set_fg(Some(severity_color)).clone();
    let secondary_spec = ColorSpec::new().set_fg(Some(secondary_color())).clone();
    let header_message_spec = ColorSpec::new().set_bold(true).set_intense(true).clone();
    let header_primary_spec = ColorSpec::new()
        .set_bold(true)
        .set_intense(true)
        .set_fg(Some(severity_color))
        .clone();

    // Diagnostic header
    //
    // ```
    // error[E0001]: Unexpected type in `+` application
    // ```

    // Write severity name
    //
    // ```
    // error
    // ```
    writer.set_color(&header_primary_spec)?;
    write!(writer, "{}", severity_name(diagnostic.severity))?;
    if let Some(code) = &diagnostic.code {
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
    writer.set_color(&header_message_spec)?;
    write!(writer, ": {}", diagnostic.message)?;
    write!(writer, "\n")?;
    writer.reset()?;

    for label in &diagnostic.labels {
        match codemap.find_file(label.span.start()) {
            None => {
                if !label.message.is_empty() {
                    write!(writer, "- {}", label.message)?;
                    write!(writer, "\n")?;
                }
            },
            Some(file) => {
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

                let label_spec = match label.style {
                    LabelStyle::Primary => &primary_spec,
                    LabelStyle::Secondary => &secondary_spec,
                };

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
                    write!(writer, "{}", underline_char(label.style))?;
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

// Blue is really difficult to see on the standard windows command line
// FIXME: Make colors configurable
#[cfg(windows)]
const BLUE: Color = Color::Cyan;
#[cfg(not(windows))]
const BLUE: Color = Color::Blue;

/// Return the termcolor to use when rendering messages of this diagnostic severity.
fn severity_color(severity: Severity) -> Color {
    match severity {
        Severity::Bug | Severity::Error => Color::Red,
        Severity::Warning => Color::Yellow,
        Severity::Note => Color::Green,
        Severity::Help => Color::Cyan,
    }
}

/// The color to use for secondary highlights.
fn secondary_color() -> Color {
    BLUE
}

/// The color to use for gutters highlights.
fn gutter_color() -> Color {
    BLUE
}

/// A string that explains this diagnostic severity.
fn severity_name(severity: Severity) -> &'static str {
    match severity {
        Severity::Bug => "bug",
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Note => "note",
        Severity::Help => "help",
    }
}

/// The character used for the underlined section of code.
fn underline_char(label_style: LabelStyle) -> char {
    match label_style {
        LabelStyle::Primary => '^',
        LabelStyle::Secondary => '-',
    }
}
