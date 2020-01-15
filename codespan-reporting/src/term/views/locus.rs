use codespan::Location;
use std::ffi::OsStr;
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
pub struct Locus<'a> {
    file_name: &'a OsStr,
    location: Location,
}

impl<'a> Locus<'a> {
    pub fn new(file_name: &'a OsStr, location: Location) -> Locus<'a> {
        Locus {
            file_name,
            location,
        }
    }

    pub fn emit(
        &self,
        writer: &mut (impl WriteColor + ?Sized),
        _config: &Config,
    ) -> io::Result<()> {
        use std::path::PathBuf;
        write!(
            writer,
            "{file}:{line}:{column}",
            file = PathBuf::from(self.file_name).display(),
            line = self.location.line.number(),
            column = self.location.column.number(),
        )
    }
}
