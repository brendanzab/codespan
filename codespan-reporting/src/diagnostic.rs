//! Diagnostic reporting support for the codespan crate

use codespan::ByteSpan;
#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};

use crate::Severity;

/// A style for the label
#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "memory_usage", derive(heapsize_derive::HeapSizeOf))]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub enum LabelStyle {
    /// The main focus of the diagnostic
    Primary,
    /// Supporting labels that may help to isolate the cause of the diagnostic
    Secondary,
}

/// A label describing an underlined region of code associated with a diagnostic
#[derive(Clone, Debug)]
#[cfg_attr(feature = "memory_usage", derive(heapsize_derive::HeapSizeOf))]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub struct Label {
    /// The span we are going to include in the final snippet.
    pub span: ByteSpan,
    /// A message to provide some additional information for the underlined code.
    pub message: String,
    /// The style to use for the label.
    pub style: LabelStyle,
}

impl Label {
    pub fn new(span: ByteSpan, style: LabelStyle) -> Label {
        Label {
            span,
            message: String::new(),
            style,
        }
    }

    pub fn new_primary(span: ByteSpan) -> Label {
        Label::new(span, LabelStyle::Primary)
    }

    pub fn new_secondary(span: ByteSpan) -> Label {
        Label::new(span, LabelStyle::Secondary)
    }

    pub fn with_message(mut self, message: impl Into<String>) -> Label {
        self.message = message.into();
        self
    }
}

/// Represents a diagnostic message and associated child messages.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "memory_usage", derive(heapsize_derive::HeapSizeOf))]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub struct Diagnostic {
    /// The overall severity of the diagnostic
    pub severity: Severity,
    /// An optional code that identifies this diagnostic.
    pub code: Option<String>,
    /// The main message associated with this diagnostic
    pub message: String,
    /// The labelled spans marking the regions of code that cause this
    /// diagnostic to be raised
    pub labels: Vec<Label>,
}

impl Diagnostic {
    pub fn new(severity: Severity, message: impl Into<String>) -> Diagnostic {
        Diagnostic {
            severity,
            code: None,
            message: message.into(),
            labels: Vec::new(),
        }
    }

    pub fn new_bug(message: impl Into<String>) -> Diagnostic {
        Diagnostic::new(Severity::Bug, message)
    }

    pub fn new_error(message: impl Into<String>) -> Diagnostic {
        Diagnostic::new(Severity::Error, message)
    }

    pub fn new_warning(message: impl Into<String>) -> Diagnostic {
        Diagnostic::new(Severity::Warning, message)
    }

    pub fn new_note(message: impl Into<String>) -> Diagnostic {
        Diagnostic::new(Severity::Note, message)
    }

    pub fn new_help(message: impl Into<String>) -> Diagnostic {
        Diagnostic::new(Severity::Help, message)
    }

    pub fn with_code(mut self, code: impl Into<String>) -> Diagnostic {
        self.code = Some(code.into());
        self
    }

    pub fn with_label(mut self, label: Label) -> Diagnostic {
        self.labels.push(label);
        self
    }

    pub fn with_labels(mut self, labels: impl IntoIterator<Item = Label>) -> Diagnostic {
        self.labels.extend(labels);
        self
    }
}
