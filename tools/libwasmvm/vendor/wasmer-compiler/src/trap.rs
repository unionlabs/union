use crate::CodeOffset;
use loupe::MemoryUsage;
#[cfg(feature = "enable-rkyv")]
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
#[cfg(feature = "enable-serde")]
use serde::{Deserialize, Serialize};
use wasmer_types::TrapCode;

/// Information about trap.
#[cfg_attr(feature = "enable-serde", derive(Deserialize, Serialize))]
#[cfg_attr(
    feature = "enable-rkyv",
    derive(RkyvSerialize, RkyvDeserialize, Archive)
)]
#[derive(Clone, Debug, PartialEq, Eq, MemoryUsage)]
pub struct TrapInformation {
    /// The offset of the trapping instruction in native code. It is relative to the beginning of the function.
    pub code_offset: CodeOffset,
    /// Code of the trap.
    pub trap_code: TrapCode,
}
