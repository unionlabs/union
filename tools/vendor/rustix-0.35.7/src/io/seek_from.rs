//! The following is derived from Rust's
//! library/std/src/io/mod.rs at revision
//! dca3f1b786efd27be3b325ed1e01e247aa589c3b.

/// Enumeration of possible methods to seek within an I/O object.
///
/// It is used by the [`Seek`] trait.
#[derive(Copy, PartialEq, Eq, Clone, Debug)]
#[cfg_attr(staged_api, stable(feature = "rust1", since = "1.0.0"))]
pub enum SeekFrom {
    /// Sets the offset to the provided number of bytes.
    #[cfg_attr(staged_api, stable(feature = "rust1", since = "1.0.0"))]
    Start(#[cfg_attr(staged_api, stable(feature = "rust1", since = "1.0.0"))] u64),

    /// Sets the offset to the size of this object plus the specified number of
    /// bytes.
    ///
    /// It is possible to seek beyond the end of an object, but it's an error
    /// to seek before byte 0.
    #[cfg_attr(staged_api, stable(feature = "rust1", since = "1.0.0"))]
    End(#[cfg_attr(staged_api, stable(feature = "rust1", since = "1.0.0"))] i64),

    /// Sets the offset to the current position plus the specified number of
    /// bytes.
    ///
    /// It is possible to seek beyond the end of an object, but it's an error
    /// to seek before byte 0.
    #[cfg_attr(staged_api, stable(feature = "rust1", since = "1.0.0"))]
    Current(#[cfg_attr(staged_api, stable(feature = "rust1", since = "1.0.0"))] i64),
}
