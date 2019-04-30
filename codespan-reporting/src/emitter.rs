use codespan::{CodeMap, LineIndex, LineNumber};
use std::{fmt, io};
use termcolor::{Color, ColorSpec, WriteColor};

use crate::{Diagnostic, LabelStyle};

struct Pad<T>(T, usize);

impl<T: fmt::Display> fmt::Display for Pad<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for _ in 0..(self.1) {
            self.0.fmt(f)?;
        }
        Ok(())
    }
}

pub fn emit<W, S>(mut writer: W, codemap: &CodeMap<S>, diagnostic: &Diagnostic) -> io::Result<()>
where
    W: WriteColor,
    S: AsRef<str>,
{
    let supports_color = writer.supports_color();
    let line_location_color = ColorSpec::new()
        // Blue is really difficult to see on the standard windows command line
        .set_fg(Some(if cfg!(windows) {
            Color::Cyan
        } else {
            Color::Blue
        }))
        .clone();
    let diagnostic_color = ColorSpec::new()
        .set_fg(Some(diagnostic.severity.color()))
        .clone();

    let highlight_color = ColorSpec::new().set_bold(true).set_intense(true).clone();

    writer.set_color(
        &highlight_color
            .clone()
            .set_fg(Some(diagnostic.severity.color())),
    )?;
    write!(writer, "{}", diagnostic.severity)?;

    if let Some(ref code) = diagnostic.code {
        write!(writer, "[{}]", code)?;
    }

    writer.set_color(&highlight_color)?;
    writeln!(writer, ": {}", diagnostic.message)?;
    writer.reset()?;

    for label in &diagnostic.labels {
        match codemap.find_file(label.span.start()) {
            None => {
                if let Some(ref message) = label.message {
                    writeln!(writer, "- {}", message)?
                }
            }
            Some(file) => {
                let (start_line, column) =
                    file.location(label.span.start()).expect("location_start");
                let (end_line, _) = file.location(label.span.end()).expect("location_end");

                writeln!(
                    writer,
                    "- {file}:{line}:{column}",
                    file = file.name(),
                    line = start_line.number(),
                    column = column.number(),
                )?;

                let start_line_span = file.line_span(start_line).expect("line_span");
                let end_line_span = file.line_span(end_line).expect("line_span");
                let line_location_width = end_line.number().to_string().len();
                let line_location_prefix = format!("{} | ", Pad(' ', line_location_width));

                let label_color = match label.style {
                    LabelStyle::Primary => diagnostic_color.clone(),
                    LabelStyle::Secondary => ColorSpec::new()
                        .set_fg(Some(if cfg!(windows) {
                            Color::Cyan
                        } else {
                            Color::Blue
                        }))
                        .clone(),
                };

                // Write prefix to marked section
                //
                writer.set_color(&line_location_color)?;
                let line_string = start_line.number().to_string();
                writeln!(writer, "{} |", Pad(' ', line_location_width))?;
                write!(writer, "{} | ", line_string)?;

                let prefix = file
                    .src_slice(start_line_span.with_end(label.span.start()))
                    .expect("prefix");
                writer.reset()?;
                write!(writer, "{}", prefix)?;

                // Write marked section
                //
                let LineNumber(start) = start_line.number();
                let LineNumber(end) = end_line.number();

                let mark_len = if start == end {
                    let marked = file.src_slice(label.span).expect("marked");
                    writer.set_color(&label_color)?;
                    write!(writer, "{}", marked)?;
                    marked.len()
                } else {
                    let marked = file
                        .src_slice(start_line_span.with_start(label.span.start()))
                        .expect("start_of_marked");
                    writer.set_color(&label_color)?;
                    write!(writer, "{}", marked)?;

                    for i in start..(end - 1) {
                        writer.set_color(&line_location_color)?;
                        write!(writer, "{} | ", (i + 1).to_string())?;

                        let line_span = file.line_span(LineIndex(i)).expect("marked_line_span");
                        let line = file
                            .src_slice(line_span)
                            .expect("line")
                            .trim_right_matches(|ch: char| ch == '\r' || ch == '\n');
                        writer.set_color(&label_color)?;
                        writeln!(writer, "{}", line)?;
                    }

                    writer.set_color(&line_location_color)?;
                    write!(writer, "{} | ", end)?;
                    let line = file
                        .src_slice(end_line_span.with_end(label.span.end()))
                        .expect("line");
                    writer.set_color(&label_color)?;
                    write!(writer, "{}", line)?;
                    line.len()
                };

                // Write suffix to marked section
                //
                let suffix = file
                    .src_slice(end_line_span.with_start(label.span.end()))
                    .expect("suffix")
                    .trim_right_matches(|ch: char| ch == '\r' || ch == '\n');
                writer.reset()?;
                writeln!(writer, "{}", suffix)?;

                // Write mark and label

                let mark = match label.style {
                    LabelStyle::Primary => '^',
                    LabelStyle::Secondary => '-',
                };

                if !supports_color || label.message.is_some() {
                    writer.set_color(&line_location_color)?;
                    write!(writer, "{}", line_location_prefix)?;
                    writer.reset()?;

                    writer.set_color(&label_color)?;
                    write!(writer, "{}{}", Pad(' ', prefix.len()), Pad(mark, mark_len))?;
                    writer.reset()?;

                    if label.message.is_none() {
                        writeln!(writer)?;
                    }
                }

                match label.message {
                    None => (),
                    Some(ref label) => {
                        writer.set_color(&label_color)?;
                        writeln!(writer, " {}", label)?;
                        writer.reset()?;
                    }
                }
                writer.set_color(&line_location_color)?;
                writeln!(writer, "{} |", Pad(' ', line_location_width))?;
                writer.reset()?;
            }
        }
    }
    Ok(())
}
