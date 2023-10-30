//! Extendable-Output Function (XOF) support

use crate::Reset;

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

/// Trait for describing readers which are used to extract extendable output
/// from XOF (extendable-output function) result.
pub trait XofReader {
    /// Read output into the `buffer`. Can be called an unlimited number of times.
    fn read(&mut self, buffer: &mut [u8]);

    /// Read output into a boxed slice of the specified size.
    ///
    /// Can be called an unlimited number of times in combination with `read`.
    ///
    /// `Box<[u8]>` is used instead of `Vec<u8>` to save stack space, since
    /// they have size of 2 and 3 words respectively.
    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    fn read_boxed(&mut self, n: usize) -> Box<[u8]> {
        let mut buf = vec![0u8; n].into_boxed_slice();
        self.read(&mut buf);
        buf
    }
}

/// Trait which describes extendable-output functions (XOF).
pub trait ExtendableOutput: Sized {
    /// Reader
    type Reader: XofReader;

    /// Retrieve XOF reader and consume hasher instance.
    fn finalize_xof(self) -> Self::Reader;

    /// Retrieve XOF reader and reset hasher instance state.
    fn finalize_xof_reset(&mut self) -> Self::Reader;

    /// Retrieve result into a boxed slice of the specified size and consume
    /// the hasher.
    ///
    /// `Box<[u8]>` is used instead of `Vec<u8>` to save stack space, since
    /// they have size of 2 and 3 words respectively.
    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    fn finalize_boxed(self, n: usize) -> Box<[u8]> {
        let mut buf = vec![0u8; n].into_boxed_slice();
        self.finalize_xof().read(&mut buf);
        buf
    }

    /// Retrieve result into a boxed slice of the specified size and reset
    /// the hasher's state.
    ///
    /// `Box<[u8]>` is used instead of `Vec<u8>` to save stack space, since
    /// they have size of 2 and 3 words respectively.
    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    fn finalize_boxed_reset(&mut self, n: usize) -> Box<[u8]> {
        let mut buf = vec![0u8; n].into_boxed_slice();
        self.finalize_xof_reset().read(&mut buf);
        buf
    }
}

/// Trait for extendable-output function (XOF) implementations to use to
/// retrieve the hash output.
///
/// Usage of this trait in user code is discouraged. Instead use the
/// [`ExtendableOutput::finalize_xof`] or
/// [`ExtendableOutput::finalize_xof_reset`] methods.
///
/// Types which impl this trait along with [`Reset`] will receive a blanket
/// impl of [`ExtendableOutput`].
pub trait ExtendableOutputDirty: Sized {
    /// Reader
    type Reader: XofReader;

    /// Retrieve XOF reader.
    ///
    /// This method is expected to only be called once unless
    /// [`Reset::reset`] is called, after which point it can be
    /// called again and reset again (and so on).
    fn finalize_xof_dirty(&mut self) -> Self::Reader;
}

impl<X: ExtendableOutputDirty + Reset> ExtendableOutput for X {
    type Reader = X::Reader;

    #[inline]
    fn finalize_xof(mut self) -> Self::Reader {
        self.finalize_xof_dirty()
    }

    #[inline]
    fn finalize_xof_reset(&mut self) -> Self::Reader {
        let reader = self.finalize_xof_dirty();
        self.reset();
        reader
    }
}
