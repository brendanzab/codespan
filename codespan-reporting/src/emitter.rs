use codespan::{CodeMap, LineIndex, RawIndex};
use std::io;
use termcolor::{Color, ColorSpec, WriteColor};

use crate::{Diagnostic, LabelStyle, Severity};

pub fn emit<W, S>(mut writer: W, codemap: &CodeMap<S>, diagnostic: &Diagnostic) -> io::Result<()>
where
    W: WriteColor,
    S: AsRef<str>,
{
    let gutter_color = ColorSpec::new().set_fg(Some(gutter_color())).clone();
    let diagnostic_color = ColorSpec::new()
        .set_fg(Some(severity_color(diagnostic.severity)))
        .clone();

    let highlight_color = ColorSpec::new().set_bold(true).set_intense(true).clone();

    // Diagnostic header

    writer.set_color(
        &highlight_color
            .clone()
            .set_fg(Some(severity_color(diagnostic.severity))),
    )?;

    // Severity
    //
    // ```
    // error
    // ```
    write!(writer, "{}", severity_name(diagnostic.severity))?;

    // Error code
    //
    // ```
    // [E0001]
    // ```
    if let Some(code) = &diagnostic.code {
        write!(writer, "[{}]", code)?;
    }

    // Diagnostic message
    //
    // ```
    // : Unexpected type in `+` application
    // ```
    writer.set_color(&highlight_color)?;
    writeln!(writer, ": {}", diagnostic.message)?;
    writer.reset()?;

    // Diagnostic Labels

    for label in &diagnostic.labels {
        match codemap.find_file(label.span.start()) {
            None => {
                if !label.message.is_empty() {
                    writeln!(writer, "- {}", label.message)?
                }
            },
            Some(file) => {
                let (start_line, column) =
                    file.location(label.span.start()).expect("location_start");
                let (end_line, _) = file.location(label.span.end()).expect("location_end");

                // File name
                //
                // ```
                // - <test>:2:9
                // ```

                writeln!(
                    writer,
                    "- {file}:{line}:{column}",
                    file = file.name(),
                    line = start_line.number(),
                    column = column.number(),
                )?;

                // Source code snippet
                //
                // ```
                //   |
                // 2 | (+ test "")
                //   |         ^^ Expected integer but got string
                //   |
                // ```

                let start_line_span = file.line_span(start_line).expect("line_span");
                let end_line_span = file.line_span(end_line).expect("line_span");
                let line_location_width = end_line.number().to_string().len();

                let label_color = match label.style {
                    LabelStyle::Primary => diagnostic_color.clone(),
                    LabelStyle::Secondary => ColorSpec::new().set_fg(Some(secondary_color())).clone(),
                };

                // Write prefix to marked section

                writer.set_color(&gutter_color)?;
                let line_string = start_line.number().to_string();
                writeln!(writer, "{: <width$} | ", "", width = line_location_width)?;
                write!(writer, "{} | ", line_string)?;

                let prefix = file
                    .src_slice(start_line_span.with_end(label.span.start()))
                    .expect("prefix");
                writer.reset()?;
                write!(writer, "{}", prefix)?;

                // Write marked section
                let mark_len = if start_line == end_line {
                    // Single line

                    let marked = file.src_slice(label.span).expect("marked");
                    writer.set_color(&label_color)?;
                    write!(writer, "{}", marked)?;
                    marked.len()
                } else {
                    // Multiple lines

                    let marked = file
                        .src_slice(start_line_span.with_start(label.span.start()))
                        .expect("start_of_marked");
                    writer.set_color(&label_color)?;
                    write!(writer, "{}", marked)?;

                    for line_index in ((start_line.to_usize() + 1)..end_line.to_usize())
                        .map(|i| LineIndex::from(i as RawIndex))
                    {
                        writer.set_color(&gutter_color)?;
                        write!(writer, "{} | ", line_index.number())?;

                        let line_span = file.line_span(line_index).expect("marked_line_span");
                        let line = file
                            .src_slice(line_span)
                            .expect("line")
                            .trim_end_matches(|ch: char| ch == '\r' || ch == '\n');
                        writer.set_color(&label_color)?;
                        writeln!(writer, "{}", line)?;
                    }

                    writer.set_color(&gutter_color)?;
                    write!(writer, "{} | ", end_line.number())?;
                    let line = file
                        .src_slice(end_line_span.with_end(label.span.end()))
                        .expect("line");
                    writer.set_color(&label_color)?;
                    write!(writer, "{}", line)?;
                    line.len()
                };

                // Write suffix to marked section

                let suffix = file
                    .src_slice(end_line_span.with_start(label.span.end()))
                    .expect("suffix")
                    .trim_end_matches(|ch: char| ch == '\r' || ch == '\n');
                writer.reset()?;
                writeln!(writer, "{}", suffix)?;

                // Write mark and label

                writer.set_color(&gutter_color)?;
                write!(writer, "{: <width$} | ", "", width = line_location_width)?;
                writer.reset()?;

                writer.set_color(&label_color)?;
                write!(writer, "{: <width$}", "", width = prefix.len())?;
                for _ in 0..mark_len {
                    write!(writer, "{}", underline_mark(label.style))?;
                }
                writer.reset()?;

                if !label.message.is_empty() {
                    writer.set_color(&label_color)?;
                    writeln!(writer, " {}", label.message)?;
                    writer.reset()?;
                }
                writer.set_color(&gutter_color)?;
                writeln!(writer, "{: <width$} | ", "", width = line_location_width)?;
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

/// The mark used for the underlined section of code.
fn underline_mark(label_style: LabelStyle) -> char {
    match label_style {
        LabelStyle::Primary => '^',
        LabelStyle::Secondary => '-',
    }
}
