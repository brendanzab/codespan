use std::io;
use termcolor::{ColorSpec, WriteColor};

use crate::emitter::Config;

/// The bullet that appears before a note.
///
/// ```text
/// =
/// ```
pub struct NoteBullet {}

impl NoteBullet {
    pub fn new() -> NoteBullet {
        NoteBullet {}
    }

    pub fn emit(&self, writer: &mut impl WriteColor, config: &Config) -> io::Result<()> {
        let note_bullet_spec = ColorSpec::new()
            .set_fg(Some(config.note_bullet_color))
            .clone();

        writer.set_color(&note_bullet_spec)?;
        write!(writer, "{}", config.note_bullet_char)?;
        writer.reset()?;

        Ok(())
    }
}
