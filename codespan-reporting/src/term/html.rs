//! HTML writer for emitting diagnostics as styled HTML.

#[cfg(not(feature = "std"))]
use alloc::format;

use crate::diagnostic::{LabelStyle, Severity};

use super::renderer::{GeneralWrite, GeneralWriteResult, WriteStyle};

/// Writer that emits diagnostics as HTML with `<span class="...">` for styling.
pub struct HtmlWriter<W> {
    upstream: W,
    span_open: bool,
}

impl<W: GeneralWrite> HtmlWriter<W> {
    pub fn new(upstream: W) -> Self {
        HtmlWriter {
            upstream,
            span_open: false,
        }
    }

    /// Get a reference to the underlying writer.
    pub fn get_ref(&self) -> &W {
        &self.upstream
    }

    /// Get a mutable reference to the underlying writer.
    pub fn get_mut(&mut self) -> &mut W {
        &mut self.upstream
    }

    /// Return the underlying writer, closing any open span first.
    #[cfg(feature = "std")]
    pub fn into_inner(mut self) -> std::io::Result<W> {
        self.close_span()?;
        Ok(self.upstream)
    }

    /// Return the underlying writer, closing any open span first.
    #[cfg(not(feature = "std"))]
    pub fn into_inner(mut self) -> Result<W, core::fmt::Error> {
        self.close_span()?;
        Ok(self.upstream)
    }

    /// Close any open span
    fn close_span(&mut self) -> GeneralWriteResult {
        if self.span_open {
            write!(self.upstream, "</span>")?;
            self.span_open = false;
        }
        Ok(())
    }

    /// Open a new span with the given CSS class
    fn open_span(&mut self, class: &str) -> GeneralWriteResult {
        self.close_span()?;
        write!(self.upstream, "<span class=\"{}\">", class)?;
        self.span_open = true;
        Ok(())
    }
}

#[cfg(feature = "std")]
impl<W: std::io::Write> std::io::Write for HtmlWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut last = 0;
        for (i, &b) in buf.iter().enumerate() {
            let escape = match b {
                b'<' => b"&lt;"[..].as_ref(),
                b'>' => b"&gt;"[..].as_ref(),
                b'&' => b"&amp;"[..].as_ref(),
                _ => continue,
            };
            self.upstream.write_all(&buf[last..i])?;
            self.upstream.write_all(escape)?;
            last = i + 1;
        }
        self.upstream.write_all(&buf[last..])?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.upstream.flush()
    }
}

#[cfg(not(feature = "std"))]
impl<W: core::fmt::Write> core::fmt::Write for HtmlWriter<W> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let mut last = 0;
        for (i, c) in s.char_indices() {
            let escape = match c {
                '<' => "&lt;",
                '>' => "&gt;",
                '&' => "&amp;",
                _ => continue,
            };
            self.upstream.write_str(&s[last..i])?;
            self.upstream.write_str(escape)?;
            last = i + c.len_utf8();
        }
        self.upstream.write_str(&s[last..])?;
        Ok(())
    }
}

impl<W: GeneralWrite> WriteStyle for HtmlWriter<W> {
    fn set_header(&mut self, severity: Severity) -> GeneralWriteResult {
        let class = match severity {
            Severity::Bug => "header-bug",
            Severity::Error => "header-error",
            Severity::Warning => "header-warning",
            Severity::Note => "header-note",
            Severity::Help => "header-help",
        };
        self.open_span(class)
    }

    fn set_header_message(&mut self) -> GeneralWriteResult {
        self.open_span("header-message")
    }

    fn set_line_number(&mut self) -> GeneralWriteResult {
        self.open_span("line-number")
    }

    fn set_note_bullet(&mut self) -> GeneralWriteResult {
        self.open_span("note-bullet")
    }

    fn set_source_border(&mut self) -> GeneralWriteResult {
        self.open_span("source-border")
    }

    fn set_label(&mut self, severity: Severity, label_style: LabelStyle) -> GeneralWriteResult {
        let sev = match severity {
            Severity::Bug => "bug",
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Note => "note",
            Severity::Help => "help",
        };
        let typ = match label_style {
            LabelStyle::Primary => "primary",
            LabelStyle::Secondary => "secondary",
        };
        self.open_span(&format!("label-{}-{}", typ, sev))
    }

    fn reset(&mut self) -> GeneralWriteResult {
        self.close_span()
    }
}
