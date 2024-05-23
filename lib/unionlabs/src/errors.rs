use core::fmt::Debug;

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("unknown enum variant `{0}`")]
pub struct UnknownEnumVariant<T>(pub T);

/// A protobuf field was none unexpectedly.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("missing field `{0}`")]
pub struct MissingField(pub &'static str);

/// For fields that are "fake options" from prost, for use in `TryFrom<<Self as Proto>::Proto>`.
///
/// `Self::Error` is expected to have a `MissingField(`[`MissingField`]`)` variant.
macro_rules! required {
    ($struct_var:ident.$field:ident) => {
        $struct_var
            .$field
            .ok_or(<Self::Error>::MissingField(MissingField(stringify!(
                $field
            ))))
    };
}

// https://stackoverflow.com/questions/26731243/how-do-i-use-a-macro-across-module-files
pub(crate) use required;

// Expected one length, but found another.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("invalid length: expected {expected}, found {found}")]
pub struct InvalidLength {
    // TODO: Make this generic with this enum as individual types
    pub expected: ExpectedLength,
    pub found: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, derive_more::Display)]
pub enum ExpectedLength {
    #[display(fmt = "exactly {_0}")]
    Exact(usize),
    #[display(fmt = "less than {_0}")]
    LessThan(usize),
    #[display(fmt = "between ({_0}, {_1})")]
    Between(usize, usize),
    #[display(fmt = "greater than or equal to ({_0})")]
    Gte(usize),
    #[display(fmt = "either {_0} or {_1}")]
    Either(usize, usize),
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("invalid value: expected {expected}, found {found}")]
pub struct InvalidValue<T> {
    pub expected: T,
    pub found: T,
}
