use std::io;
use termcolor::WriteColor;

use crate::diagnostic::Severity;
use crate::term::Config;

#[derive(Copy, Clone)]
pub enum MarkStyle {
    Primary(Severity),
    Secondary,
}

/// The underline of a single source line.
///
/// ```text
///       ^^ expected `Int` but found `String`
/// ```
pub struct Underline<'a> {
    mark_style: MarkStyle,
    source_prefix: &'a str,
    marked_source: &'a str,
    message: &'a str,
}

impl<'a> Underline<'a> {
    pub fn new(
        mark_style: MarkStyle,
        source_prefix: &'a str,
        marked_source: &'a str,
        message: &'a str,
    ) -> Underline<'a> {
        Underline {
            mark_style,
            source_prefix,
            marked_source,
            message,
        }
    }

    pub fn emit(&self, writer: &mut impl WriteColor, config: &Config) -> io::Result<()> {
        let prefix_len = config.width(self.source_prefix);
        write!(writer, " {space: >width$}", space = "", width = prefix_len)?;

        writer.set_color(self.mark_style.label_style(config))?;
        // We use `usize::max` here to ensure that we print at least one
        // underline character - even when we have a zero-length span.
        let underline_len = usize::max(config.width(self.marked_source), 1);
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
        write!(writer, "{}", config.multiline_top_left_char)?;
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
    source_prefix: &'a str,
}

impl<'a> UnderlineTop<'a> {
    pub fn new(mark_style: MarkStyle, source_prefix: &'a str) -> UnderlineTop<'a> {
        UnderlineTop {
            mark_style,
            source_prefix,
        }
    }

    pub fn emit(&self, writer: &mut impl WriteColor, config: &Config) -> io::Result<()> {
        write!(writer, " ")?;

        writer.set_color(self.mark_style.label_style(config))?;
        write!(writer, "{}", config.multiline_top_left_char)?;
        let underline_len = config.width(self.source_prefix) + 1;
        for _ in 0..underline_len {
            write!(writer, "{}", config.multiline_top_char)?;
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
        write!(writer, "{}", config.multiline_left_char)?;
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
    marked_source: &'a str,
    message: &'a str,
}

impl<'a> UnderlineBottom<'a> {
    pub fn new(
        mark_style: MarkStyle,
        marked_source: &'a str,
        message: &'a str,
    ) -> UnderlineBottom<'a> {
        UnderlineBottom {
            mark_style,
            marked_source,
            message,
        }
    }

    pub fn emit(&self, writer: &mut impl WriteColor, config: &Config) -> io::Result<()> {
        write!(writer, " ")?;

        writer.set_color(self.mark_style.label_style(config))?;
        write!(writer, "{}", config.multiline_bottom_left_char)?;
        let width = config.width(self.marked_source);
        for _ in 0..width {
            write!(writer, "{}", config.multiline_bottom_char)?;
        }
        write!(writer, "{}", self.mark_style.multiline_caret_char(config))?;
        if !self.message.is_empty() {
            write!(writer, " {}", self.message)?;
        }
        writer.reset()?;

        Ok(())
    }
}
