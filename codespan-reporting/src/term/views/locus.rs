use std::io;
use termcolor::WriteColor;

use crate::term::Config;

/// The 'location focus' of a source code snippet.
///
/// This is displayed in a way that other tools can understand, for
/// example when command+clicking in iTerm.
///
/// ```text
/// test:2:9
/// ```
pub struct Locus<Origin> {
    origin: Origin,
    line_number: usize,
    column_number: usize,
}

impl<Origin> Locus<Origin>
where
    Origin: std::fmt::Display,
{
    pub fn new(origin: Origin, line_number: usize, column_number: usize) -> Locus<Origin> {
        Locus {
            origin,
            line_number,
            column_number,
        }
    }

    pub fn emit(
        &self,
        writer: &mut (impl WriteColor + ?Sized),
        _config: &Config,
    ) -> io::Result<()> {
        write!(
            writer,
            "{origin}:{line_number}:{column_number}",
            origin = self.origin,
            line_number = self.line_number,
            column_number = self.column_number,
        )
    }
}
