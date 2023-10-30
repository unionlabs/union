//! A `CompiledFunctionUnwindInfo` contains the function unwind information.
//!
//! The unwind information is used to determine which function
//! called the function that threw the exception, and which
//! function called that one, and so forth.
//!
//! [Learn more](https://en.wikipedia.org/wiki/Call_stack).
use crate::lib::std::vec::Vec;
use loupe::MemoryUsage;
#[cfg(feature = "enable-rkyv")]
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
#[cfg_attr(
    feature = "enable-rkyv",
    derive(RkyvSerialize, RkyvDeserialize, Archive)
)]
#[derive(Debug, Clone, PartialEq, Eq, MemoryUsage)]
pub enum CompiledFunctionUnwindInfo {
    /// Windows UNWIND_INFO.
    WindowsX64(Vec<u8>),

    /// The unwind info is added to the Dwarf section in `Compilation`.
    Dwarf,
}
