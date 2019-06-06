use std::io;
use termcolor::{ColorSpec, WriteColor};

use crate::emitter::Config;

/// The left-hand border of a source line.
pub struct BorderLeft {}

impl<'a> BorderLeft {
    pub fn new() -> BorderLeft {
        BorderLeft {}
    }

    pub fn emit(&self, writer: &mut impl WriteColor, config: &Config) -> io::Result<()> {
        let border_spec = ColorSpec::new().set_fg(Some(config.border_color)).clone();

        writer.set_color(&border_spec)?;
        write!(writer, "{left} ", left = config.border_left_char)?;
        writer.reset()?;

        Ok(())
    }
}
