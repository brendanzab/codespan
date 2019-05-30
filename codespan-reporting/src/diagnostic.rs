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
    pub fn new(span: ByteSpan, message: impl Into<String>) -> Label {
        Label {
            span,
            message: message.into(),
        }
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
    /// The main message associated with this diagnostic.
    pub message: String,
    /// A label that marks the primary cause of this diagnostic.
    pub primary_label: Label,
    /// Secondary labels that provide additional context for the diagnostic.
    pub secondary_labels: Vec<Label>,
}

impl Diagnostic {
    pub fn new(severity: Severity, message: impl Into<String>, primary_label: Label) -> Diagnostic {
        Diagnostic {
            severity,
            code: None,
            message: message.into(),
            primary_label,
            secondary_labels: Vec::new(),
        }
    }

    pub fn new_bug(message: impl Into<String>, primary_label: Label) -> Diagnostic {
        Diagnostic::new(Severity::Bug, message, primary_label)
    }

    pub fn new_error(message: impl Into<String>, primary_label: Label) -> Diagnostic {
        Diagnostic::new(Severity::Error, message, primary_label)
    }

    pub fn new_warning(message: impl Into<String>, primary_label: Label) -> Diagnostic {
        Diagnostic::new(Severity::Warning, message, primary_label)
    }

    pub fn new_note(message: impl Into<String>, primary_label: Label) -> Diagnostic {
        Diagnostic::new(Severity::Note, message, primary_label)
    }

    pub fn new_help(message: impl Into<String>, primary_label: Label) -> Diagnostic {
        Diagnostic::new(Severity::Help, message, primary_label)
    }

    pub fn with_code(mut self, code: impl Into<String>) -> Diagnostic {
        self.code = Some(code.into());
        self
    }

    pub fn with_secondary_labels(mut self, labels: impl IntoIterator<Item = Label>) -> Diagnostic {
        self.secondary_labels.extend(labels);
        self
    }
}
