//! Diagnostic reporting support for the codespan crate

use codespan::ByteSpan;
#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};

use crate::Severity;

/// A label describing an underlined region of code associated with a diagnostic.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "memory_usage", derive(heapsize_derive::HeapSizeOf))]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub struct Label {
    /// The span we are going to include in the final snippet.
    pub span: ByteSpan,
    /// A message to provide some additional information for the underlined code.
    pub message: String,
}

impl Label {
    /// Create a new label.
    pub fn new(span: ByteSpan, message: impl Into<String>) -> Label {
        Label {
            span,
            message: message.into(),
        }
    }
}

/// Represents a diagnostic message that can provide information like errors and
/// warnings to the user.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "memory_usage", derive(heapsize_derive::HeapSizeOf))]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub struct Diagnostic {
    /// The overall severity of the diagnostic
    pub severity: Severity,
    /// An optional code that identifies this diagnostic.
    pub code: Option<String>,
    /// The main message associated with this diagnostic.
    pub message: String,
    /// A label that describes the primary cause of this diagnostic.
    pub primary_label: Label,
    /// Secondary labels that provide additional context for the diagnostic.
    pub secondary_labels: Vec<Label>,
}

impl Diagnostic {
    /// Create a new diagnostic.
    pub fn new(severity: Severity, message: impl Into<String>, primary_label: Label) -> Diagnostic {
        Diagnostic {
            severity,
            code: None,
            message: message.into(),
            primary_label,
            secondary_labels: Vec::new(),
        }
    }

    /// Create a new diagnostic with a severity of `Severity::Bug`.
    pub fn new_bug(message: impl Into<String>, primary_label: Label) -> Diagnostic {
        Diagnostic::new(Severity::Bug, message, primary_label)
    }

    /// Create a new diagnostic with a severity of `Severity::Error`.
    pub fn new_error(message: impl Into<String>, primary_label: Label) -> Diagnostic {
        Diagnostic::new(Severity::Error, message, primary_label)
    }

    /// Create a new diagnostic with a severity of `Severity::Warning`.
    pub fn new_warning(message: impl Into<String>, primary_label: Label) -> Diagnostic {
        Diagnostic::new(Severity::Warning, message, primary_label)
    }

    /// Create a new diagnostic with a severity of `Severity::Note`.
    pub fn new_note(message: impl Into<String>, primary_label: Label) -> Diagnostic {
        Diagnostic::new(Severity::Note, message, primary_label)
    }

    /// Create a new diagnostic with a severity of `Severity::Help`.
    pub fn new_help(message: impl Into<String>, primary_label: Label) -> Diagnostic {
        Diagnostic::new(Severity::Help, message, primary_label)
    }

    /// Add an error code to the diagnostic.
    pub fn with_code(mut self, code: impl Into<String>) -> Diagnostic {
        self.code = Some(code.into());
        self
    }

    /// Add some secondary labels to the diagnostic.
    pub fn with_secondary_labels(mut self, labels: impl IntoIterator<Item = Label>) -> Diagnostic {
        self.secondary_labels.extend(labels);
        self
    }
}
