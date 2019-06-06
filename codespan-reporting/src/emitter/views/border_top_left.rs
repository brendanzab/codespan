use std::io;
use termcolor::{ColorSpec, WriteColor};

use crate::emitter::Config;

/// The top-left corner of a source line.
pub struct BorderTopLeft {}

impl BorderTopLeft {
    pub fn new() -> BorderTopLeft {
        BorderTopLeft {}
    }

    pub fn emit(&self, writer: &mut impl WriteColor, config: &Config) -> io::Result<()> {
        let border_spec = ColorSpec::new().set_fg(Some(config.border_color)).clone();

        writer.set_color(&border_spec)?;
        write!(writer, "{top_left}", top_left = config.border_top_left_char)?;
        writer.reset()?;

        Ok(())
    }
}
