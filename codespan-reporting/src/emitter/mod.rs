use codespan::Files;
use std::io;
use termcolor::{Color, WriteColor};

use crate::{Diagnostic, Severity};

mod views;

/// The display style to use when rendering diagnostics.
#[derive(Clone, Debug)]
pub enum DisplayStyle {
    /// Output a richly formatted diagnostic, with source code previews.
    ///
    /// ```text
    /// error[E0001]: unexpected type in `+` application
    ///
    ///    ┌── test:2:9 ───
    ///    │
    ///  2 │ (+ test "")
    ///    │         ^^ expected `Int` but found `String`
    ///    │
    ///    = expected type `Int`
    ///         found type `String`
    /// ```
    Rich,
    /// Output a short diagnostic, with a line number, severity, and message.
    ///
    /// ```text
    /// test:2:9: error[E0001]: unexpected type in `+` application
    /// ```
    Short,
}

/// Configures how a diagnostic is rendered.
#[derive(Clone, Debug)]
pub struct Config {
    /// The display style to use when rendering diagnostics.
    /// Defaults to: `DisplayStyle::Rich`.
    pub display_style: DisplayStyle,
    /// Column width of tabs.
    /// Defaults to: `4`.
    pub tab_width: usize,
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
            display_style: DisplayStyle::Rich,
            tab_width: 4,
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

    /// Measure the width of a string, taking into account the tab width.
    pub fn width(&self, s: &str) -> usize {
        use unicode_width::UnicodeWidthChar;

        s.chars()
            .map(|ch| match ch {
                '\t' => self.tab_width,
                _ => ch.width().unwrap_or(0),
            })
            .sum()
    }

    /// Get the amount of spaces we should use for printing tabs.
    pub fn tab_padding(&self) -> String {
        (0..self.tab_width).map(|_| ' ').collect()
    }
}

pub fn emit(
    writer: &mut impl WriteColor,
    config: &Config,
    files: &Files,
    diagnostic: &Diagnostic,
) -> io::Result<()> {
    use self::views::{RichDiagnostic, ShortDiagnostic};

    match config.display_style {
        DisplayStyle::Rich => RichDiagnostic::new(files, diagnostic).emit(writer, config),
        DisplayStyle::Short => ShortDiagnostic::new(files, diagnostic).emit(writer, config),
    }
}
