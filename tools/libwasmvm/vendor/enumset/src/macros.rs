/// Everything in this module is internal API and may change at any time.
#[doc(hidden)]
pub mod __internal {
    /// A reexport of core to allow our macros to be generic to std vs core.
    pub use ::core as core_export;

    /// A reexport of serde so our users don't have to also have a serde dependency.
    #[cfg(feature = "serde")]
    pub use serde2 as serde;

    /// Reexports of internal types
    pub use crate::{
        repr::{ArrayRepr, EnumSetTypeRepr},
        traits::EnumSetTypePrivate,
    };
}

/// Creates a EnumSet literal, which can be used in const contexts.
///
/// The syntax used is `enum_set!(Type::A | Type::B | Type::C)`. Each variant must be of the same
/// type, or a error will occur at compile-time.
///
/// This macro accepts trailing `|`s to allow easier use in other macros.
///
/// # Examples
///
/// ```rust
/// # use enumset::*;
/// # #[derive(EnumSetType, Debug)] enum Enum { A, B, C }
/// const CONST_SET: EnumSet<Enum> = enum_set!(Enum::A | Enum::B);
/// assert_eq!(CONST_SET, Enum::A | Enum::B);
/// ```
///
/// This macro is strongly typed. For example, the following will not compile:
///
/// ```compile_fail
/// # use enumset::*;
/// # #[derive(EnumSetType, Debug)] enum Enum { A, B, C }
/// # #[derive(EnumSetType, Debug)] enum Enum2 { A, B, C }
/// let type_error = enum_set!(Enum::A | Enum2::B);
/// ```
#[macro_export]
macro_rules! enum_set {
    ($(|)*) => {
        EnumSet::EMPTY
    };
    ($value:path $(|)*) => {
        {
            #[allow(deprecated)] let value = $value.__impl_enumset_internal__const_only();
            value
        }
    };
    ($value:path | $($rest:path)|* $(|)*) => {
        {
            #[allow(deprecated)] let value = $value.__impl_enumset_internal__const_only();
            $(#[allow(deprecated)] let value = $rest.__impl_enumset_internal__const_merge(value);)*
            value
        }
    };
}
