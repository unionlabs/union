use loupe::MemoryUsage;
#[cfg(feature = "enable-rkyv")]
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
#[cfg(feature = "enable-serde")]
use serde::{Deserialize, Serialize};

/// Implementation styles for WebAssembly tables.
#[derive(Debug, Clone, Hash, PartialEq, Eq, MemoryUsage)]
#[cfg_attr(
    feature = "enable-rkyv",
    derive(RkyvSerialize, RkyvDeserialize, Archive)
)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub enum TableStyle {
    /// Signatures are stored in the table and checked in the caller.
    CallerChecksSignature,
}
