//! Diagnostic reporting support for the codespan crate

use codespan::FileSpan;
#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// A severity level for diagnostic messages.
///
/// These are ordered in the following way:
///
/// ```rust
/// use codespan_reporting::Severity;
///
/// assert!(Severity::Bug > Severity::Error);
/// assert!(Severity::Error > Severity::Warning);
/// assert!(Severity::Warning > Severity::Note);
/// assert!(Severity::Note > Severity::Help);
/// ```
#[derive(Copy, Clone, PartialEq, Hash, Debug)]
#[cfg_attr(feature = "memory_usage", derive(heapsize_derive::HeapSizeOf))]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub enum Severity {
    /// An unexpected bug.
    Bug,
    /// An error.
    Error,
    /// A warning.
    Warning,
    /// A note.
    Note,
    /// A help message.
    Help,
}

impl Severity {
    /// We want bugs to be the maximum severity, errors next, etc...
    fn to_cmp_int(self) -> u8 {
        match self {
            Severity::Bug => 5,
            Severity::Error => 4,
            Severity::Warning => 3,
            Severity::Note => 2,
            Severity::Help => 1,
        }
    }
}

impl PartialOrd for Severity {
    fn partial_cmp(&self, other: &Severity) -> Option<Ordering> {
        u8::partial_cmp(&self.to_cmp_int(), &other.to_cmp_int())
    }
}

/// A label describing an underlined region of code associated with a diagnostic.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "memory_usage", derive(heapsize_derive::HeapSizeOf))]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub struct Label {
    /// The span we are going to include in the final snippet.
    pub file_span: FileSpan,
    /// A message to provide some additional information for the underlined code.
    pub message: String,
}

impl Label {
    /// Create a new label.
    pub fn new(file_span: FileSpan, message: impl Into<String>) -> Label {
        Label {
            file_span,
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
