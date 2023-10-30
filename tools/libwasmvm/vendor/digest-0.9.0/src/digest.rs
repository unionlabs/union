use super::{FixedOutput, Reset, Update};
use generic_array::typenum::Unsigned;
use generic_array::{ArrayLength, GenericArray};

/// The `Digest` trait specifies an interface common for digest functions.
///
/// It's a convenience wrapper around [`Update`], [`FixedOutput`], [`Reset`],
/// [`Clone`], and [`Default`] traits. It also provides additional convenience methods.
pub trait Digest {
    /// Output size for `Digest`
    type OutputSize: ArrayLength<u8>;

    /// Create new hasher instance
    fn new() -> Self;

    /// Digest data, updating the internal state.
    ///
    /// This method can be called repeatedly for use with streaming messages.
    fn update(&mut self, data: impl AsRef<[u8]>);

    /// Digest input data in a chained manner.
    fn chain(self, data: impl AsRef<[u8]>) -> Self
    where
        Self: Sized;

    /// Retrieve result and consume hasher instance.
    fn finalize(self) -> Output<Self>;

    /// Retrieve result and reset hasher instance.
    ///
    /// This method sometimes can be more efficient compared to hasher
    /// re-creation.
    fn finalize_reset(&mut self) -> Output<Self>;

    /// Reset hasher instance to its initial state.
    fn reset(&mut self);

    /// Get output size of the hasher
    fn output_size() -> usize;

    /// Convenience function to compute hash of the `data`. It will handle
    /// hasher creation, data feeding and finalization.
    ///
    /// Example:
    ///
    /// ```rust,ignore
    /// println!("{:x}", sha2::Sha256::digest(b"Hello world"));
    /// ```
    fn digest(data: &[u8]) -> Output<Self>;
}

impl<D: Update + FixedOutput + Reset + Clone + Default> Digest for D {
    type OutputSize = <Self as FixedOutput>::OutputSize;

    fn new() -> Self {
        Self::default()
    }

    fn update(&mut self, data: impl AsRef<[u8]>) {
        Update::update(self, data);
    }

    fn chain(self, data: impl AsRef<[u8]>) -> Self
    where
        Self: Sized,
    {
        Update::chain(self, data)
    }

    fn finalize(self) -> Output<Self> {
        self.finalize_fixed()
    }

    fn finalize_reset(&mut self) -> Output<Self> {
        let res = self.clone().finalize_fixed();
        self.reset();
        res
    }

    fn reset(&mut self) {
        <Self as Reset>::reset(self)
    }

    fn output_size() -> usize {
        Self::OutputSize::to_usize()
    }

    fn digest(data: &[u8]) -> Output<Self> {
        let mut hasher = Self::default();
        Update::update(&mut hasher, data);
        hasher.finalize_fixed()
    }
}

/// Output of a [`Digest`] function
pub type Output<D> = GenericArray<u8, <D as Digest>::OutputSize>;
