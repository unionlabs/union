//! Fixed-size output digest support

use crate::Reset;
use generic_array::{ArrayLength, GenericArray};

/// Trait for returning digest result with the fixed size
pub trait FixedOutput {
    /// Output size for fixed output digest
    type OutputSize: ArrayLength<u8>;

    /// Write result into provided array and consume the hasher instance.
    fn finalize_into(self, out: &mut GenericArray<u8, Self::OutputSize>);

    /// Write result into provided array and reset the hasher instance.
    fn finalize_into_reset(&mut self, out: &mut GenericArray<u8, Self::OutputSize>);

    /// Retrieve result and consume the hasher instance.
    #[inline]
    fn finalize_fixed(self) -> GenericArray<u8, Self::OutputSize>
    where
        Self: Sized,
    {
        let mut out = Default::default();
        self.finalize_into(&mut out);
        out
    }

    /// Retrieve result and reset the hasher instance.
    #[inline]
    fn finalize_fixed_reset(&mut self) -> GenericArray<u8, Self::OutputSize> {
        let mut out = Default::default();
        self.finalize_into_reset(&mut out);
        out
    }
}

/// Trait for fixed-output digest implementations to use to retrieve the
/// hash output.
///
/// Usage of this trait in user code is discouraged. Instead use the
/// [`FixedOutput::finalize_fixed`] or [`FixedOutput::finalize_fixed_reset`]
/// methods.
///
/// Types which impl this trait along with [`Reset`] will receive a blanket
/// impl of [`FixedOutput`].
pub trait FixedOutputDirty {
    /// Output size for fixed output digest
    type OutputSize: ArrayLength<u8>;

    /// Retrieve result into provided buffer and leave hasher in a dirty state.
    ///
    /// This method is expected to only be called once unless
    /// [`Reset::reset`] is called, after which point it can be
    /// called again and reset again (and so on).
    fn finalize_into_dirty(&mut self, out: &mut GenericArray<u8, Self::OutputSize>);
}

impl<D: FixedOutputDirty + Reset> FixedOutput for D {
    type OutputSize = D::OutputSize;

    #[inline]
    fn finalize_into(mut self, out: &mut GenericArray<u8, Self::OutputSize>) {
        self.finalize_into_dirty(out);
    }

    #[inline]
    fn finalize_into_reset(&mut self, out: &mut GenericArray<u8, Self::OutputSize>) {
        self.finalize_into_dirty(out);
        self.reset();
    }
}
