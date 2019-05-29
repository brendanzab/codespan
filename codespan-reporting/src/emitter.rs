use codespan::{CodeMap, LineIndex, RawIndex};
use std::io;
use termcolor::{Color, ColorSpec, WriteColor};

use crate::{Diagnostic, LabelStyle};

// Blue is really difficult to see on the standard windows command line
// FIXME: Make colors configurable
#[cfg(windows)]
const BLUE: Color = Color::Cyan;
#[cfg(not(windows))]
const BLUE: Color = Color::Blue;

pub fn emit<W, S>(mut writer: W, codemap: &CodeMap<S>, diagnostic: &Diagnostic) -> io::Result<()>
where
    W: WriteColor,
    S: AsRef<str>,
{
    let line_location_color = ColorSpec::new().set_fg(Some(BLUE)).clone();
    let diagnostic_color = ColorSpec::new()
        .set_fg(Some(diagnostic.severity.color()))
        .clone();

    let highlight_color = ColorSpec::new().set_bold(true).set_intense(true).clone();

    // Diagnostic header

    writer.set_color(
        &highlight_color
            .clone()
            .set_fg(Some(diagnostic.severity.color())),
    )?;

    // Severity
    //
    // ```
    // error
    // ```
    write!(writer, "{}", diagnostic.severity)?;

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
                    LabelStyle::Secondary => ColorSpec::new().set_fg(Some(BLUE)).clone(),
                };

                // Write prefix to marked section

                writer.set_color(&line_location_color)?;
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
                        writer.set_color(&line_location_color)?;
                        write!(writer, "{} | ", line_index.number())?;

                        let line_span = file.line_span(line_index).expect("marked_line_span");
                        let line = file
                            .src_slice(line_span)
                            .expect("line")
                            .trim_end_matches(|ch: char| ch == '\r' || ch == '\n');
                        writer.set_color(&label_color)?;
                        writeln!(writer, "{}", line)?;
                    }

                    writer.set_color(&line_location_color)?;
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

                let mark = match label.style {
                    LabelStyle::Primary => '^',
                    LabelStyle::Secondary => '-',
                };

                writer.set_color(&line_location_color)?;
                write!(writer, "{: <width$} | ", "", width = line_location_width)?;
                writer.reset()?;

                writer.set_color(&label_color)?;
                write!(writer, "{: <width$}", "", width = prefix.len())?;
                for _ in 0..mark_len {
                    write!(writer, "{}", mark)?;
                }
                writer.reset()?;

                if !label.message.is_empty() {
                    writer.set_color(&label_color)?;
                    writeln!(writer, " {}", label.message)?;
                    writer.reset()?;
                }
                writer.set_color(&line_location_color)?;
                writeln!(writer, "{: <width$} | ", "", width = line_location_width)?;
                writer.reset()?;
            },
        }
    }
    Ok(())
}
