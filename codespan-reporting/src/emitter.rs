use codespan::CodeMap;
use std::{fmt, io};
use termcolor::{Color, ColorSpec, WriteColor};

use {Diagnostic, LabelStyle};

struct Pad<T>(T, usize);

impl<T: fmt::Display> fmt::Display for Pad<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for _ in 0..(self.1) {
            self.0.fmt(f)?;
        }
        Ok(())
    }
}

pub fn emit<W>(mut writer: W, codemap: &CodeMap, diagnostic: &Diagnostic) -> io::Result<()>
where
    W: WriteColor,
{
    let supports_color = writer.supports_color();
    let line_location_color = ColorSpec::new()
        // Blue is really difficult to see on the standard windows command line
        .set_fg(Some(if cfg!(windows) { Color::Cyan } else { Color::Blue }))
        .clone();
    let diagnostic_color = ColorSpec::new()
        .set_fg(Some(diagnostic.severity.color()))
        .clone();

    let highlight_color = ColorSpec::new().set_bold(true).set_intense(true).clone();

    writer.set_color(&diagnostic_color)?;
    write!(writer, "{}", diagnostic.severity)?;
    writer.reset()?;
    write!(writer, ":")?;
    writer.set_color(&highlight_color)?;
    writeln!(writer, " {}", diagnostic.message)?;
    writer.reset()?;

    for label in &diagnostic.labels {
        match codemap.find_file(label.span.start()) {
            None => if let Some(ref message) = label.message {
                writeln!(writer, "- {}", message)?
            },
            Some(file) => {
                let (line, column) = file.location(label.span.start()).expect("location");
                writeln!(
                    writer,
                    "- {file}:{line}:{column}",
                    file = file.name(),
                    line = line.number(),
                    column = column.number(),
                )?;

                let line_span = file.line_span(line).expect("line_span");

                let line_prefix = file.src_slice(line_span.with_end(label.span.start()))
                    .expect("line_prefix");
                let line_marked = file.src_slice(label.span).expect("line_marked");
                let line_suffix = file.src_slice(line_span.with_start(label.span.end()))
                    .expect("line_suffix")
                    .trim_right_matches(|ch: char| ch == '\r' || ch == '\n');

                let mark = match label.style {
                    LabelStyle::Primary => '^',
                    LabelStyle::Secondary => '-',
                };
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

                writer.set_color(&line_location_color)?;
                let line_string = line.number().to_string();
                let line_location_prefix = format!("{} | ", Pad(' ', line_string.len()));
                write!(writer, "{} | ", line_string)?;
                writer.reset()?;

                write!(writer, "{}", line_prefix)?;
                writer.set_color(&label_color)?;
                write!(writer, "{}", line_marked)?;
                writer.reset()?;
                writeln!(writer, "{}", line_suffix)?;

                if !supports_color || label.message.is_some() {
                    writer.set_color(&line_location_color)?;
                    write!(writer, "{}", line_location_prefix)?;
                    writer.reset()?;

                    writer.set_color(&label_color)?;
                    write!(
                        writer,
                        "{}{}",
                        Pad(' ', line_prefix.len()),
                        Pad(mark, line_marked.len()),
                    )?;
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
            }
        }
    }
    Ok(())
}
