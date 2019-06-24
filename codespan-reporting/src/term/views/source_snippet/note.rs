use std::io;
use termcolor::WriteColor;

use crate::term::Config;

use super::{Gutter, NewLine};

/// Additional note
///
/// ```text
/// = expected type `Int`
///      found type `String`
/// ```
pub struct Note<'a> {
    gutter_padding: usize,
    message: &'a str,
}

impl<'a> Note<'a> {
    pub fn new(gutter_padding: usize, message: &'a str) -> Note<'a> {
        Note {
            gutter_padding,
            message,
        }
    }

    pub fn emit(&self, writer: &mut impl WriteColor, config: &Config) -> io::Result<()> {
        for (i, line) in self.message.lines().enumerate() {
            Gutter::new(None, self.gutter_padding).emit(writer, config)?;
            match i {
                0 => Bullet::new().emit(writer, config)?,
                _ => write!(writer, " ")?,
            }
            // Write line of message
            write!(writer, " {}", line)?;
            NewLine::new().emit(writer, config)?;
        }

        Ok(())
    }
}

/// The bullet that appears before a note.
///
/// ```text
/// =
/// ```
struct Bullet {}

impl Bullet {
    fn new() -> Bullet {
        Bullet {}
    }

    fn emit(&self, writer: &mut impl WriteColor, config: &Config) -> io::Result<()> {
        writer.set_color(&config.styles.note_bullet)?;
        write!(writer, "{}", config.chars.note_bullet)?;
        writer.reset()?;

        Ok(())
    }
}
