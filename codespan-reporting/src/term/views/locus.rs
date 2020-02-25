use std::io;
use termcolor::WriteColor;

use crate::diagnostic::Location;
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
    location: Location,
}

impl<Origin> Locus<Origin>
where
    Origin: std::fmt::Display,
{
    pub fn new(origin: Origin, location: Location) -> Locus<Origin> {
        Locus { origin, location }
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
            line_number = self.location.line_number,
            column_number = self.location.column_number,
        )
    }
}
