use std::io;
use termcolor::WriteColor;

use crate::term::Config;

/// The left-hand gutter of a source line.
pub struct Gutter {
    line_index: Option<usize>,
    gutter_padding: usize,
}

impl Gutter {
    pub fn new(line_index: impl Into<Option<usize>>, gutter_padding: usize) -> Gutter {
        Gutter {
            line_index: line_index.into(),
            gutter_padding,
        }
    }

    pub fn emit(&self, writer: &mut (impl WriteColor + ?Sized), config: &Config) -> io::Result<()> {
        write!(writer, " ")?;
        match self.line_index {
            None => {
                write!(
                    writer,
                    "{space: >width$}",
                    space = "",
                    width = self.gutter_padding,
                )?;
            },
            Some(line_index) => {
                writer.set_color(&config.styles.line_number)?;
                write!(
                    writer,
                    "{line: >width$}",
                    line = line_index + 1,
                    width = self.gutter_padding,
                )?;
                writer.reset()?;
            },
        }
        write!(writer, " ")?;

        Ok(())
    }
}
