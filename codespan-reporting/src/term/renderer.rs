use std::io::{self, Write};
use std::ops::{Range, RangeTo};
use termcolor::{ColorSpec, WriteColor};

use crate::diagnostic::{LabelStyle, Severity};
use crate::files::Location;
use crate::term::{Chars, Config, Spacing, Styles};

/// The 'location focus' of a source code snippet.
pub struct Locus {
    /// The user-facing name of the file.
    pub name: String,
    /// The location.
    pub location: Location,
}

/// Single-line label, with an optional message.
///
/// ```text
/// ^^^^^^^^^ blah blah
/// ```
pub type SingleLabel<'diagnostic> = (LabelStyle, Range<usize>, &'diagnostic str);

/// A multi-line label to render.
///
/// Locations are relative to the start of where the source cord is rendered.
pub enum MultiLabel<'diagnostic> {
    /// Left top corner for multi-line labels.
    ///
    /// ```text
    /// ╭
    /// ```
    TopLeft(LabelStyle),
    /// Multi-line label top.
    ///
    /// ```text
    /// ╭────────────^
    /// ```
    Top(LabelStyle, RangeTo<usize>),
    /// Left vertical labels for multi-line labels.
    ///
    /// ```text
    /// │
    /// ```
    Left(LabelStyle),
    /// Multi-line label bottom, with an optional message.
    ///
    /// ```text
    /// ╰────────────^ blah blah
    /// ```
    Bottom(LabelStyle, RangeTo<usize>, &'diagnostic str),
}

#[derive(Copy, Clone)]
enum VerticalBound {
    Top,
    Bottom,
}

type Underline = (LabelStyle, VerticalBound);

/// A renderer of display list entries.
///
/// The following diagram gives an overview of each of the parts of the renderer's output:
///
/// ```text
///                     ┌ outer gutter
///                     │ ┌ left border
///                     │ │ ┌ inner gutter
///                     │ │ │   ┌─────────────────────────── source ─────────────────────────────┐
///                     │ │ │   │                                                                │
///                  ┌────────────────────────────────────────────────────────────────────────────
///        header ── │ error[0001]: oh noes, a cupcake has occurred!
///         empty ── │
/// snippet start ── │    ┌─ test:9:0
/// snippet empty ── │    │
///  snippet line ── │  9 │   ╭ Cupcake ipsum dolor. Sit amet marshmallow topping cheesecake
///  snippet line ── │ 10 │   │ muffin. Halvah croissant candy canes bonbon candy. Apple pie jelly
///                  │    │ ╭─│─────────^
/// snippet break ── │    · │ │
///  snippet line ── │ 33 │ │ │ Muffin danish chocolate soufflé pastry icing bonbon oat cake.
///  snippet line ── │ 34 │ │ │ Powder cake jujubes oat cake. Lemon drops tootsie roll marshmallow
///                  │    │ │ ╰─────────────────────────────^ blah blah
/// snippet break ── │    · │
///  snippet line ── │ 38 │ │   Brownie lemon drops chocolate jelly-o candy canes. Danish marzipan
///  snippet line ── │ 39 │ │   jujubes soufflé carrot cake marshmallow tiramisu caramels candy canes.
///                  │    │ │           ^^^^^^^^^^^^^^^^^^ blah blah
///                  │    │ │                               -------------------- blah blah
///  snippet line ── │ 40 │ │   Fruitcake jelly-o danish toffee. Tootsie roll pastry cheesecake
///  snippet line ── │ 41 │ │   soufflé marzipan. Chocolate bar oat cake jujubes lollipop pastry
///  snippet line ── │ 42 │ │   cupcake. Candy canes cupcake toffee gingerbread candy canes muffin
///                  │    │ │                                ^^^^^^^^^^^^^^^^^^ blah blah
///                  │    │ ╰──────────^ blah blah
/// snippet break ── │    ·
///  snippet line ── │ 82 │     gingerbread toffee chupa chups chupa chups jelly-o cotton candy.
///                  │    │                 ^^^^^^                         ------- blah blah
/// snippet empty ── │    │
///  snippet note ── │    = blah blah
///  snippet note ── │    = blah blah blah
///                  │      blah blah
///  snippet note ── │    = blah blah blah
///                  │      blah blah
///         empty ── │
/// ```
///
/// Filler text from http://www.cupcakeipsum.com
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

    /// Diagnostic header, with severity, code, and message.
    ///
    /// ```text
    /// error[E0001]: unexpected type in `+` application
    /// ```
    pub fn render_header(
        &mut self,
        locus: Option<&Locus>,
        severity: Severity,
        code: Option<&str>,
        message: &str,
    ) -> io::Result<()> {
        // Write locus
        //
        // ```text
        // test:2:9:
        // ```
        if let Some(locus) = locus {
            self.snippet_locus(locus)?;
            write!(self, ": ")?;
        }

        // Write severity name
        //
        // ```text
        // error
        // ```
        self.set_color(self.styles().header(severity))?;
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

        Ok(())
    }

    /// Empty line.
    pub fn render_empty(&mut self, spacing: Spacing) -> io::Result<()> {
        if self.config.spacing <= spacing {
            write!(self, "\n")?;
        }

        Ok(())
    }

    /// Top left border and locus.
    ///
    /// ```text
    /// ┌─ test:2:9
    /// ```
    pub fn render_snippet_start(&mut self, outer_padding: usize, locus: &Locus) -> io::Result<()> {
        self.outer_gutter(outer_padding)?;

        self.set_color(&self.styles().source_border)?;
        write!(self, "{}", self.chars().source_border_top_left)?;
        write!(self, "{0}", self.chars().source_border_top)?;
        self.reset()?;

        write!(self, " ")?;
        self.snippet_locus(&locus)?;

        write!(self, "\n")?;

        Ok(())
    }

    /// A line of source code.
    ///
    /// ```text
    /// 10 │   │ muffin. Halvah croissant candy canes bonbon candy. Apple pie jelly
    ///    │ ╭─│─────────^
    /// ```
    pub fn render_snippet_source(
        &mut self,
        outer_padding: usize,
        line_number: usize,
        source: &str,
        severity: Severity,
        single_labels: &[SingleLabel<'_>],
        num_multi_labels: usize,
        multi_labels: &[(usize, MultiLabel<'_>)],
    ) -> io::Result<()> {
        // Write source line
        //
        // ```text
        //  10 │   │ muffin. Halvah croissant candy canes bonbon candy. Apple pie jelly
        // ```
        {
            // Write outer gutter (with line number) and border
            self.outer_gutter_number(line_number, outer_padding)?;
            self.border_left()?;

            // Write inner gutter (with multi-line continuations on the left if necessary)
            let mut multi_labels_iter = multi_labels.iter().peekable();
            for label_column in 0..num_multi_labels {
                match multi_labels_iter.peek() {
                    Some((label_index, label)) if *label_index == label_column => {
                        match label {
                            MultiLabel::TopLeft(label_style) => {
                                self.label_multi_top_left(severity, *label_style)?;
                            }
                            MultiLabel::Top(..) => self.inner_gutter_space()?,
                            MultiLabel::Left(label_style) | MultiLabel::Bottom(label_style, ..) => {
                                self.label_multi_left(severity, *label_style, None)?;
                            }
                        }
                        multi_labels_iter.next();
                    }
                    Some((_, _)) | None => self.inner_gutter_space()?,
                }
            }

            // Write source
            write!(self.config.source(self.writer), " {}", source.trim_end())?;
            write!(self, "\n")?;
        }

        // Write single labels underneath source
        //
        // ```text
        //     │ │   │    ^^^^ oh noes
        // ```
        for (label_style, range, message) in single_labels.iter() {
            self.outer_gutter(outer_padding)?;
            self.border_left()?;
            self.inner_gutter(severity, num_multi_labels, multi_labels)?;
            self.label_single(severity, *label_style, source, range.clone(), message)?;
        }

        // Write top or bottom label carets underneath source
        //
        // ```text
        //     │ ╰───│──────────────────^ woops
        //     │   ╭─│─────────^
        // ```
        for (multi_label_index, (_, label)) in multi_labels.iter().enumerate() {
            let (label_style, range, bottom_message) = match label {
                MultiLabel::TopLeft(_) | MultiLabel::Left(_) => continue, // no label caret needed
                MultiLabel::Top(ls, range) => (*ls, range, None),
                MultiLabel::Bottom(ls, range, message) => (*ls, range, Some(message)),
            };

            self.outer_gutter(outer_padding)?;
            self.border_left()?;

            // Write inner gutter.
            //
            // ```text
            //  │ ╭─│───│
            // ```
            let mut underline = None;
            let mut multi_labels_iter = multi_labels.iter().enumerate().peekable();
            for label_column in 0..num_multi_labels {
                match multi_labels_iter.peek() {
                    Some((i, (label_index, label))) if *label_index == label_column => {
                        match label {
                            MultiLabel::TopLeft(ls) | MultiLabel::Left(ls) => {
                                self.label_multi_left(severity, *ls, underline.map(|(s, _)| s))?;
                            }
                            MultiLabel::Top(ls, ..) if multi_label_index > *i => {
                                self.label_multi_left(severity, *ls, underline.map(|(s, _)| s))?;
                            }
                            MultiLabel::Bottom(ls, ..) if multi_label_index < *i => {
                                self.label_multi_left(severity, *ls, underline.map(|(s, _)| s))?;
                            }
                            MultiLabel::Top(ls, ..) if multi_label_index == *i => {
                                underline = Some((*ls, VerticalBound::Top));
                                self.label_multi_top_left(severity, label_style)?
                            }
                            MultiLabel::Bottom(ls, ..) if multi_label_index == *i => {
                                underline = Some((*ls, VerticalBound::Bottom));
                                self.label_multi_bottom_left(severity, label_style)?;
                            }
                            MultiLabel::Top(..) | MultiLabel::Bottom(..) => {
                                self.inner_gutter_column(severity, underline)?;
                            }
                        }
                        multi_labels_iter.next();
                    }
                    Some((_, _)) | None => self.inner_gutter_column(severity, underline)?,
                }
            }

            // Finish the top or bottom caret
            let range = range.clone();
            match bottom_message {
                None => self.label_multi_top_caret(severity, label_style, source, range)?,
                Some(message) => {
                    self.label_multi_bottom_caret(severity, label_style, source, range, message)?
                }
            }
        }

        Ok(())
    }

    /// An empty source line, for providing additional whitespace to source snippets.
    ///
    /// ```text
    /// │ │ │
    /// ```
    pub fn render_snippet_empty(
        &mut self,
        spacing: Spacing,
        outer_padding: usize,
        severity: Severity,
        num_multi_labels: usize,
        multi_labels: &[(usize, MultiLabel<'_>)],
    ) -> io::Result<()> {
        if self.config.spacing <= spacing {
            self.outer_gutter(outer_padding)?;
            self.border_left()?;
            self.inner_gutter(severity, num_multi_labels, multi_labels)?;
            write!(self, "\n")?;
        }
        Ok(())
    }

    /// A broken source line, for labeling skipped sections of source.
    ///
    /// ```text
    /// · │ │
    /// ```
    pub fn render_snippet_break(
        &mut self,
        outer_padding: usize,
        severity: Severity,
        num_multi_labels: usize,
        multi_labels: &[(usize, MultiLabel<'_>)],
    ) -> io::Result<()> {
        self.outer_gutter(outer_padding)?;
        self.border_left_break()?;
        self.inner_gutter(severity, num_multi_labels, multi_labels)?;
        write!(self, "\n")?;
        Ok(())
    }

    /// Additional notes.
    ///
    /// ```text
    /// = expected type `Int`
    ///      found type `String`
    /// ```
    pub fn render_snippet_note(&mut self, outer_padding: usize, message: &str) -> io::Result<()> {
        for (note_line_index, line) in message.lines().enumerate() {
            self.outer_gutter(outer_padding)?;
            match note_line_index {
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

        Ok(())
    }

    /// Location focus.
    fn snippet_locus(&mut self, locus: &Locus) -> io::Result<()> {
        write!(
            self,
            "{origin}:{line_number}:{column_number}",
            origin = locus.name,
            line_number = locus.location.line_number,
            column_number = locus.location.column_number,
        )
    }

    /// The outer gutter of a source line.
    fn outer_gutter(&mut self, outer_padding: usize) -> io::Result<()> {
        if self.config.spacing <= Spacing::Cozy {
            write!(self, " ")?;
        }
        write!(self, "{space: >width$}", space = "", width = outer_padding,)?;
        write!(self, " ")?;
        Ok(())
    }

    /// The outer gutter of a source line, with line number.
    fn outer_gutter_number(&mut self, line_number: usize, outer_padding: usize) -> io::Result<()> {
        if self.config.spacing <= Spacing::Cozy {
            write!(self, " ")?;
        }
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

    /// Single-line label with a message.
    ///
    /// ```text
    /// ^^ expected `Int` but found `String`
    /// ```
    fn label_single(
        &mut self,
        severity: Severity,
        label_style: LabelStyle,
        source: &str,
        range: Range<usize>,
        message: &str,
    ) -> io::Result<()> {
        let space_len = self.config.width(&source[..range.start]);
        write!(self, " {space: >width$}", space = "", width = space_len)?;
        self.set_color(self.styles().label(severity, label_style))?;
        // We use `usize::max` here to ensure that we print at least one
        // label character - even when we have a zero-length span.
        for _ in 0..usize::max(self.config.width(&source[range.clone()]), 1) {
            write!(self, "{}", self.chars().single_caret_char(label_style))?;
        }
        if !message.is_empty() {
            write!(self, " {}", message)?;
        }
        self.reset()?;
        write!(self, "\n")?;
        Ok(())
    }

    /// The left of a multi-line label.
    ///
    /// ```text
    ///  │
    /// ```
    fn label_multi_left(
        &mut self,
        severity: Severity,
        label_style: LabelStyle,
        underline: Option<LabelStyle>,
    ) -> io::Result<()> {
        match underline {
            None => write!(self, " ")?,
            // Continue an underline horizontally
            Some(label_style) => {
                self.set_color(self.styles().label(severity, label_style))?;
                write!(self, "{}", self.chars().multi_top)?;
                self.reset()?;
            }
        }
        self.set_color(self.styles().label(severity, label_style))?;
        write!(self, "{}", self.chars().multi_left)?;
        self.reset()?;
        Ok(())
    }

    /// The top of a multi-line label.
    fn label_multi_top_line(
        &mut self,
        severity: Severity,
        label_style: LabelStyle,
        len: usize,
    ) -> io::Result<()> {
        self.set_color(self.styles().label(severity, label_style))?;
        for _ in 0..len {
            write!(self, "{}", self.config.chars.multi_top)?;
        }
        self.reset()?;
        Ok(())
    }

    /// The top of a multi-line label.
    fn label_multi_bottom_line(
        &mut self,
        severity: Severity,
        label_style: LabelStyle,
        len: usize,
    ) -> io::Result<()> {
        self.set_color(self.styles().label(severity, label_style))?;
        for _ in 0..len {
            write!(self, "{}", self.config.chars.multi_bottom)?;
        }
        self.reset()?;
        Ok(())
    }

    /// The top-left of a multi-line label.
    ///
    /// ```text
    ///  ╭
    /// ```
    fn label_multi_top_left(
        &mut self,
        severity: Severity,
        label_style: LabelStyle,
    ) -> io::Result<()> {
        write!(self, " ")?;
        self.set_color(self.styles().label(severity, label_style))?;
        write!(self, "{}", self.chars().multi_top_left)?;
        self.reset()?;
        Ok(())
    }

    /// The bottom left of a multi-line label.
    ///
    /// ```text
    ///  ╰
    /// ```
    fn label_multi_bottom_left(
        &mut self,
        severity: Severity,
        label_style: LabelStyle,
    ) -> io::Result<()> {
        write!(self, " ")?;
        self.set_color(self.styles().label(severity, label_style))?;
        write!(self, "{}", self.chars().multi_bottom_left)?;
        self.reset()?;
        Ok(())
    }

    /// Multi-line label top.
    ///
    /// ```text
    /// ─────────────^
    /// ```
    fn label_multi_top_caret(
        &mut self,
        severity: Severity,
        label_style: LabelStyle,
        source: &str,
        range: RangeTo<usize>,
    ) -> io::Result<()> {
        self.set_color(self.styles().label(severity, label_style))?;
        for _ in 0..(self.config.width(&source[range.clone()]) + 1) {
            write!(self, "{}", self.chars().multi_top)?;
        }
        write!(self, "{}", self.chars().multi_caret_char_start(label_style))?;
        self.reset()?;
        write!(self, "\n")?;
        Ok(())
    }

    /// Multi-line label bottom, with a message.
    ///
    /// ```text
    /// ─────────────^ expected `Int` but found `String`
    /// ```
    fn label_multi_bottom_caret(
        &mut self,
        severity: Severity,
        label_style: LabelStyle,
        source: &str,
        range: RangeTo<usize>,
        message: &str,
    ) -> io::Result<()> {
        self.set_color(self.styles().label(severity, label_style))?;
        for _ in 0..self.config.width(&source[range.clone()]) {
            write!(self, "{}", self.chars().multi_bottom)?;
        }
        write!(self, "{}", self.chars().multi_caret_char_end(label_style))?;
        if !message.is_empty() {
            write!(self, " {}", message)?;
        }
        self.reset()?;
        write!(self, "\n")?;
        Ok(())
    }

    /// Writes an empty gutter space, or continues an underline horizontally.
    fn inner_gutter_column(
        &mut self,
        severity: Severity,
        underline: Option<Underline>,
    ) -> io::Result<()> {
        match underline {
            None => self.inner_gutter_space(),
            Some((ls, VerticalBound::Top)) => self.label_multi_top_line(severity, ls, 2),
            Some((ls, VerticalBound::Bottom)) => self.label_multi_bottom_line(severity, ls, 2),
        }
    }

    /// Writes an empty gutter space.
    fn inner_gutter_space(&mut self) -> io::Result<()> {
        write!(self, "  ")
    }

    /// Writes an inner gutter, with the left lines if necessary.
    fn inner_gutter(
        &mut self,
        severity: Severity,
        num_multi_labels: usize,
        multi_labels: &[(usize, MultiLabel<'_>)],
    ) -> io::Result<()> {
        let mut multi_labels_iter = multi_labels.iter().peekable();
        for label_column in 0..num_multi_labels {
            match multi_labels_iter.peek() {
                Some((label_index, label)) if *label_index == label_column => match label {
                    MultiLabel::TopLeft(label_style)
                    | MultiLabel::Left(label_style)
                    | MultiLabel::Bottom(label_style, ..) => {
                        self.label_multi_left(severity, *label_style, None)?;
                        multi_labels_iter.next();
                    }
                    MultiLabel::Top(..) => {
                        self.inner_gutter_space()?;
                        multi_labels_iter.next();
                    }
                },
                Some((_, _)) | None => self.inner_gutter_space()?,
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
