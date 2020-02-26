use std::io::{self, Write};
use std::ops::{Range, RangeTo};
use termcolor::{ColorSpec, WriteColor};

use crate::diagnostic::Severity;
use crate::term::display_list::{Entry, Locus, Mark, MarkSeverity};
use crate::term::{Chars, Config, Styles};

/// A renderer of display list entries.
pub struct Renderer<'writer, 'config> {
    writer: &'writer mut dyn WriteColor,
    config: &'config Config,
}

impl<'writer, 'config> Renderer<'writer, 'config> {
    /// Construct a renderer from the given writer and config.
    pub fn new(
        writer: &'writer mut dyn WriteColor,
        config: &'config Config,
    ) -> Renderer<'writer, 'config> {
        Renderer { writer, config }
    }

    fn chars(&self) -> &'config Chars {
        &self.config.chars
    }

    fn styles(&self) -> &'config Styles {
        &self.config.styles
    }

    /// Render a display list entry to the writer.
    pub fn render(&mut self, entry: &Entry<'_, '_>) -> io::Result<()> {
        match entry {
            // Diagnostic header, with severity, code, and message.
            //
            // ```text
            // error[E0001]: unexpected type in `+` application
            // ```
            Entry::Header {
                locus,
                severity,
                code,
                message,
            } => {
                // Write locus
                //
                // ```text
                // test:2:9:
                // ```
                if let Some(locus) = locus {
                    self.source_locus(locus)?;
                    write!(self, ": ")?;
                }

                // Write severity name
                //
                // ```text
                // error
                // ```
                self.set_color(self.styles().header(*severity))?;
                match severity {
                    Severity::Bug => write!(self, "bug")?,
                    Severity::Error => write!(self, "error")?,
                    Severity::Warning => write!(self, "warning")?,
                    Severity::Help => write!(self, "help")?,
                    Severity::Note => write!(self, "note")?,
                }

                // Write error code
                //
                // ```text
                // [E0001]
                // ```
                if let Some(code) = &code {
                    write!(self, "[{}]", code)?;
                }

                // Write diagnostic message
                //
                // ```text
                // : unexpected type in `+` application
                // ```
                self.set_color(&self.styles().header_message)?;
                write!(self, ": {}", message)?;
                self.reset()?;

                write!(self, "\n")?;
            }

            // Empty line.
            Entry::Empty => {
                write!(self, "\n")?;
            }

            // Top left border and locus.
            //
            // ```text
            // ┌── test:2:9 ───
            // ```
            Entry::SourceStart {
                outer_padding,
                locus,
            } => {
                self.outer_gutter(*outer_padding)?;

                self.set_color(&self.styles().source_border)?;
                write!(self, "{}", self.chars().source_border_top_left)?;
                write!(self, "{0}{0}", self.chars().source_border_top)?;
                self.reset()?;

                write!(self, " ")?;
                self.source_locus(locus)?;
                write!(self, " ")?;

                self.set_color(&self.styles().source_border)?;
                write!(self, "{0}{0}{0}", self.chars().source_border_top)?;
                self.reset()?;
                write!(self, "\n")?;
            }

            // A line of source code.
            //
            // ```text
            // 10 │   │ muffin. Halvah croissant candy canes bonbon candy. Apple pie jelly
            //    │ ╭─│─────────^
            // ```
            Entry::SourceLine {
                outer_padding,
                line_number,
                marks,
                source,
            } => {
                // Write source line
                //
                // ```text
                //  10 │   │ muffin. Halvah croissant candy canes bonbon candy. Apple pie jelly
                // ```
                {
                    // Write outer gutter (with line number) and border
                    self.outer_gutter_number(*line_number, *outer_padding)?;
                    self.border_left()?;

                    // Write inner gutter (with multi-line continuations on the left if necessary)
                    for mark in marks {
                        match mark {
                            Some((_, Mark::Single(..))) => {}
                            // Write a top-left mark
                            Some((severity, Mark::MultiTopLeft)) => {
                                self.mark_multi_top_left(*severity)?;
                            }
                            // Write a left mark
                            Some((severity, Mark::MultiLeft))
                            | Some((severity, Mark::MultiBottom(_, _))) => {
                                self.mark_multi_left(*severity, None)?;
                            }
                            // Write a space
                            Some((_, Mark::MultiTop(..))) | None => write!(self, "  ")?,
                        }
                    }

                    // Write source
                    write!(self.config.source(self.writer), " {}", source.trim_end())?;
                    write!(self, "\n")?;
                }

                // Write marks underneath source
                //
                // ```text
                //     │ │   │    ^^^^ oh noes
                //     │ ╰───│──────────────────^ woops
                //     │   ╭─│─────────^
                // ```
                for (i, styled_mark) in marks.iter().enumerate() {
                    // No marks needed for gaps.
                    let (severity, mark) = match styled_mark {
                        None => continue,
                        Some((severity, mark)) => (severity, mark),
                    };

                    match mark {
                        Mark::Single(range, message) => {
                            self.outer_gutter(*outer_padding)?;
                            self.border_left()?;
                            self.mark_inner_gutter(i, marks)?;
                            self.mark_single(*severity, source, range.clone(), message)?;
                        }
                        Mark::MultiTopLeft => {} // SKIP: no mark needed
                        Mark::MultiTop(range) => {
                            self.outer_gutter(*outer_padding)?;
                            self.border_left()?;
                            self.mark_inner_gutter(i, marks)?;
                            self.mark_multi_top(*severity, source, range.clone())?;
                        }
                        Mark::MultiLeft => {} // SKIP: no mark needed
                        Mark::MultiBottom(range, message) => {
                            let range = range.clone();
                            self.outer_gutter(*outer_padding)?;
                            self.border_left()?;
                            self.mark_inner_gutter(i, marks)?;
                            self.mark_multi_bottom(*severity, source, range, message)?;
                        }
                    }
                }
            }

            // An empty source line, for providing additional whitespace to source snippets.
            //
            // ```text
            // │ │ │
            // ```
            Entry::SourceEmpty {
                outer_padding,
                left_marks,
            } => {
                self.outer_gutter(*outer_padding)?;
                self.border_left()?;
                for left_severity in left_marks {
                    match left_severity {
                        None => write!(self, "  ")?,
                        Some(severity) => self.mark_multi_left(*severity, None)?,
                    }
                }
                write!(self, "\n")?;
            }

            // A broken source line, for marking skipped sections of source.
            //
            // ```text
            // · │ │
            // ```
            Entry::SourceBreak {
                outer_padding,
                left_marks,
            } => {
                self.outer_gutter(*outer_padding)?;
                self.border_left_break()?;
                for left_severity in left_marks {
                    match left_severity {
                        None => write!(self, "  ")?,
                        Some(severity) => self.mark_multi_left(*severity, None)?,
                    }
                }
                write!(self, "\n")?;
            }

            // Additional notes.
            //
            // ```text
            // = expected type `Int`
            //      found type `String`
            // ```
            Entry::SourceNote {
                outer_padding,
                message,
            } => {
                for (i, line) in message.lines().enumerate() {
                    self.outer_gutter(*outer_padding)?;
                    match i {
                        0 => {
                            self.set_color(&self.styles().note_bullet)?;
                            write!(self, "{}", self.chars().note_bullet)?;
                            self.reset()?;
                        }
                        _ => write!(self, " ")?,
                    }
                    // Write line of message
                    write!(self, " {}", line)?;
                    write!(self, "\n")?;
                }
            }
        }

        Ok(())
    }

    /// Location focus.
    fn source_locus(&mut self, locus: &Locus) -> io::Result<()> {
        write!(
            self,
            "{origin}:{line_number}:{column_number}",
            origin = locus.origin,
            line_number = locus.line_number,
            column_number = locus.column_number,
        )
    }

    /// The outer gutter of a source line.
    fn outer_gutter(&mut self, outer_padding: usize) -> io::Result<()> {
        write!(self, " ")?;
        write!(self, "{space: >width$}", space = "", width = outer_padding,)?;
        write!(self, " ")?;
        Ok(())
    }

    /// The outer gutter of a source line, with line number.
    fn outer_gutter_number(&mut self, line_number: usize, outer_padding: usize) -> io::Result<()> {
        write!(self, " ")?;
        self.set_color(&self.styles().line_number)?;
        write!(
            self,
            "{line_number: >width$}",
            line_number = line_number,
            width = outer_padding,
        )?;
        self.reset()?;
        write!(self, " ")?;
        Ok(())
    }

    /// The left-hand border of a source line.
    fn border_left(&mut self) -> io::Result<()> {
        self.set_color(&self.styles().source_border)?;
        write!(self, "{}", self.chars().source_border_left)?;
        self.reset()?;
        Ok(())
    }

    /// The broken left-hand border of a source line.
    fn border_left_break(&mut self) -> io::Result<()> {
        self.set_color(&self.styles().source_border)?;
        write!(self, "{}", self.chars().source_border_left_break)?;
        self.reset()?;
        Ok(())
    }

    // Single-line mark with a message.
    //
    // ```text
    // ^^ expected `Int` but found `String`
    // ```
    fn mark_single(
        &mut self,
        severity: MarkSeverity,
        source: &str,
        range: Range<usize>,
        message: &str,
    ) -> io::Result<()> {
        let space_len = self.config.width(&source[..range.start]);
        write!(self, " {space: >width$}", space = "", width = space_len)?;
        self.set_color(self.styles().label(severity))?;
        // We use `usize::max` here to ensure that we print at least one
        // mark character - even when we have a zero-length span.
        for _ in 0..usize::max(self.config.width(&source[range.clone()]), 1) {
            write!(self, "{}", self.chars().caret_char(severity))?;
        }
        if !message.is_empty() {
            write!(self, " {}", message)?;
        }
        self.reset()?;
        write!(self, "\n")?;
        Ok(())
    }

    /// The left of a multi-line mark.
    ///
    /// ```text
    ///  │
    /// ```
    fn mark_multi_left(
        &mut self,
        severity: MarkSeverity,
        current_severity: Option<MarkSeverity>,
    ) -> io::Result<()> {
        match current_severity {
            None => write!(self, " ")?,
            // Continue a projected mark horizontally
            Some(severity) => {
                self.set_color(self.styles().label(severity))?;
                write!(self, "{}", self.chars().multi_top)?;
                self.reset()?;
            }
        }
        self.set_color(self.styles().label(severity))?;
        write!(self, "{}", self.chars().multi_left)?;
        self.reset()?;
        Ok(())
    }

    /// The top-left of a multi-line mark.
    ///
    /// ```text
    ///  ╭
    /// ```
    fn mark_multi_top_left(&mut self, severity: MarkSeverity) -> io::Result<()> {
        write!(self, " ")?;
        self.set_color(self.styles().label(severity))?;
        write!(self, "{}", self.chars().multi_top_left)?;
        self.reset()?;
        Ok(())
    }

    /// The bottom left of a multi-line mark.
    ///
    /// ```text
    ///  ╰
    /// ```
    fn mark_multi_bottom_left(&mut self, severity: MarkSeverity) -> io::Result<()> {
        write!(self, " ")?;
        self.set_color(self.styles().label(severity))?;
        write!(self, "{}", self.chars().multi_bottom_left)?;
        self.reset()?;
        Ok(())
    }

    // Multi-line mark top.
    //
    // ```text
    // ─────────────^
    // ```
    fn mark_multi_top(
        &mut self,
        severity: MarkSeverity,
        source: &str,
        range: RangeTo<usize>,
    ) -> io::Result<()> {
        self.set_color(self.styles().label(severity))?;
        for _ in 0..(self.config.width(&source[range.clone()]) + 1) {
            write!(self, "{}", self.chars().multi_top)?;
        }
        write!(self, "{}", self.chars().multi_caret_char(severity))?;
        self.reset()?;
        write!(self, "\n")?;
        Ok(())
    }

    // Multi-line mark bottom, with a message.
    //
    // ```text
    // ─────────────^ expected `Int` but found `String`
    // ```
    fn mark_multi_bottom(
        &mut self,
        severity: MarkSeverity,
        source: &str,
        range: RangeTo<usize>,
        message: &str,
    ) -> io::Result<()> {
        self.set_color(self.styles().label(severity))?;
        for _ in 0..self.config.width(&source[range.clone()]) {
            write!(self, "{}", self.chars().multi_bottom)?;
        }
        write!(self, "{}", self.chars().multi_caret_char(severity))?;
        if !message.is_empty() {
            write!(self, " {}", message)?;
        }
        self.reset()?;
        write!(self, "\n")?;
        Ok(())
    }
    /// Writes an empty gutter space, or continues a projected mark horizontally.
    fn mark_inner_gutter_space(
        &mut self,
        current_severity: Option<MarkSeverity>,
    ) -> io::Result<()> {
        match current_severity {
            None => write!(self, "  ")?,
            // Continue a projected mark horizontally
            Some(severity) => {
                self.set_color(self.styles().label(severity))?;
                write!(self, "{0}{0}", self.chars().multi_top)?;
                self.reset()?;
            }
        }
        Ok(())
    }

    /// Writes an inner gutter.
    ///
    /// ```text
    ///  │ ╭─│───│
    /// ```
    fn mark_inner_gutter(
        &mut self,
        current_mark_index: usize,
        marks: &[Option<(MarkSeverity, Mark<'_>)>],
    ) -> io::Result<()> {
        let mut current_severity = None;

        for (i, mark) in marks.iter().enumerate() {
            match mark {
                None => self.mark_inner_gutter_space(current_severity)?,
                Some((severity, mark)) => match mark {
                    Mark::Single(..) => {}
                    Mark::MultiTopLeft | Mark::MultiLeft => {
                        self.mark_multi_left(*severity, current_severity)?;
                    }
                    Mark::MultiTop(..) if current_mark_index > i => {
                        self.mark_multi_left(*severity, current_severity)?;
                    }
                    Mark::MultiBottom(..) if current_mark_index < i => {
                        self.mark_multi_left(*severity, current_severity)?;
                    }
                    Mark::MultiTop(..) if current_mark_index == i => {
                        current_severity = Some(*severity);
                        self.mark_multi_top_left(*severity)?
                    }
                    Mark::MultiBottom(..) if current_mark_index == i => {
                        current_severity = Some(*severity);
                        self.mark_multi_bottom_left(*severity)?;
                    }
                    Mark::MultiTop(..) | Mark::MultiBottom(..) => {
                        self.mark_inner_gutter_space(current_severity)?;
                    }
                },
            }
        }

        Ok(())
    }
}

impl<'writer, 'config> Write for Renderer<'writer, 'config> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.writer.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

impl<'writer, 'config> WriteColor for Renderer<'writer, 'config> {
    fn supports_color(&self) -> bool {
        self.writer.supports_color()
    }

    fn set_color(&mut self, spec: &ColorSpec) -> io::Result<()> {
        self.writer.set_color(spec)
    }

    fn reset(&mut self) -> io::Result<()> {
        self.writer.reset()
    }

    fn is_synchronous(&self) -> bool {
        self.writer.is_synchronous()
    }
}
