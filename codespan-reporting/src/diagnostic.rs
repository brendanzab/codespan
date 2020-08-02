//! Diagnostic data structures.

#[cfg(feature = "serialization")]
use serde::{Deserialize, Serialize};
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
    fn partial_cmp(&self, other: &Severity) -> Option<std::cmp::Ordering> {
        u8::partial_cmp(&self.to_cmp_int(), &other.to_cmp_int())
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd)]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub enum LabelStyle {
    /// Labels that describe the primary cause of a diagnostic.
    Primary,
    /// Labels that provide additional context for a diagnostic.
    Secondary,
}

/// A label describing an underlined region of code associated with a diagnostic.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub struct Label<FileId> {
    /// The style of the label.
    pub style: LabelStyle,
    /// The file that we are labelling.
    pub file_id: FileId,
    /// The range in bytes we are going to include in the final snippet.
    pub range: Range<usize>,
    /// An optional message to provide some additional information for the
    /// underlined code. These should not include line breaks.
    pub message: String,
}

impl<FileId> Label<FileId> {
    /// Create a new label.
    pub fn new(
        style: LabelStyle,
        file_id: FileId,
        range: impl Into<Range<usize>>,
    ) -> Label<FileId> {
        Label {
            style,
            file_id,
            range: range.into(),
            message: String::new(),
        }
    }

    /// Create a new label with a style of [`LabelStyle::Primary`].
    ///
    /// [`LabelStyle::Primary`]: LabelStyle::Primary
    pub fn primary(file_id: FileId, range: impl Into<Range<usize>>) -> Label<FileId> {
        Label::new(LabelStyle::Primary, file_id, range)
    }

    /// Create a new label with a style of [`LabelStyle::Secondary`].
    ///
    /// [`LabelStyle::Secondary`]: LabelStyle::Secondary
    pub fn secondary(file_id: FileId, range: impl Into<Range<usize>>) -> Label<FileId> {
        Label::new(LabelStyle::Secondary, file_id, range)
    }

    /// Add a message to the diagnostic.
    pub fn with_message(mut self, message: impl Into<String>) -> Label<FileId> {
        self.message = message.into();
        self
    }
}

/// Represents a diagnostic message that can provide information like errors and
/// warnings to the user.
///
/// The position of a Diagnostic is considered to be the position of the [`Label`] with a style of [`LabelStyle::primary`] that has the smallest start position.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serialization", derive(Serialize, Deserialize))]
pub struct Diagnostic<FileId> {
    /// The overall severity of the diagnostic
    pub severity: Severity,
    /// An optional code that identifies this diagnostic.
    pub code: Option<String>,
    /// The main message associated with this diagnostic.
    ///
    /// These should not include line breaks, and in order support the 'short'
    /// diagnostic display mod, the message should be specific enough to make
    /// sense on its own, without additional context provided by labels and notes.
    pub message: String,
    /// Source labels that describe the cause of the diagnostic.
    pub labels: Vec<Label<FileId>>,
    /// Notes that are associated with the primary cause of the diagnostic.
    /// These can include line breaks for improved formatting.
    pub notes: Vec<String>,
}

impl<FileId> Diagnostic<FileId> {
    /// Create a new diagnostic.
    pub fn new(severity: Severity) -> Diagnostic<FileId> {
        Diagnostic {
            severity,
            code: None,
            message: String::new(),
            labels: Vec::new(),
            notes: Vec::new(),
        }
    }

    /// Create a new diagnostic with a severity of [`Severity::Bug`].
    ///
    /// [`Severity::Bug`]: Severity::Bug
    pub fn bug() -> Diagnostic<FileId> {
        Diagnostic::new(Severity::Bug)
    }

    /// Create a new diagnostic with a severity of [`Severity::Error`].
    ///
    /// [`Severity::Error`]: Severity::Error
    pub fn error() -> Diagnostic<FileId> {
        Diagnostic::new(Severity::Error)
    }

    /// Create a new diagnostic with a severity of [`Severity::Warning`].
    ///
    /// [`Severity::Warning`]: Severity::Warning
    pub fn warning() -> Diagnostic<FileId> {
        Diagnostic::new(Severity::Warning)
    }

    /// Create a new diagnostic with a severity of [`Severity::Note`].
    ///
    /// [`Severity::Note`]: Severity::Note
    pub fn note() -> Diagnostic<FileId> {
        Diagnostic::new(Severity::Note)
    }

    /// Create a new diagnostic with a severity of [`Severity::Help`].
    ///
    /// [`Severity::Help`]: Severity::Help
    pub fn help() -> Diagnostic<FileId> {
        Diagnostic::new(Severity::Help)
    }

    /// Add an error code to the diagnostic.
    pub fn with_code(mut self, code: impl Into<String>) -> Diagnostic<FileId> {
        self.code = Some(code.into());
        self
    }

    /// Add a message to the diagnostic.
    pub fn with_message(mut self, message: impl Into<String>) -> Diagnostic<FileId> {
        self.message = message.into();
        self
    }

    /// Add some labels to the diagnostic.
    pub fn with_labels(mut self, labels: Vec<Label<FileId>>) -> Diagnostic<FileId> {
        self.labels = labels;
        self
    }

    /// Add some notes to the diagnostic.
    pub fn with_notes(mut self, notes: Vec<String>) -> Diagnostic<FileId> {
        self.notes = notes;
        self
    }
}

/// A macro that enables easy construction of a [`Diagnostic`]
///
/// Every invocation will start with one of `error`, `warn`, `warning`, `bug`, `help` or `note` and the diagnostic's message
///
/// ```rust
/// # use codespan_reporting::{diagnostic::Diagnostic, diagnostic};
/// # // This is here to give the `Diagnostic`s a type so the doc tests pass
/// # let _: Vec<Diagnostic<u32>> = vec![
/// diagnostic! {
///     error: "This is my error message!"
/// }
/// # ,
/// diagnostic! {
///     warn: "This is a scary warning"
/// }
/// # ];
/// ```
///
/// From after that, there's a few different things to choose from, each of them adding to the [`Diagnostic`] being built:
///
/// - `label`: Takes a single [`Label`], adding it to [`Diagnostic.labels`]
///
/// ```rust
/// # use codespan_reporting::{diagnostic::Label, diagnostic};
/// # let problematic_file: u32 = 0;
/// diagnostic! {
///     error: "Something's amiss",
///     label: Label::primary(problematic_file, 10..56)
/// }
/// # ;
/// ```
///
/// - `labels`: Takes multiple labels, adding all of them to [`Diagnostic.labels`]
///
/// ```rust
/// # use codespan_reporting::{diagnostic::Label, diagnostic};
/// # let problematic_file: u32 = 0;
/// diagnostic! {
///     error: "Something's amiss",
///     labels: [
///         Label::primary(problematic_file, 10..56).with_message("Maybe here?"),
///         Label::secondary(problematic_file, 643..800).with_message("Or possibly here?")
///     ]
/// }
/// # ;
/// ```
///
/// - `note`: Takes a single expression that implements [`Into`]`<`[`String`]`>`, adding it to [`Diagnostic.notes`]
///
/// ```rust
/// # use codespan_reporting::{diagnostic::Diagnostic, diagnostic};
/// # let _: Diagnostic<u32> =
/// diagnostic! {
///     error: "Something's amiss",
///     note: "It's all gone horribly wrong!"
/// }
/// # ;
/// ```
///
/// - `notes`: Takes multiple expressions that implement [`Into`]`<`[`String`]`>`, adding them all to [`Diagnostic.notes`]
///
/// ```rust
/// # use codespan_reporting::{diagnostic::Diagnostic, diagnostic};
/// # let _: Diagnostic<u32> =
/// diagnostic! {
///     error: "Something's amiss",
///     notes: [
///         "All of it's broken!",
///         "Every single bit!"
///     ]
/// }
/// # ;
/// ```
///
/// - `code`: Takes a single expression that implements [`Into`]`<`[`String`]`>` and sets the [`Diagnostic`]'s [error code] to it  
///
/// ```rust
/// # use codespan_reporting::{diagnostic::Diagnostic, diagnostic};
/// # let _: Diagnostic<u32> =
/// diagnostic! {
///     error: "Something's amiss",
///     code: "E000"
/// }
/// # ;
/// ```
///
/// Note that multiple calls to `code` will overwrite previous ones
///
/// ```rust
/// # use codespan_reporting::{diagnostic::Diagnostic, diagnostic};
/// let diagnostic = diagnostic! {
///     error: "Something's amiss",
///     code: "E000",
///     code: "Overwritten!"
/// };
///
/// assert_eq!(diagnostic.code, Some("Overwritten!".to_owned()));
/// # let _: Diagnostic<u32> = diagnostic; // This is to give `diagnostic` a type
/// ```
///
/// [`Diagnostic`]: crate::diagnostic::Diagnostic
/// [`Label`]: crate::diagnostic::Label
/// [`Diagnostic.labels`]: crate::diagnostic::Diagnostic#structfield.labels
/// [`Into`]: https://doc.rust-lang.org/std/convert/trait.Into.html
/// [`String`]: https://doc.rust-lang.org/std/string/struct.String.html
/// [`Diagnostic.notes`]: crate::diagnostic::Diagnostic#structfield.notes
/// [error code]: crate::diagnostic::Diagnostic#structfield.code
#[macro_export]
macro_rules! diagnostic {
    // Makes a `Diagnostic` with the `Error` severity
    (error: $message:expr $(, $($tt:tt)*)?) => {
        $crate::diagnostic!(
            @init [rest: $($($tt)*)?]
            [severity: $crate::diagnostic::Severity::Error]
            [message: $message]
        )
    };

    // Makes a `Diagnostic` with the `Warning` severity
    (warn: $message:expr $(, $($tt:tt)*)?) => {
        $crate::diagnostic!(
            @init [rest: $($($tt)*)?]
            [severity: $crate::diagnostic::Severity::Warning]
            [message: $message]
        )
    };

    // Makes a `Diagnostic` with the `Warning` severity
    (warning: $message:expr $(, $($tt:tt)*)?) => {
        $crate::diagnostic!(
            @init [rest: $($($tt)*)?]
            [severity: $crate::diagnostic::Severity::Warning]
            [message: $message]
        )
    };

    // Makes a `Diagnostic` with the `Bug` severity
    (bug: $message:expr $(, $($tt:tt)*)?) => {
        $crate::diagnostic!(
            @init [rest: $($($tt)*)?]
            [severity: $crate::diagnostic::Severity::Bug]
            [message: $message]
        )
    };

    // Makes a `Diagnostic` with the `Help` severity
    (help: $message:expr $(, $($tt:tt)*)?) => {
        $crate::diagnostic!(
            @init [rest: $($($tt)*)?]
            [severity: $crate::diagnostic::Severity::Help]
            [message: $message]
        )
    };

    // Makes a `Diagnostic` with the `Note` severity
    (note: $message:expr $(, $($tt:tt)*)?) => {
        $crate::diagnostic!(
            @init [rest: $($($tt)*)?]
            [severity: $crate::diagnostic::Severity::Note]
            [message: $message]
        )
    };

    // Sets up the state that's passed along through the macros
    (@init [rest: $($tt:tt)*] [severity: $severity:expr] [message: $message:expr]) => {{
        #[allow(unused_imports)]
        use ::std::{string::String, option::Option};

        $crate::diagnostic!(
            @inner
            [$($tt)*]
            [severity: $severity]
            [message: $message]
            [labels: ]
            [notes: ]
            [code: Option::<String>::None]
        )
    }};

    // Adds a label
    (
        @inner [label: $label:expr $(, $($tt:tt)*)?]
        [severity: $severity:expr]
        [message: $message:expr]
        [labels: $($labels:tt)*]
        [notes: $($notes:tt)*]
        [code: $code:expr]
    ) => {
        $crate::diagnostic!(
            @inner
            [$($($tt)*)?]
            [severity: $severity]
            [message: $message]
            [labels: $($labels)* $label,]
            [notes: $($notes)*]
            [code: $code]
        )
    };

    // Adds multiple labels
    (
        @inner [labels: [$($label:expr),* $(,)?] $(, $($tt:tt)*)?]
        [severity: $severity:expr]
        [message: $message:expr]
        [labels: $($labels:tt)*]
        [notes: $($notes:tt)*]
        [code: $code:expr]
    ) => {
        $crate::diagnostic!(
            @inner
            [$($($tt)*)?]
            [severity: $severity]
            [message: $message]
            [labels: $($labels)* $($label,)*]
            [notes: $($notes)*]
            [code: $code]
        )
    };

    // Adds a note
    (
        @inner [note: $note:expr $(, $($tt:tt)*)?]
        [severity: $severity:expr]
        [message: $message:expr]
        [labels: $($labels:tt)*]
        [notes: $($notes:tt)*]
        [code: $code:expr]
    ) => {{
        #[allow(unused_imports)]
        use ::std::convert::Into;

        $crate::diagnostic!(
            @inner
            [$($($tt)*)?]
            [severity: $severity]
            [message: $message]
            [labels: $($labels)*]
            [notes: $($notes)* Into::into($note),]
            [code: $code]
        )
    }};

    // Adds multiple notes
    (
        @inner [notes: [$($note:expr),* $(,)?] $(, $($tt:tt)*)?]
        [severity: $severity:expr]
        [message: $message:expr]
        [labels: $($labels:tt)*]
        [notes: $($notes:tt)*]
        [code: $code:expr]
    ) => {{
        #[allow(unused_imports)]
        use ::std::convert::Into;

        $crate::diagnostic!(
            @inner
            [$($($tt)*)?]
            [severity: $severity]
            [message: $message]
            [labels: $($labels)*]
            [notes: $($notes)* $(Into::into($note),)*]
            [code: $code]
        )
    }};

    // Sets the error code, overwriting the previous if set multiple times
    (
        @inner [code: $new_code:expr $(, $($tt:tt)*)?]
        [severity: $severity:expr]
        [message: $message:expr]
        [labels: $($labels:tt)*]
        [notes: $($notes:tt)*]
        [code: $code:expr]
    ) => {{
        #[allow(unused_imports)]
        use ::std::{convert::Into, string::String, option::Option};

        $crate::diagnostic!(
            @inner
            [$($($tt)*)?]
            [severity: $severity]
            [message: $message]
            [labels: $($labels)*]
            [notes: $($notes)*]
            [code: Option::<String>::Some(Into::into($new_code))]
        )
    }};

    // Finishes up the macro, creating the final `Diagnostic`
    (
        @inner []
        [severity: $severity:expr]
        [message: $message:expr]
        [labels: $($labels:tt)*]
        [notes: $($notes:tt)*]
        [code: $code:expr]
    ) => {{
        #[allow(unused_imports)]
        use ::std::convert::Into;

        $crate::diagnostic::Diagnostic {
            severity: $severity,
            message: Into::into($message),
            labels: vec![$($labels)*],
            notes: vec![$($notes)*],
            code: $code,
        }
    }};
}

#[cfg(test)]
mod tests {
    use super::{Diagnostic, Label, Severity};

    const MESSAGE: &str = "Some random message";

    #[test]
    fn messages() {
        let expected = |severity| -> Diagnostic<u32> {
            Diagnostic {
                severity,
                message: MESSAGE.into(),
                labels: vec![],
                notes: vec![],
                code: None,
            }
        };

        let diagnostic: Diagnostic<u32> = diagnostic! {
            error: MESSAGE,
        };
        assert_eq!(diagnostic, expected(Severity::Error));

        let diagnostic: Diagnostic<u32> = diagnostic! {
            error: MESSAGE
        };
        assert_eq!(diagnostic, expected(Severity::Error));

        let diagnostic: Diagnostic<u32> = diagnostic! {
            warn: MESSAGE,
        };
        assert_eq!(diagnostic, expected(Severity::Warning));

        let diagnostic: Diagnostic<u32> = diagnostic! {
            warn: MESSAGE
        };
        assert_eq!(diagnostic, expected(Severity::Warning));

        let diagnostic: Diagnostic<u32> = diagnostic! {
            bug: MESSAGE,
        };
        assert_eq!(diagnostic, expected(Severity::Bug));

        let diagnostic: Diagnostic<u32> = diagnostic! {
            bug: MESSAGE
        };
        assert_eq!(diagnostic, expected(Severity::Bug));

        let diagnostic: Diagnostic<u32> = diagnostic! {
            help: MESSAGE,
        };
        assert_eq!(diagnostic, expected(Severity::Help));

        let diagnostic: Diagnostic<u32> = diagnostic! {
            help: MESSAGE
        };
        assert_eq!(diagnostic, expected(Severity::Help));

        let diagnostic: Diagnostic<u32> = diagnostic! {
            note: MESSAGE,
        };
        assert_eq!(diagnostic, expected(Severity::Note));

        let diagnostic: Diagnostic<u32> = diagnostic! {
            note: MESSAGE
        };
        assert_eq!(diagnostic, expected(Severity::Note));
    }

    #[test]
    fn notes() {
        let diagnostic: Diagnostic<u32> = diagnostic! {
            error: MESSAGE,
            note: "You can try to like, not suck",
        };

        let expected = Diagnostic {
            severity: Severity::Error,
            message: MESSAGE.into(),
            labels: vec![],
            notes: vec!["You can try to like, not suck".into()],
            code: None,
        };

        assert_eq!(diagnostic, expected);

        let diagnostic: Diagnostic<u32> = diagnostic! {
            error: MESSAGE,
            note: "You can try to like, not suck"
        };

        let expected = Diagnostic {
            severity: Severity::Error,
            message: MESSAGE.into(),
            labels: vec![],
            notes: vec!["You can try to like, not suck".into()],
            code: None,
        };

        assert_eq!(diagnostic, expected);

        let diagnostic: Diagnostic<u32> = diagnostic! {
            error: MESSAGE,
            note: "You can try to like, not suck",
            note: "But it's alright, we all start somewhere",
        };

        let expected = Diagnostic {
            severity: Severity::Error,
            message: MESSAGE.into(),
            labels: vec![],
            notes: vec![
                "You can try to like, not suck".into(),
                "But it's alright, we all start somewhere".into(),
            ],
            code: None,
        };

        assert_eq!(diagnostic, expected);

        let diagnostic: Diagnostic<u32> = diagnostic! {
            error: MESSAGE,
            notes: [
                "You can try to like, not suck",
                "But it's alright, we all start somewhere",
            ],
            note: "I can no longer think of test notes",
        };

        let expected = Diagnostic {
            severity: Severity::Error,
            message: MESSAGE.into(),
            labels: vec![],
            notes: vec![
                "You can try to like, not suck".into(),
                "But it's alright, we all start somewhere".into(),
                "I can no longer think of test notes".into(),
            ],
            code: None,
        };

        assert_eq!(diagnostic, expected);

        let diagnostic: Diagnostic<u32> = diagnostic! {
            error: MESSAGE,
            notes: [],
        };

        let expected = Diagnostic {
            severity: Severity::Error,
            message: MESSAGE.into(),
            labels: vec![],
            notes: vec![],
            code: None,
        };

        assert_eq!(diagnostic, expected);
    }

    #[test]
    fn code() {
        let diagnostic: Diagnostic<u32> = diagnostic! {
            error: MESSAGE,
            code: "",
        };

        let expected = Diagnostic {
            severity: Severity::Error,
            message: MESSAGE.into(),
            labels: vec![],
            notes: vec![],
            code: Some("".into()),
        };

        assert_eq!(diagnostic, expected);

        let diagnostic: Diagnostic<u32> = diagnostic! {
            error: MESSAGE,
            code: "E000",
        };

        let expected = Diagnostic {
            severity: Severity::Error,
            message: MESSAGE.into(),
            labels: vec![],
            notes: vec![],
            code: Some("E000".into()),
        };

        assert_eq!(diagnostic, expected);

        let diagnostic: Diagnostic<u32> = diagnostic! {
            error: MESSAGE,
            code: "E000"
        };

        let expected = Diagnostic {
            severity: Severity::Error,
            message: MESSAGE.into(),
            labels: vec![],
            notes: vec![],
            code: Some("E000".into()),
        };

        assert_eq!(diagnostic, expected);
    }

    #[test]
    fn labels() {
        let diagnostic: Diagnostic<u32> = diagnostic! {
            error: MESSAGE,
            label: Label::primary(0, 0..0),
        };

        let expected = Diagnostic {
            severity: Severity::Error,
            message: MESSAGE.into(),
            labels: vec![Label::primary(0, 0..0)],
            notes: vec![],
            code: None,
        };

        assert_eq!(diagnostic, expected);

        let diagnostic: Diagnostic<u32> = diagnostic! {
            error: MESSAGE,
            label: Label::primary(0, 0..0)
        };

        let expected = Diagnostic {
            severity: Severity::Error,
            message: MESSAGE.into(),
            labels: vec![Label::primary(0, 0..0)],
            notes: vec![],
            code: None,
        };

        assert_eq!(diagnostic, expected);

        let diagnostic: Diagnostic<u32> = diagnostic! {
            error: MESSAGE,
            label: Label::primary(0, 0..0),
            label: Label::secondary(0, 0..0),
        };

        let expected = Diagnostic {
            severity: Severity::Error,
            message: MESSAGE.into(),
            labels: vec![Label::primary(0, 0..0), Label::secondary(0, 0..0)],
            notes: vec![],
            code: None,
        };

        assert_eq!(diagnostic, expected);

        let diagnostic: Diagnostic<u32> = diagnostic! {
            error: MESSAGE,
            labels: [
                Label::primary(0, 0..0),
                Label::secondary(0, 0..0),
            ],
        };

        let expected = Diagnostic {
            severity: Severity::Error,
            message: MESSAGE.into(),
            labels: vec![Label::primary(0, 0..0), Label::secondary(0, 0..0)],
            notes: vec![],
            code: None,
        };

        assert_eq!(diagnostic, expected);

        let diagnostic: Diagnostic<u32> = diagnostic! {
            error: MESSAGE,
            labels: [
                Label::primary(0, 0..0),
                Label::secondary(0, 0..0),
            ]
        };

        let expected = Diagnostic {
            severity: Severity::Error,
            message: MESSAGE.into(),
            labels: vec![Label::primary(0, 0..0), Label::secondary(0, 0..0)],
            notes: vec![],
            code: None,
        };

        assert_eq!(diagnostic, expected);

        let diagnostic: Diagnostic<u32> = diagnostic! {
            error: MESSAGE,
            labels: [],
        };

        let expected = Diagnostic {
            severity: Severity::Error,
            message: MESSAGE.into(),
            labels: vec![],
            notes: vec![],
            code: None,
        };

        assert_eq!(diagnostic, expected);
    }

    #[test]
    fn complex_use() {
        let diagnostic: Diagnostic<u32> = diagnostic! {
            error: "This is an error message",
            labels: [Label::primary(0, 100..200), Label::secondary(10, 0..1)],
            code: "E100",
            label: Label::secondary(50, 1..2),
            note: "One. Singular. Note.",
            notes: [],
            notes: ["Another one", "And another one"]
        };

        let expected = Diagnostic {
            severity: Severity::Error,
            message: "This is an error message".into(),
            labels: vec![
                Label::primary(0, 100..200),
                Label::secondary(10, 0..1),
                Label::secondary(50, 1..2),
            ],
            notes: vec![
                "One. Singular. Note.".into(),
                "Another one".into(),
                "And another one".into(),
            ],
            code: Some("E100".into()),
        };

        assert_eq!(diagnostic, expected);
    }
}
