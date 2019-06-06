use std::io;
use termcolor::{ColorSpec, WriteColor};

use crate::emitter::Config;

/// The top border of a source line.
pub struct BorderTop {
    width: usize,
}

impl BorderTop {
    pub fn new(width: usize) -> BorderTop {
        BorderTop { width }
    }

    pub fn emit(&self, writer: &mut impl WriteColor, config: &Config) -> io::Result<()> {
        let border_spec = ColorSpec::new().set_fg(Some(config.border_color)).clone();

        writer.set_color(&border_spec)?;
        for _ in 0..self.width {
            write!(writer, "{top}", top = config.border_top_char)?
        }
        writer.reset()?;

        Ok(())
    }
}
