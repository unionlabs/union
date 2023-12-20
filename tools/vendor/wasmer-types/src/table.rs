use bytecheck::CheckBytes;
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
#[cfg(feature = "enable-serde")]
use serde::{Deserialize, Serialize};

/// Implementation styles for WebAssembly tables.
#[derive(
    Debug, Clone, Hash, PartialEq, Eq, RkyvSerialize, RkyvDeserialize, Archive, CheckBytes,
)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
#[archive(as = "Self")]
#[repr(u8)]
pub enum TableStyle {
    /// Signatures are stored in the table and checked in the caller.
    CallerChecksSignature,
}
