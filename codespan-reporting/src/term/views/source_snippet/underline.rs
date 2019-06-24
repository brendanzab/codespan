use std::io;
use termcolor::{ColorSpec, WriteColor};

use crate::diagnostic::Severity;
use crate::term::Config;

#[derive(Copy, Clone)]
pub enum MarkStyle {
    Primary(Severity),
    Secondary,
}

impl MarkStyle {
    pub fn label_style<'config>(self, config: &'config Config) -> &'config ColorSpec {
        match self {
            MarkStyle::Primary(severity) => config.styles.primary_label(severity),
            MarkStyle::Secondary => &config.styles.secondary_label,
        }
    }

    pub fn caret_char(self, config: &Config) -> char {
        match self {
            MarkStyle::Primary(_) => config.chars.primary_caret,
            MarkStyle::Secondary => config.chars.secondary_caret,
        }
    }

    pub fn multiline_caret_char(self, config: &Config) -> char {
        match self {
            MarkStyle::Primary(_) => config.chars.multiline_primary_caret,
            MarkStyle::Secondary => config.chars.multiline_secondary_caret,
        }
    }
}

/// The underline of a single source line.
///
/// ```text
///       ^^ expected `Int` but found `String`
/// ```
pub struct Underline<'a> {
    mark_style: MarkStyle,
    prefix_source: &'a str,
    highlighted_source: &'a str,
    message: &'a str,
}

impl<'a> Underline<'a> {
    pub fn new(
        mark_style: MarkStyle,
        prefix_source: &'a str,
        highlighted_source: &'a str,
        message: &'a str,
    ) -> Underline<'a> {
        Underline {
            mark_style,
            prefix_source,
            highlighted_source,
            message,
        }
    }

    pub fn emit(&self, writer: &mut impl WriteColor, config: &Config) -> io::Result<()> {
        let prefix_len = config.width(self.prefix_source);
        write!(writer, " {space: >width$}", space = "", width = prefix_len)?;

        writer.set_color(self.mark_style.label_style(config))?;
        // We use `usize::max` here to ensure that we print at least one
        // underline character - even when we have a zero-length span.
        let underline_len = usize::max(config.width(self.highlighted_source), 1);
        for _ in 0..underline_len {
            write!(writer, "{}", self.mark_style.caret_char(config))?;
        }
        if !self.message.is_empty() {
            write!(writer, " {}", self.message)?;
        }
        writer.reset()?;

        Ok(())
    }
}

/// The top-left of a multi-line underline.
///
/// ```text
///  ╭
/// ```
pub struct UnderlineTopLeft {
    mark_style: MarkStyle,
}

impl UnderlineTopLeft {
    pub fn new(mark_style: MarkStyle) -> UnderlineTopLeft {
        UnderlineTopLeft { mark_style }
    }

    pub fn emit(&self, writer: &mut impl WriteColor, config: &Config) -> io::Result<()> {
        write!(writer, " ")?;

        writer.set_color(self.mark_style.label_style(config))?;
        write!(writer, "{}", config.chars.multiline_top_left)?;
        writer.reset()?;

        Ok(())
    }
}

/// The top of a multi-line underline.
///
/// ```text
///  ╭─────────────^
/// ```
pub struct UnderlineTop<'a> {
    mark_style: MarkStyle,
    prefix_source: &'a str,
}

impl<'a> UnderlineTop<'a> {
    pub fn new(mark_style: MarkStyle, prefix_source: &'a str) -> UnderlineTop<'a> {
        UnderlineTop {
            mark_style,
            prefix_source,
        }
    }

    pub fn emit(&self, writer: &mut impl WriteColor, config: &Config) -> io::Result<()> {
        write!(writer, " ")?;

        writer.set_color(self.mark_style.label_style(config))?;
        write!(writer, "{}", config.chars.multiline_top_left)?;
        let underline_len = config.width(self.prefix_source) + 1;
        for _ in 0..underline_len {
            write!(writer, "{}", config.chars.multiline_top)?;
        }
        write!(writer, "{}", self.mark_style.multiline_caret_char(config))?;
        writer.reset()?;

        Ok(())
    }
}

/// The left of a multi-line underline.
pub struct UnderlineLeft {
    mark_style: MarkStyle,
}

impl UnderlineLeft {
    pub fn new(mark_style: MarkStyle) -> UnderlineLeft {
        UnderlineLeft { mark_style }
    }

    pub fn emit(&self, writer: &mut impl WriteColor, config: &Config) -> io::Result<()> {
        write!(writer, " ")?;
        writer.set_color(self.mark_style.label_style(config))?;
        write!(writer, "{}", config.chars.multiline_left)?;
        writer.reset()?;

        Ok(())
    }
}

/// The bottom of a multi-line underline.
///
/// ```text
///  ╰──────────────^ `case` clauses have incompatible types
/// ```
pub struct UnderlineBottom<'a> {
    mark_style: MarkStyle,
    highlighted_source: &'a str,
    message: &'a str,
}

impl<'a> UnderlineBottom<'a> {
    pub fn new(
        mark_style: MarkStyle,
        highlighted_source: &'a str,
        message: &'a str,
    ) -> UnderlineBottom<'a> {
        UnderlineBottom {
            mark_style,
            highlighted_source,
            message,
        }
    }

    pub fn emit(&self, writer: &mut impl WriteColor, config: &Config) -> io::Result<()> {
        write!(writer, " ")?;

        writer.set_color(self.mark_style.label_style(config))?;
        write!(writer, "{}", config.chars.multiline_bottom_left)?;
        let width = config.width(self.highlighted_source);
        for _ in 0..width {
            write!(writer, "{}", config.chars.multiline_bottom)?;
        }
        write!(writer, "{}", self.mark_style.multiline_caret_char(config))?;
        if !self.message.is_empty() {
            write!(writer, " {}", self.message)?;
        }
        writer.reset()?;

        Ok(())
    }
}
