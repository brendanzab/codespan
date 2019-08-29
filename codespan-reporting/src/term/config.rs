use termcolor::{Color, ColorSpec};

use crate::diagnostic::Severity;

/// Configures how a diagnostic is rendered.
#[derive(Clone, Debug)]
pub struct Config {
    /// The display style to use when rendering diagnostics.
    /// Defaults to: `DisplayStyle::Rich`.
    pub display_style: DisplayStyle,
    /// Column width of tabs.
    /// Defaults to: `4`.
    pub tab_width: usize,
    /// Styles to use when rendering the diagnostic.
    pub styles: Styles,
    /// Characters to use when rendering the diagnostic.
    pub chars: Chars,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            display_style: DisplayStyle::Rich,
            tab_width: 4,
            styles: Styles::default(),
            chars: Chars::default(),
        }
    }
}

impl Config {
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

/// Styles to use when rendering the diagnostic.
#[derive(Clone, Debug)]
pub struct Styles {
    /// The style to use when rendering bug headers.
    /// Defaults to `fg:red bold intense`.
    pub header_bug: ColorSpec,
    /// The style to use when rendering error headers.
    /// Defaults to `fg:red bold intense`.
    pub header_error: ColorSpec,
    /// The style to use when rendering warning headers.
    /// Defaults to `fg:yellow bold intense`.
    pub header_warning: ColorSpec,
    /// The style to use when rendering note headers.
    /// Defaults to `fg:green bold intense`.
    pub header_note: ColorSpec,
    /// The style to use when rendering help headers.
    /// Defaults to `fg:cyan bold intense`.
    pub header_help: ColorSpec,
    /// The style to use when the main diagnostic message.
    /// Defaults to `bold intense`.
    pub header_message: ColorSpec,

    /// The style to use when rendering bug labels.
    /// Defaults to `fg:red`.
    pub primary_label_bug: ColorSpec,
    /// The style to use when rendering error labels.
    /// Defaults to `fg:red`.
    pub primary_label_error: ColorSpec,
    /// The style to use when rendering warning labels.
    /// Defaults to `fg:yellow`.
    pub primary_label_warning: ColorSpec,
    /// The style to use when rendering note labels.
    /// Defaults to `fg:green`.
    pub primary_label_note: ColorSpec,
    /// The style to use when rendering help labels.
    /// Defaults to `fg:cyan`.
    pub primary_label_help: ColorSpec,
    /// The style to use when rendering secondary labels.
    /// Defaults `fg:blue` (or `fg:cyan` on windows).
    pub secondary_label: ColorSpec,

    /// The style to use when rendering the line numbers.
    /// Defaults `fg:blue` (or `fg:cyan` on windows).
    pub line_number: ColorSpec,
    /// The style to use when rendering the source code borders.
    /// Defaults `fg:blue` (or `fg:cyan` on windows).
    pub source_border: ColorSpec,
    /// The style to use when rendering the note bullets.
    /// Defaults `fg:blue` (or `fg:cyan` on windows).
    pub note_bullet: ColorSpec,
}

impl Styles {
    /// The style used to mark a header at a given severity.
    pub fn header(&self, severity: Severity) -> &ColorSpec {
        match severity {
            Severity::Bug => &self.header_bug,
            Severity::Error => &self.header_error,
            Severity::Warning => &self.header_warning,
            Severity::Note => &self.header_note,
            Severity::Help => &self.header_help,
        }
    }

    /// The style used to mark a primary label at a given severity.
    pub fn primary_label(&self, severity: Severity) -> &ColorSpec {
        match severity {
            Severity::Bug => &self.primary_label_bug,
            Severity::Error => &self.primary_label_error,
            Severity::Warning => &self.primary_label_warning,
            Severity::Note => &self.primary_label_note,
            Severity::Help => &self.primary_label_help,
        }
    }
}

impl Default for Styles {
    fn default() -> Styles {
        // Blue is really difficult to see on the standard windows command line
        #[cfg(windows)]
        const BLUE: Color = Color::Cyan;
        #[cfg(not(windows))]
        const BLUE: Color = Color::Blue;

        let header = ColorSpec::new().set_bold(true).set_intense(true).clone();

        Styles {
            header_bug: header.clone().set_fg(Some(Color::Red)).clone(),
            header_error: header.clone().set_fg(Some(Color::Red)).clone(),
            header_warning: header.clone().set_fg(Some(Color::Yellow)).clone(),
            header_note: header.clone().set_fg(Some(Color::Green)).clone(),
            header_help: header.clone().set_fg(Some(Color::Cyan)).clone(),
            header_message: header.clone(),

            primary_label_bug: ColorSpec::new().set_fg(Some(Color::Red)).clone(),
            primary_label_error: ColorSpec::new().set_fg(Some(Color::Red)).clone(),
            primary_label_warning: ColorSpec::new().set_fg(Some(Color::Yellow)).clone(),
            primary_label_note: ColorSpec::new().set_fg(Some(Color::Green)).clone(),
            primary_label_help: ColorSpec::new().set_fg(Some(Color::Cyan)).clone(),
            secondary_label: ColorSpec::new().set_fg(Some(BLUE)).clone(),

            line_number: ColorSpec::new().set_fg(Some(BLUE)).clone(),
            source_border: ColorSpec::new().set_fg(Some(BLUE)).clone(),
            note_bullet: ColorSpec::new().set_fg(Some(BLUE)).clone(),
        }
    }
}

/// Characters to use when rendering the diagnostic.
#[derive(Clone, Debug)]
pub struct Chars {
    /// The character to use for the top-left border of the source.
    /// Defaults to: `'┌'`.
    pub source_border_top_left: char,
    /// The character to use for the top border of the source.
    /// Defaults to: `'─'`.
    pub source_border_top: char,
    /// The character to use for the left border of the source.
    /// Defaults to: `'│'`.
    pub source_border_left: char,
    /// The character to use for the left border break of the source.
    /// Defaults to: `'·'`.
    pub source_border_left_break: char,

    /// The character to use for the note bullet.
    /// Defaults to: `'='`.
    pub note_bullet: char,

    /// The character to use for marking a primary label.
    /// Defaults to: `'^'`.
    pub primary_caret: char,
    /// The character to use for marking a secondary label.
    /// Defaults to: `'-'`.
    pub secondary_caret: char,

    /// The character to use for marking the ends of a multi-line primary label.
    /// Defaults to: `'^'`.
    pub multiline_primary_caret: char,
    /// The character to use for marking the ends of a multi-line secondary label.
    /// Defaults to: `'\''`.
    pub multiline_secondary_caret: char,
    /// The character to use for the top-left corner of a multi-line label.
    /// Defaults to: `'╭'`.
    pub multiline_top_left: char,
    /// The character to use for the top of a multi-line label.
    /// Defaults to: `'─'`.
    pub multiline_top: char,
    /// The character to use for the bottom-left corner of a multi-line label.
    /// Defaults to: `'╰'`.
    pub multiline_bottom_left: char,
    /// The character to use when marking the bottom of a multi-line label.
    /// Defaults to: `'─'`.
    pub multiline_bottom: char,
    /// The character to use for the left of a multi-line label.
    /// Defaults to: `'│'`.
    pub multiline_left: char,
}

impl Default for Chars {
    fn default() -> Chars {
        Chars {
            source_border_top_left: '┌',
            source_border_top: '─',
            source_border_left: '│',
            source_border_left_break: '·',

            note_bullet: '=',

            primary_caret: '^',
            secondary_caret: '-',

            multiline_primary_caret: '^',
            multiline_secondary_caret: '\'',
            multiline_top_left: '╭',
            multiline_top: '─',
            multiline_bottom_left: '╰',
            multiline_bottom: '─',
            multiline_left: '│',
        }
    }
}
