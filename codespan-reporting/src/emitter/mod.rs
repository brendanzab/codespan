use codespan::Files;
use std::io;
use termcolor::{Color, WriteColor};

use crate::{Diagnostic, Severity};

mod views;

/// Configures how a diagnostic is rendered.
#[derive(Clone, Debug)]
pub struct Config {
    /// The color to use when rendering bugs.
    /// Defaults to `Color::Red`.
    pub bug_color: Color,
    /// The color to use when rendering errors.
    /// Defaults to `Color::Red`.
    pub error_color: Color,
    /// The color to use when rendering warnings.
    /// Defaults to `Color::Yellow`.
    pub warning_color: Color,
    /// The color to use when rendering notes.
    /// Defaults to `Color::Green`.
    pub note_color: Color,
    /// The color to use when rendering helps.
    /// Defaults to `Color::Cyan`.
    pub help_color: Color,
    /// The color to use when rendering secondary labels.
    /// Defaults `Color::Blue` (or `Color::Cyan` on windows).
    pub secondary_color: Color,
    /// The color to use when rendering the line numbers.
    /// Defaults `Color::Blue` (or `Color::Cyan` on windows).
    pub line_number_color: Color,
    /// The color to use when rendering the source code borders.
    /// Defaults `Color::Blue` (or `Color::Cyan` on windows).
    pub border_color: Color,
    /// The color to use when rendering the note bullets.
    /// Defaults `Color::Blue` (or `Color::Cyan` on windows).
    pub note_bullet_color: Color,
    /// The character to use when marking the top-left corner of the source.
    /// Defaults to: `'┌'`.
    pub border_top_left_char: char,
    /// The character to use when marking the top border of the source.
    /// Defaults to: `'─'`.
    pub border_top_char: char,
    /// The character to use when marking the left border of the source.
    /// Defaults to: `'│'`.
    pub border_left_char: char,
    /// The character to use when underlining a primary label.
    /// Defaults to: `'^'`.
    pub primary_underline_char: char,
    /// The character to use when underlining a secondary label.
    /// Defaults to: `'-'`.
    pub secondary_underline_char: char,
    /// The character to use for the note bullet.
    /// Defaults to: `'='`.
    pub note_bullet_char: char,
}

impl Default for Config {
    fn default() -> Config {
        // Blue is really difficult to see on the standard windows command line
        #[cfg(windows)]
        const BLUE: Color = Color::Cyan;
        #[cfg(not(windows))]
        const BLUE: Color = Color::Blue;

        Config {
            bug_color: Color::Red,
            error_color: Color::Red,
            warning_color: Color::Yellow,
            note_color: Color::Green,
            help_color: Color::Cyan,
            secondary_color: BLUE,
            line_number_color: BLUE,
            border_color: BLUE,
            note_bullet_color: BLUE,
            border_top_left_char: '┌',
            border_top_char: '─',
            border_left_char: '│',
            primary_underline_char: '^',
            secondary_underline_char: '-',
            note_bullet_char: '=',
        }
    }
}

impl Config {
    /// The color used to mark a given severity.
    pub fn severity_color(&self, severity: Severity) -> Color {
        match severity {
            Severity::Bug => self.bug_color,
            Severity::Error => self.error_color,
            Severity::Warning => self.warning_color,
            Severity::Note => self.note_color,
            Severity::Help => self.help_color,
        }
    }
}

pub fn emit(
    writer: &mut impl WriteColor,
    config: &Config,
    files: &Files,
    diagnostic: &Diagnostic,
) -> io::Result<()> {
    use self::views::RichDiagnostic;

    RichDiagnostic::new(files, diagnostic).emit(writer, config)
}
