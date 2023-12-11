use crate::lib::std::convert::TryFrom;
use crate::lib::std::fmt;
use crate::lib::std::ops::{Add, Sub};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
#[cfg(feature = "enable-serde")]
use serde::{Deserialize, Serialize};
use std::convert::TryInto;
use thiserror::Error;

/// WebAssembly page sizes are fixed to be 64KiB.
/// Note: large page support may be added in an opt-in manner in the [future].
///
/// [future]: https://webassembly.org/docs/future-features/#large-page-support
pub const WASM_PAGE_SIZE: usize = 0x10000;

/// The number of pages we can have before we run out of byte index space.
pub const WASM_MAX_PAGES: u32 = 0x10000;

/// The minimum number of pages allowed.
pub const WASM_MIN_PAGES: u32 = 0x100;

/// Units of WebAssembly pages (as specified to be 65,536 bytes).
#[derive(
    Copy,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    RkyvSerialize,
    RkyvDeserialize,
    Archive,
    rkyv::CheckBytes,
)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
#[archive(as = "Self")]
pub struct Pages(pub u32);

impl Pages {
    /// Returns the largest value that can be represented by the Pages type.
    ///
    /// This is defined by the WebAssembly standard as 65,536 pages.
    #[inline(always)]
    pub const fn max_value() -> Self {
        Self(WASM_MAX_PAGES)
    }

    /// Checked addition. Computes `self + rhs`,
    /// returning `None` if overflow occurred.
    pub fn checked_add(self, rhs: Self) -> Option<Self> {
        let added = (self.0 as usize) + (rhs.0 as usize);
        if added <= (WASM_MAX_PAGES as usize) {
            Some(Self(added as u32))
        } else {
            None
        }
    }

    /// Calculate number of bytes from pages.
    pub fn bytes(self) -> Bytes {
        self.into()
    }
}

impl fmt::Debug for Pages {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} pages", self.0)
    }
}

impl From<u32> for Pages {
    fn from(other: u32) -> Self {
        Self(other)
    }
}

/// Units of WebAssembly memory in terms of 8-bit bytes.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub struct Bytes(pub usize);

impl fmt::Debug for Bytes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} bytes", self.0)
    }
}

impl From<Pages> for Bytes {
    fn from(pages: Pages) -> Self {
        Self((pages.0 as usize) * WASM_PAGE_SIZE)
    }
}

impl From<usize> for Bytes {
    fn from(other: usize) -> Self {
        Self(other)
    }
}

impl From<u32> for Bytes {
    fn from(other: u32) -> Self {
        Self(other.try_into().unwrap())
    }
}

impl<T> Sub<T> for Pages
where
    T: Into<Self>,
{
    type Output = Self;
    fn sub(self, rhs: T) -> Self {
        Self(self.0 - rhs.into().0)
    }
}

impl<T> Add<T> for Pages
where
    T: Into<Self>,
{
    type Output = Self;
    fn add(self, rhs: T) -> Self {
        Self(self.0 + rhs.into().0)
    }
}

/// The only error that can happen when converting `Bytes` to `Pages`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("Number of pages exceeds uint32 range")]
pub struct PageCountOutOfRange;

impl TryFrom<Bytes> for Pages {
    type Error = PageCountOutOfRange;

    fn try_from(bytes: Bytes) -> Result<Self, Self::Error> {
        let pages: u32 = (bytes.0 / WASM_PAGE_SIZE)
            .try_into()
            .or(Err(PageCountOutOfRange))?;
        Ok(Self(pages))
    }
}

impl<T> Sub<T> for Bytes
where
    T: Into<Self>,
{
    type Output = Self;
    fn sub(self, rhs: T) -> Self {
        Self(self.0 - rhs.into().0)
    }
}

impl<T> Add<T> for Bytes
where
    T: Into<Self>,
{
    type Output = Self;
    fn add(self, rhs: T) -> Self {
        Self(self.0 + rhs.into().0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_bytes_to_pages() {
        // rounds down
        let pages = Pages::try_from(Bytes(0)).unwrap();
        assert_eq!(pages, Pages(0));
        let pages = Pages::try_from(Bytes(1)).unwrap();
        assert_eq!(pages, Pages(0));
        let pages = Pages::try_from(Bytes(WASM_PAGE_SIZE - 1)).unwrap();
        assert_eq!(pages, Pages(0));
        let pages = Pages::try_from(Bytes(WASM_PAGE_SIZE)).unwrap();
        assert_eq!(pages, Pages(1));
        let pages = Pages::try_from(Bytes(WASM_PAGE_SIZE + 1)).unwrap();
        assert_eq!(pages, Pages(1));
        let pages = Pages::try_from(Bytes(28 * WASM_PAGE_SIZE + 42)).unwrap();
        assert_eq!(pages, Pages(28));
        let pages = Pages::try_from(Bytes((u32::MAX as usize) * WASM_PAGE_SIZE)).unwrap();
        assert_eq!(pages, Pages(u32::MAX));
        let pages = Pages::try_from(Bytes((u32::MAX as usize) * WASM_PAGE_SIZE + 1)).unwrap();
        assert_eq!(pages, Pages(u32::MAX));

        // Errors when page count cannot be represented as u32
        let result = Pages::try_from(Bytes((u32::MAX as usize + 1) * WASM_PAGE_SIZE));
        assert_eq!(result.unwrap_err(), PageCountOutOfRange);
        let result = Pages::try_from(Bytes(usize::MAX));
        assert_eq!(result.unwrap_err(), PageCountOutOfRange);
    }
}
