/// Semantics for a piece of error information
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ContextKind {
    /// The cause of the error
    InvalidSubcommand,
    /// The cause of the error
    InvalidArg,
    /// Existing arguments
    PriorArg,
    /// Accepted values
    ValidValue,
    /// Rejected values
    InvalidValue,
    /// Number of values present
    ActualNumValues,
    /// Number of allowed values
    ExpectedNumValues,
    /// Minimum number of allowed values
    MinValues,
    /// Number of occurrences present
    ActualNumOccurrences,
    /// Maximum number of allowed occurrences
    MaxOccurrences,
    /// Potential fix for the user
    SuggestedCommand,
    /// Potential fix for the user
    SuggestedSubcommand,
    /// Potential fix for the user
    SuggestedArg,
    /// Potential fix for the user
    SuggestedValue,
    /// Trailing argument
    TrailingArg,
    /// A usage string
    Usage,
    /// An opaque message to the user
    Custom,
}

/// A piece of error information
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ContextValue {
    /// [`ContextKind`] is self-sufficient, no additional information needed
    None,
    /// A single value
    Bool(bool),
    /// A single value
    String(String),
    /// Many values
    Strings(Vec<String>),
    /// A single value
    Number(isize),
}
