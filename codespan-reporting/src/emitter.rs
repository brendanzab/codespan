use std::io;

use codespan::CodeMap;

use termcolor::{Color, ColorSpec, WriteColor};

use Diagnostic;

pub fn emit<W>(mut writer: W, codemap: &CodeMap, diagnostic: &Diagnostic) -> io::Result<()>
where
    W: WriteColor,
{
    writer.set_color(ColorSpec::new().set_fg(Some(diagnostic.severity.color())))?;
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
                    .expect("line_suffix");

                writer.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)))?;
                write!(writer, "{} | ", line.number())?;
                writer.reset()?;

                write!(writer, "{}", line_prefix)?;
                writer.set_color(ColorSpec::new().set_fg(Some(Color::Red)))?;
                write!(writer, "{}", line_marked)?;
                writer.reset()?;
                write!(writer, "{}", line_suffix)?;

                match label.message {
                    None => writeln!(writer)?,
                    Some(ref label) => writeln!(writer, ": {}", label)?,
                }
            },
        }
    }
    Ok(())
}
