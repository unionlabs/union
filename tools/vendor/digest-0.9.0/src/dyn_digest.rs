#![cfg(feature = "alloc")]
use alloc::boxed::Box;

use super::{FixedOutput, Reset, Update};
use generic_array::typenum::Unsigned;

/// The `DynDigest` trait is a modification of `Digest` trait suitable
/// for trait objects.
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
pub trait DynDigest {
    /// Digest input data.
    ///
    /// This method can be called repeatedly for use with streaming messages.
    fn update(&mut self, data: &[u8]);

    /// Retrieve result and reset hasher instance
    fn finalize_reset(&mut self) -> Box<[u8]>;

    /// Retrieve result and consume boxed hasher instance
    fn finalize(self: Box<Self>) -> Box<[u8]>;

    /// Reset hasher instance to its initial state.
    fn reset(&mut self);

    /// Get output size of the hasher
    fn output_size(&self) -> usize;

    /// Clone hasher state into a boxed trait object
    fn box_clone(&self) -> Box<dyn DynDigest>;
}

impl<D: Update + FixedOutput + Reset + Clone + 'static> DynDigest for D {
    fn update(&mut self, data: &[u8]) {
        Update::update(self, data);
    }

    fn finalize_reset(&mut self) -> Box<[u8]> {
        let res = self.finalize_fixed_reset().to_vec().into_boxed_slice();
        Reset::reset(self);
        res
    }

    fn finalize(self: Box<Self>) -> Box<[u8]> {
        self.finalize_fixed().to_vec().into_boxed_slice()
    }

    fn reset(&mut self) {
        Reset::reset(self);
    }

    fn output_size(&self) -> usize {
        <Self as FixedOutput>::OutputSize::to_usize()
    }

    fn box_clone(&self) -> Box<dyn DynDigest> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn DynDigest> {
    fn clone(&self) -> Self {
        self.box_clone()
    }
}
