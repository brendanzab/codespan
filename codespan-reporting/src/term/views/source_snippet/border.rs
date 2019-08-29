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
        write!(writer, "{}", config.chars.source_border_top_left)?;
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
            write!(writer, "{}", config.chars.source_border_top)?
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
        write!(writer, "{}", config.chars.source_border_left)?;
        writer.reset()?;

        Ok(())
    }
}

/// The left-hand border of a source line.
pub struct BorderLeftBreak {}

impl BorderLeftBreak {
    pub fn new() -> BorderLeftBreak {
        BorderLeftBreak {}
    }

    pub fn emit(&self, writer: &mut impl WriteColor, config: &Config) -> io::Result<()> {
        writer.set_color(&config.styles.source_border)?;
        write!(writer, "{}", config.chars.source_border_left_break)?;
        writer.reset()?;

        Ok(())
    }
}
