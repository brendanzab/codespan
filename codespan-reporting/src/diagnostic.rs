//! Diagnostic reporting support for the codespan crate

use codespan::Span;

use Severity;

/// A style for the label
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum LabelStyle {
    Primary,
    Secondary,
}

/// A label describing an underlined region of code associated with a diagnostic
#[derive(Clone, Debug)]
pub struct Label {
    /// The span we are going to include in the final snippet.
    pub span: Span,
    /// A message to provide some additional information for the underlined code.
    pub message: Option<String>,
    /// The style to use for the label.
    pub style: LabelStyle,
}

/// Represents a diagnostic message and associated child messages.
#[derive(Clone, Debug)]
pub struct Diagnostic {
    /// The overall severity of the diagnostic
    pub severity: Severity,
    /// The main message associated with this diagnostic
    pub message: String,
    /// The labelled spans marking the regions of code that cause this
    /// diagnostic to be raised
    pub labels: Vec<Label>,
}
