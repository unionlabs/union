use std::ops::{Deref, Range};

/// The underlying buffer that is used by a flexbuffer Reader.
///
/// This allows for custom buffer implementations as long as they can be viewed as a &[u8].
pub trait Buffer: Deref<Target = [u8]> + Sized {
    // The `BufferString` allows for a buffer to return a custom string which will have the
    // lifetime of the underlying buffer. A simple `std::str::from_utf8` wouldn't work since that
    // returns a &str, which is then owned by the callee (cannot be returned from a function).
    //
    // Example: During deserialization a `BufferString` is returned, allowing the deserializer
    // to "borrow" the given str - b/c there is a "lifetime" guarantee, so to speak, from the
    // underlying buffer.
    /// A BufferString which will live at least as long as the Buffer itself.
    ///
    /// Deref's to UTF-8 `str`, and only generated from the `buffer_str` function Result.
    type BufferString: Deref<Target = str> + Sized + serde::ser::Serialize;

    /// This method returns an instance of type Self. This allows for lifetimes to be tracked
    /// in cases of deserialization.
    ///
    /// It also lets custom buffers manage reference counts.
    ///
    /// Returns None if:
    /// - range start is greater than end
    /// - range end is out of bounds
    ///
    /// This operation should be fast -> O(1), ideally with no heap allocations.
    fn slice(&self, range: Range<usize>) -> Option<Self>;

    /// Creates a shallow copy of the given buffer, similar to `slice`.
    ///
    /// This operation should be fast -> O(1), ideally with no heap allocations.
    #[inline]
    fn shallow_copy(&self) -> Self {
        self.slice(0..self.len()).unwrap()
    }

    /// Creates an empty instance of a `Buffer`. This is different than `Default` b/c it
    /// guarantees that the buffer instance will have length zero.
    ///
    /// Most impls shold be able to implement this via `Default`.
    fn empty() -> Self;

    /// Based off of the `empty` function, allows override for optimization purposes.
    #[inline]
    fn empty_str() -> Self::BufferString {
        Self::empty().buffer_str().unwrap()
    }

    /// Attempts to convert the given buffer to a custom string type.
    ///
    /// This should fail if the type does not have valid UTF-8 bytes, and must be zero copy.
    fn buffer_str(&self) -> Result<Self::BufferString, std::str::Utf8Error>;
}

impl<'de> Buffer for &'de [u8] {
    type BufferString = &'de str;

    #[inline]
    fn slice(&self, range: Range<usize>) -> Option<Self> {
        self.get(range)
    }

    #[inline]
    fn empty() -> Self {
        &[]
    }

    /// Based off of the `empty` function, allows override for optimization purposes.
    #[inline]
    fn empty_str() -> Self::BufferString {
        ""
    }

    #[inline]
    fn buffer_str(&self) -> Result<Self::BufferString, std::str::Utf8Error> {
        std::str::from_utf8(self)
    }
}
