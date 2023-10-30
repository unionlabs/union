use super::Array;

impl<T: Default, const N: usize> Array for [T; N] {
  type Item = T;
  const CAPACITY: usize = N;

  #[inline(always)]
  #[must_use]
  fn as_slice(&self) -> &[T] {
    &*self
  }

  #[inline(always)]
  #[must_use]
  fn as_slice_mut(&mut self) -> &mut [T] {
    &mut *self
  }

  #[inline(always)]
  fn default() -> Self {
    [(); N].map(|_| Default::default())
  }
}
