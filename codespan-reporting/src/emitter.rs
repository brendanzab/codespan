use std::io;

use codespan::CodeMap;

use termcolor::{ColorSpec, WriteColor};

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
                match label.message {
                    None => writeln!(writer)?,
                    Some(ref label) => writeln!(writer, ": {}", label)?,
                }
            },
        }
    }
    Ok(())
}
