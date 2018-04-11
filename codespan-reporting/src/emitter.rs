use std::io;

use codespan::CodeMap;

use termcolor::{Color, ColorSpec, WriteColor};

use Diagnostic;

pub fn emit<W>(mut writer: W, codemap: &CodeMap, diagnostic: &Diagnostic) -> io::Result<()>
where
    W: WriteColor,
{
    let supports_color = writer.supports_color();
    let line_location_color = ColorSpec::new().set_fg(Some(Color::Cyan)).clone();
    let diagnostic_color = ColorSpec::new()
        .set_fg(Some(diagnostic.severity.color()))
        .clone();

    writer.set_color(&diagnostic_color)?;
    write!(writer, "{}", diagnostic.severity)?;
    writer.reset()?;
    writeln!(writer, ": {}", diagnostic.message)?;
    for label in &diagnostic.labels {
        match codemap.find_file(label.span.start()) {
            None => if let Some(ref message) = label.message {
                writeln!(writer, "- {}", message)?
            },
            Some(file) => {
                let (line, col) = file.location(label.span.start()).expect("location");
                writeln!(
                    writer,
                    "- {}:{}:{}",
                    file.name(),
                    line.number(),
                    col.number()
                )?;

                let line_span = file.line_span(line).expect("line_span");

                let line_prefix = file.src_slice(line_span.with_end(label.span.start()))
                    .expect("line_prefix");
                let line_marked = file.src_slice(label.span).expect("line_marked");
                let line_suffix = file.src_slice(line_span.with_start(label.span.end()))
                    .expect("line_suffix")
                    .trim_right_matches(|c: char| c == '\r' || c == '\n');

                writer.set_color(&line_location_color)?;
                let line_string = line.number().to_string();
                let line_location_prefix = format!("{:prefix$} | ", "", prefix = line_string.len());
                write!(writer, "{} | ", line_string)?;
                writer.reset()?;

                write!(writer, "{}", line_prefix)?;
                writer.set_color(&diagnostic_color)?;
                write!(writer, "{}", line_marked)?;
                writer.reset()?;
                writeln!(writer, "{}", line_suffix)?;

                if !supports_color || label.message.is_some() {
                    writer.set_color(&line_location_color)?;
                    write!(writer, "{}", line_location_prefix)?;
                    writer.reset()?;

                    writer.set_color(&diagnostic_color)?;
                    write!(
                        writer,
                        "{:prefix$}{:^>marked$}",
                        "",
                        "",
                        prefix = line_prefix.len(),
                        marked = line_marked.len()
                    )?;
                    writer.reset()?;

                    if label.message.is_none() {
                        writeln!(writer)?;
                    }
                }

                match label.message {
                    None => (),
                    Some(ref label) => {
                        writer.set_color(&diagnostic_color)?;
                        writeln!(writer, " {}", label)?;
                        writer.reset()?;
                    },
                }
            },
        }
    }
    Ok(())
}
