use std::io;
use termcolor::WriteColor;

use crate::term::Config;

/// A new line.
pub struct NewLine {}

impl NewLine {
    pub fn new() -> NewLine {
        NewLine {}
    }

    pub fn emit(&self, writer: &mut impl WriteColor, _config: &Config) -> io::Result<()> {
        write!(writer, "\n")
    }
}
