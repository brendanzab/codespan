//! Diagnostic data structures.

#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::ops::Range;

/// A severity level for diagnostic messages.
///
/// These are ordered in the following way:
///
/// ```rust
/// use codespan_reporting::diagnostic::Severity;
///
/// assert!(Severity::Bug > Severity::Error);
/// assert!(Severity::Error > Severity::Warning);
/// assert!(Severity::Warning > Severity::Note);
/// assert!(Severity::Note > Severity::Help);
/// ```
#[derive(Copy, Clone, PartialEq, Hash, Debug)]
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
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub struct Label<FileId> {
    /// The file that we are labelling.
    pub file_id: FileId,
    /// The range we are going to include in the final snippet.
    pub range: Range<usize>,
    /// A message to provide some additional information for the underlined
    /// code. These should not include line breaks.
    pub message: String,
}

impl<FileId> Label<FileId> {
    /// Create a new label.
    pub fn new(
        file_id: FileId,
        range: impl Into<Range<usize>>,
        message: impl Into<String>,
    ) -> Label<FileId> {
        Label {
            file_id,
            range: range.into(),
            message: message.into(),
        }
    }
}

/// Represents a diagnostic message that can provide information like errors and
/// warnings to the user.
#[derive(Clone)]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub struct Diagnostic<FileId> {
    /// The overall severity of the diagnostic
    pub severity: Severity,
    /// An optional code that identifies this diagnostic.
    pub code: Option<String>,
    /// The main message associated with this diagnostic.
    ///
    /// These should not include line breaks, and the message should be specific
    /// enough to make sense when paired only with the location given by the
    /// `primary_label`.
    pub message: String,
    /// A label that describes the primary cause of this diagnostic.
    pub primary_label: Label<FileId>,
    /// Notes that are associated with the primary cause of the diagnostic.
    /// These can include line breaks for improved formatting.
    pub notes: Vec<String>,
    /// Secondary labels that provide additional context for the diagnostic.
    pub secondary_labels: Vec<Label<FileId>>,
}

impl<FileId> Diagnostic<FileId> {
    /// Create a new diagnostic.
    pub fn new(
        severity: Severity,
        message: impl Into<String>,
        primary_label: Label<FileId>,
    ) -> Diagnostic<FileId> {
        Diagnostic {
            severity,
            code: None,
            message: message.into(),
            primary_label,
            notes: Vec::new(),
            secondary_labels: Vec::new(),
        }
    }

    /// Create a new diagnostic with a severity of `Severity::Bug`.
    pub fn new_bug(message: impl Into<String>, primary_label: Label<FileId>) -> Diagnostic<FileId> {
        Diagnostic::new(Severity::Bug, message, primary_label)
    }

    /// Create a new diagnostic with a severity of `Severity::Error`.
    pub fn new_error(
        message: impl Into<String>,
        primary_label: Label<FileId>,
    ) -> Diagnostic<FileId> {
        Diagnostic::new(Severity::Error, message, primary_label)
    }

    /// Create a new diagnostic with a severity of `Severity::Warning`.
    pub fn new_warning(
        message: impl Into<String>,
        primary_label: Label<FileId>,
    ) -> Diagnostic<FileId> {
        Diagnostic::new(Severity::Warning, message, primary_label)
    }

    /// Create a new diagnostic with a severity of `Severity::Note`.
    pub fn new_note(
        message: impl Into<String>,
        primary_label: Label<FileId>,
    ) -> Diagnostic<FileId> {
        Diagnostic::new(Severity::Note, message, primary_label)
    }

    /// Create a new diagnostic with a severity of `Severity::Help`.
    pub fn new_help(
        message: impl Into<String>,
        primary_label: Label<FileId>,
    ) -> Diagnostic<FileId> {
        Diagnostic::new(Severity::Help, message, primary_label)
    }

    /// Add an error code to the diagnostic.
    pub fn with_code(mut self, code: impl Into<String>) -> Diagnostic<FileId> {
        self.code = Some(code.into());
        self
    }

    /// Add some notes to the diagnostic.
    pub fn with_notes(mut self, notes: Vec<String>) -> Diagnostic<FileId> {
        self.notes = notes;
        self
    }

    /// Add some secondary labels to the diagnostic.
    pub fn with_secondary_labels(
        mut self,
        labels: impl IntoIterator<Item = Label<FileId>>,
    ) -> Diagnostic<FileId> {
        self.secondary_labels.extend(labels);
        self
    }
}

/// A location in a source file.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Location {
    /// The line index in the source file.
    pub line: usize,
    /// The column index in the source file.
    pub column: usize,
}

/// A line within a source file.
pub struct Line<Source> {
    /// The starting byte index of the line.
    pub start: usize,
    /// The source of the line.
    pub source: Source,
}

impl<Source> Line<Source>
where
    Source: AsRef<str>,
{
    pub fn column_index(&self, byte_index: usize) -> Option<usize> {
        match byte_index.checked_sub(self.start) {
            None => Some(0),
            Some(relative_index) => {
                let source = self.source.as_ref().get(..relative_index)?;
                Some(source.chars().count())
            },
        }
    }
}

/// Files that can be used for pretty printing.
pub trait Files {
    type FileId: Copy + PartialEq + PartialOrd + Eq + Ord + std::hash::Hash;
    type Origin: std::fmt::Display;
    type LineSource: AsRef<str>;

    /// The origin of a file.
    fn origin(&self, id: Self::FileId) -> Option<Self::Origin>;

    /// The line at the given index.
    fn line(&self, id: Self::FileId, line_index: usize) -> Option<Line<Self::LineSource>>;

    /// The index of the line at the given byte index.
    fn line_index(&self, id: Self::FileId, byte_index: usize) -> Option<usize>;

    /// The location of the given byte index.
    fn location(&self, id: Self::FileId, byte_index: usize) -> Option<Location> {
        let line_index = self.line_index(id, byte_index)?;
        let column_index = self.line(id, line_index)?.column_index(byte_index)?;

        Some(Location {
            line: line_index,
            column: column_index,
        })
    }
}

impl<Source> Files for codespan::Files<Source>
where
    Source: AsRef<str>,
{
    type FileId = codespan::FileId;
    type Origin = String;
    type LineSource = String;

    fn origin(&self, id: codespan::FileId) -> Option<String> {
        use std::path::PathBuf;

        Some(PathBuf::from(self.name(id)).display().to_string())
    }

    fn line_index(&self, id: Self::FileId, byte_index: usize) -> Option<usize> {
        Some(self.line_index(id, byte_index as u32).to_usize())
    }

    fn line(&self, id: codespan::FileId, line_index: usize) -> Option<Line<String>> {
        let span = self.line_span(id, line_index as u32).ok()?;
        let source = self.source_slice(id, span).ok()?;

        Some(Line {
            start: span.start().to_usize(),
            source: source.to_owned(),
        })
    }
}
