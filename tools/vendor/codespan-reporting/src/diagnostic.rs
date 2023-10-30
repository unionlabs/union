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
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
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
#[derive(Clone, Debug, PartialEq, Eq)]
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
/// The position of a Diagnostic is considered to be the position of the [`Label`] that has the earliest starting position and has the highest style which appears in all the labels of the diagnostic.
#[derive(Clone, Debug, PartialEq, Eq)]
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
    /// The order of the labels inside the vector does not have any meaning.
    /// The labels are always arranged in the order they appear in the source code.
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

    /// Set the error code of the diagnostic.
    pub fn with_code(mut self, code: impl Into<String>) -> Diagnostic<FileId> {
        self.code = Some(code.into());
        self
    }

    /// Set the message of the diagnostic.
    pub fn with_message(mut self, message: impl Into<String>) -> Diagnostic<FileId> {
        self.message = message.into();
        self
    }

    /// Add some labels to the diagnostic.
    pub fn with_labels(mut self, mut labels: Vec<Label<FileId>>) -> Diagnostic<FileId> {
        self.labels.append(&mut labels);
        self
    }

    /// Add some notes to the diagnostic.
    pub fn with_notes(mut self, mut notes: Vec<String>) -> Diagnostic<FileId> {
        self.notes.append(&mut notes);
        self
    }
}
