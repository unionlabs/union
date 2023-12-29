//! A `CompiledFunctionUnwindInfo` contains the function unwind information.
//!
//! The unwind information is used to determine which function
//! called the function that threw the exception, and which
//! function called that one, and so forth.
//!
//! [Learn more](https://en.wikipedia.org/wiki/Call_stack).
use crate::lib::std::vec::Vec;
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
#[cfg(feature = "enable-serde")]
use serde::{Deserialize, Serialize};

/// Compiled function unwind information.
///
/// > Note: Windows OS have a different way of representing the [unwind info],
/// > That's why we keep the Windows data and the Unix frame layout in different
/// > fields.
///
/// [unwind info]: https://docs.microsoft.com/en-us/cpp/build/exception-handling-x64?view=vs-2019
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
#[derive(RkyvSerialize, RkyvDeserialize, Archive, Debug, Clone, PartialEq, Eq)]
#[archive_attr(derive(rkyv::CheckBytes))]
pub enum CompiledFunctionUnwindInfo {
    /// Windows UNWIND_INFO.
    WindowsX64(Vec<u8>),

    /// The unwind info is added to the Dwarf section in `Compilation`.
    Dwarf,
}

/// Generic reference to data in a `CompiledFunctionUnwindInfo`
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompiledFunctionUnwindInfoReference<'a> {
    WindowsX64(&'a [u8]),
    Dwarf,
}

/// Any struct that acts like a `CompiledFunctionUnwindInfo`.
#[allow(missing_docs)]
pub trait CompiledFunctionUnwindInfoLike<'a> {
    fn get(&'a self) -> CompiledFunctionUnwindInfoReference<'a>;
}

impl<'a> CompiledFunctionUnwindInfoLike<'a> for CompiledFunctionUnwindInfo {
    fn get(&'a self) -> CompiledFunctionUnwindInfoReference<'a> {
        match self {
            Self::WindowsX64(v) => CompiledFunctionUnwindInfoReference::WindowsX64(v.as_ref()),
            Self::Dwarf => CompiledFunctionUnwindInfoReference::Dwarf,
        }
    }
}

impl<'a> CompiledFunctionUnwindInfoLike<'a> for ArchivedCompiledFunctionUnwindInfo {
    fn get(&'a self) -> CompiledFunctionUnwindInfoReference<'a> {
        match self {
            Self::WindowsX64(v) => CompiledFunctionUnwindInfoReference::WindowsX64(v.as_ref()),
            Self::Dwarf => CompiledFunctionUnwindInfoReference::Dwarf,
        }
    }
}
