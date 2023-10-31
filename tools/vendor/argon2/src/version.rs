//! Version of the algorithm.

use crate::{Error, Result};

/// Version of the algorithm.
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Version {
    /// Version 16 (0x10 in hex)
    ///
    /// Performs overwrite internally
    V0x10 = 0x10,

    /// Version 19 (0x13 in hex, default)
    ///
    /// Performs XOR internally
    V0x13 = 0x13,
}

impl Version {
    /// Serialize version as little endian bytes
    pub(crate) fn to_le_bytes(self) -> [u8; 4] {
        (self as u32).to_le_bytes()
    }
}

impl Default for Version {
    fn default() -> Self {
        Self::V0x13
    }
}

impl From<Version> for u32 {
    fn from(version: Version) -> u32 {
        version as u32
    }
}

impl TryFrom<u32> for Version {
    type Error = Error;

    fn try_from(version_id: u32) -> Result<Version> {
        match version_id {
            0x10 => Ok(Version::V0x10),
            0x13 => Ok(Version::V0x13),
            _ => Err(Error::VersionInvalid),
        }
    }
}
