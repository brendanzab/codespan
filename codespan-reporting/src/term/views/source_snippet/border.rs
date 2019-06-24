use std::io;
use termcolor::WriteColor;

use crate::term::Config;

/// The top-left corner of a source line.
pub struct BorderTopLeft {}

impl BorderTopLeft {
    pub fn new() -> BorderTopLeft {
        BorderTopLeft {}
    }

    pub fn emit(&self, writer: &mut impl WriteColor, config: &Config) -> io::Result<()> {
        writer.set_color(&config.styles.source_border)?;
        write!(writer, "{top_left}", top_left = config.source_border_top_left_char)?;
        writer.reset()?;

        Ok(())
    }
}

/// The top border of a source line.
pub struct BorderTop {
    width: usize,
}

impl BorderTop {
    pub fn new(width: usize) -> BorderTop {
        BorderTop { width }
    }

    pub fn emit(&self, writer: &mut impl WriteColor, config: &Config) -> io::Result<()> {
        writer.set_color(&config.styles.source_border)?;
        for _ in 0..self.width {
            write!(writer, "{top}", top = config.source_border_top_char)?
        }
        writer.reset()?;

        Ok(())
    }
}

/// The left-hand border of a source line.
pub struct BorderLeft {}

impl BorderLeft {
    pub fn new() -> BorderLeft {
        BorderLeft {}
    }

    pub fn emit(&self, writer: &mut impl WriteColor, config: &Config) -> io::Result<()> {
        writer.set_color(&config.styles.source_border)?;
        write!(writer, "{left}", left = config.source_border_left_char)?;
        writer.reset()?;

        Ok(())
    }
}
