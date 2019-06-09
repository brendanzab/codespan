use codespan::Files;
use std::io;
use termcolor::{Color, WriteColor};

use crate::{Diagnostic, Severity};

mod views;

/// Configures how a diagnostic is rendered.
#[derive(Clone, Debug)]
pub struct Config {
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
    use self::views::RichDiagnostic;

    RichDiagnostic::new(files, diagnostic).emit(writer, config)
}

#[cfg(test)]
mod tests {
    use codespan::Files;

    use super::*;
    use crate::diagnostic::{Diagnostic, Label};
    use crate::termcolor::Buffer;

    fn emit_fizz_buzz(writer: &mut impl WriteColor) {
        let mut files = Files::new();

        let file_id = files.add(
            "FizzBuzz.fun",
            unindent::unindent(
                r#"
                    module FizzBuzz where

                    fizz₁ : Nat → String
                    fizz₁ num = case (mod num 5) (mod num 3) of
                        0 0 => "FizzBuzz"
                        0 _ => "Fizz"
                        _ 0 => "Buzz"
                        _ _ => num

                    fizz₂ num =
                        case (mod num 5) (mod num 3) of
                            0 0 => "FizzBuzz"
                            0 _ => "Fizz"
                            _ 0 => "Buzz"
                            _ _ => num
                "#,
            ),
        );

        let diagnostics = vec![
            // Incompatible match clause error
            Diagnostic::new_error(
                "`case` clauses have incompatible types",
                Label::new(file_id, 163..166, "expected `String`, found `Nat`"),
            )
            .with_code("E0308")
            .with_notes(vec![unindent::unindent(
                "
                    expected type `String`
                       found type `Nat`
                ",
            )])
            .with_secondary_labels(vec![
                Label::new(file_id, 62..166, "`case` clauses have incompatible types"),
                Label::new(file_id, 41..47, "expected type `String` found here"),
            ]),
            // Incompatible match clause error
            Diagnostic::new_error(
                "`case` clauses have incompatible types",
                Label::new(file_id, 303..306, "expected `String`, found `Nat`"),
            )
            .with_code("E0308")
            .with_notes(vec![unindent::unindent(
                "
                    expected type `String`
                       found type `Nat`
                ",
            )])
            .with_secondary_labels(vec![
                Label::new(file_id, 186..306, "`case` clauses have incompatible types"),
                Label::new(file_id, 233..243, "this is found to be of type `String`"),
                Label::new(file_id, 259..265, "this is found to be of type `String`"),
                Label::new(file_id, 281..287, "this is found to be of type `String`"),
            ]),
        ];

        for diagnostic in &diagnostics {
            emit(writer, &Config::default(), &files, &diagnostic).unwrap();
        }
    }

    #[test]
    fn fizz_buzz_no_color() {
        let mut buffer = Buffer::no_color();
        emit_fizz_buzz(&mut buffer);
        let result = String::from_utf8_lossy(buffer.as_slice());
        insta::assert_snapshot_matches!("fizz_buzz_no_color", result);
    }

    fn emit_tabbed(writer: &mut impl WriteColor, config: &Config) {
        let mut files = Files::new();

        let file_id = files.add(
            "FizzBuzz.fun",
            [
                "Entity:",
                "\tArmament:",
                "\t\tWeapon: DogJaw",
                "\t\tReloadingCondition:\tattack-cooldown",
                "\tFoo: Bar",
            ]
            .join("\n"),
        );

        let diagnostics = vec![
            Diagnostic::new_warning(
                "unknown weapon `DogJaw`",
                Label::new(file_id, 29..35, "the weapon"),
            ),
            Diagnostic::new_warning(
                "unknown condition `attack-cooldown`",
                Label::new(file_id, 58..73, "the condition"),
            ),
            Diagnostic::new_warning(
                "unknown field `Foo`",
                Label::new(file_id, 75..78, "the field"),
            ),
        ];

        for diagnostic in &diagnostics {
            emit(writer, config, &files, &diagnostic).unwrap();
        }
    }

    #[test]
    fn fizz_tabbed_default_no_color() {
        let config = Config::default();
        let mut buffer = Buffer::no_color();
        emit_tabbed(&mut buffer, &config);
        let result = String::from_utf8_lossy(buffer.as_slice());
        insta::assert_snapshot_matches!("tabbed_default_no_color", result);
    }

    #[test]
    fn fizz_tabbed_tab_3_no_color() {
        let config = Config {
            tab_width: 3,
            ..Config::default()
        };
        let mut buffer = Buffer::no_color();
        emit_tabbed(&mut buffer, &config);
        let result = String::from_utf8_lossy(buffer.as_slice());
        insta::assert_snapshot_matches!("tabbed_tab_3_no_color", result);
    }

    #[test]
    fn fizz_tabbed_tab_6_no_color() {
        let config = Config {
            tab_width: 6,
            ..Config::default()
        };
        let mut buffer = Buffer::no_color();
        emit_tabbed(&mut buffer, &config);
        let result = String::from_utf8_lossy(buffer.as_slice());
        insta::assert_snapshot_matches!("tabbed_tab_6_no_color", result);
    }
}
