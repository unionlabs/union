//! Variable-sized output digest support

use crate::{InvalidOutputSize, Reset};

#[cfg(feature = "alloc")]
use alloc::boxed::Box;

/// Trait for returning digest result with the variable size
pub trait VariableOutput: Sized {
    /// Create new hasher instance with the given output size.
    ///
    /// It will return `Err(InvalidOutputSize)` in case if hasher can not return
    /// specified output size. It will always return an error if output size
    /// equals to zero.
    fn new(output_size: usize) -> Result<Self, InvalidOutputSize>;

    /// Get output size of the hasher instance provided to the `new` method
    fn output_size(&self) -> usize;

    /// Retrieve result via closure and consume hasher.
    ///
    /// Closure is guaranteed to be called, length of the buffer passed to it
    /// will be equal to `output_size`.
    fn finalize_variable(self, f: impl FnOnce(&[u8]));

    /// Retrieve result via closure and reset the hasher state.
    ///
    /// Closure is guaranteed to be called, length of the buffer passed to it
    /// will be equal to `output_size`.
    fn finalize_variable_reset(&mut self, f: impl FnOnce(&[u8]));

    /// Retrieve result into a boxed slice and consume hasher.
    ///
    /// `Box<[u8]>` is used instead of `Vec<u8>` to save stack space, since
    /// they have size of 2 and 3 words respectively.
    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    fn finalize_boxed(self) -> Box<[u8]> {
        let n = self.output_size();
        let mut buf = vec![0u8; n].into_boxed_slice();
        self.finalize_variable(|res| buf.copy_from_slice(res));
        buf
    }

    /// Retrieve result into a boxed slice and reset hasher state.
    ///
    /// `Box<[u8]>` is used instead of `Vec<u8>` to save stack space, since
    /// they have size of 2 and 3 words respectively.
    #[cfg(feature = "alloc")]
    #[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
    fn finalize_boxed_reset(&mut self) -> Box<[u8]> {
        let n = self.output_size();
        let mut buf = vec![0u8; n].into_boxed_slice();
        self.finalize_variable_reset(|res| buf.copy_from_slice(res));
        buf
    }
}

/// Trait for variable-sized output digest implementations to use to retrieve
/// the hash output.
///
/// Usage of this trait in user code is discouraged. Instead use the
/// [`VariableOutput::finalize_variable`] or
/// [`VariableOutput::finalize_variable_reset`] methods.
///
/// Types which impl this trait along with [`Reset`] will receive a blanket
/// impl of [`VariableOutput`].
pub trait VariableOutputDirty: Sized {
    /// Create new hasher instance with the given output size.
    ///
    /// It will return `Err(InvalidOutputSize)` in case if hasher can not return
    /// specified output size. It will always return an error if output size
    /// equals to zero.
    fn new(output_size: usize) -> Result<Self, InvalidOutputSize>;

    /// Get output size of the hasher instance provided to the `new` method
    fn output_size(&self) -> usize;

    /// Retrieve result into provided buffer and leave hasher in a dirty state.
    ///
    /// This method is expected to only be called once unless
    /// [`Reset::reset`] is called, after which point it can be
    /// called again and reset again (and so on).
    fn finalize_variable_dirty(&mut self, f: impl FnOnce(&[u8]));
}

impl<D: VariableOutputDirty + Reset> VariableOutput for D {
    fn new(output_size: usize) -> Result<Self, InvalidOutputSize> {
        <Self as VariableOutputDirty>::new(output_size)
    }

    fn output_size(&self) -> usize {
        <Self as VariableOutputDirty>::output_size(self)
    }

    #[inline]
    fn finalize_variable(mut self, f: impl FnOnce(&[u8])) {
        self.finalize_variable_dirty(f);
    }

    #[inline]
    fn finalize_variable_reset(&mut self, f: impl FnOnce(&[u8])) {
        self.finalize_variable_dirty(f);
        self.reset();
    }
}
