// This file contains code from external sources.
// Attributions: https://github.com/wasmerio/wasmer/blob/master/ATTRIBUTIONS.md

//! Source locations.
//!
//! A [`SourceLoc`] determines the position of a certain instruction
//! relative to the WebAssembly module. This is used mainly for debugging
//! and tracing errors.

use crate::lib::std::fmt;
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
#[cfg(feature = "enable-serde")]
use serde::{Deserialize, Serialize};

/// A source location.
///
/// The default source location uses the all-ones bit pattern `!0`. It is used for instructions
/// that can't be given a real source location.
#[cfg_attr(
    feature = "enable-serde",
    derive(Serialize, Deserialize),
    serde(transparent)
)]
#[derive(RkyvSerialize, RkyvDeserialize, Archive)]
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, rkyv::CheckBytes)]
#[archive(as = "Self")]
pub struct SourceLoc(u32);

impl SourceLoc {
    /// Create a new source location with the given bits.
    pub fn new(bits: u32) -> Self {
        Self(bits)
    }

    /// Is this the default source location?
    pub fn is_default(self) -> bool {
        self == Default::default()
    }

    /// Read the bits of this source location.
    pub fn bits(self) -> u32 {
        self.0
    }
}

impl Default for SourceLoc {
    fn default() -> Self {
        Self(!0)
    }
}

impl fmt::Display for SourceLoc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_default() {
            write!(f, "0x-")
        } else {
            write!(f, "0x{:04x}", self.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SourceLoc;
    use crate::lib::std::string::ToString;

    #[test]
    fn display() {
        assert_eq!(SourceLoc::default().to_string(), "0x-");
        assert_eq!(SourceLoc::new(0).to_string(), "0x0000");
        assert_eq!(SourceLoc::new(16).to_string(), "0x0010");
        assert_eq!(SourceLoc::new(0xabcdef).to_string(), "0xabcdef");
    }
}
