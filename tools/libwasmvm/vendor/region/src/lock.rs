use crate::{os, util, Result};

/// Locks one or more memory regions to RAM.
///
/// The memory pages within the address range is guaranteed to stay in RAM
/// except for specials cases, such as hibernation and memory starvation. It
/// returns a [`LockGuard`], which [`unlock`]s the affected regions once
/// dropped.
///
/// # Parameters
///
/// - The range is `[address, address + size)`
/// - The address is rounded down to the closest page boundary.
/// - The size may not be zero.
/// - The size is rounded up to the closest page boundary, relative to the
///   address.
///
/// # Errors
///
/// - If an interaction with the underlying operating system fails, an error
/// will be returned.
/// - If size is zero,
/// [`Error::InvalidParameter`](crate::Error::InvalidParameter) will be
/// returned.
///
/// # Examples
///
/// ```
/// # fn main() -> region::Result<()> {
/// let data = [0; 100];
/// let _guard = region::lock(data.as_ptr(), data.len())?;
/// # Ok(())
/// # }
/// ```
#[inline]
pub fn lock<T>(address: *const T, size: usize) -> Result<LockGuard> {
  let (address, size) = util::round_to_page_boundaries(address, size)?;
  os::lock(address.cast(), size).map(|_| LockGuard::new(address, size))
}

/// Unlocks one or more memory regions from RAM.
///
/// If possible, prefer to use [`lock`] combined with the [`LockGuard`].
///
/// # Parameters
///
/// - The range is `[address, address + size)`
/// - The address is rounded down to the closest page boundary.
/// - The size may not be zero.
/// - The size is rounded up to the closest page boundary, relative to the
///   address.
///
/// # Errors
///
/// - If an interaction with the underlying operating system fails, an error
/// will be returned.
/// - If size is zero,
/// [`Error::InvalidParameter`](crate::Error::InvalidParameter) will be
/// returned.
#[inline]
pub fn unlock<T>(address: *const T, size: usize) -> Result<()> {
  let (address, size) = util::round_to_page_boundaries(address, size)?;
  os::unlock(address.cast(), size)
}

/// A RAII implementation of a scoped lock.
///
/// When this structure is dropped (falls out of scope), the virtual lock will be
/// released.
#[must_use]
pub struct LockGuard {
  address: *const (),
  size: usize,
}

impl LockGuard {
  #[inline(always)]
  fn new<T>(address: *const T, size: usize) -> Self {
    Self {
      address: address.cast(),
      size,
    }
  }
}

impl Drop for LockGuard {
  #[inline]
  fn drop(&mut self) {
    let result = os::unlock(self.address, self.size);
    debug_assert!(result.is_ok(), "unlocking region: {:?}", result);
  }
}

unsafe impl Send for LockGuard {}
unsafe impl Sync for LockGuard {}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::tests::util::alloc_pages;
  use crate::{page, Protection};

  #[test]
  fn lock_mapped_pages_succeeds() -> Result<()> {
    let map = alloc_pages(&[Protection::READ_WRITE]);
    let _guard = lock(map.as_ptr(), page::size())?;
    Ok(())
  }

  #[test]
  fn unlock_mapped_pages_succeeds() -> Result<()> {
    let map = alloc_pages(&[Protection::READ_WRITE]);
    std::mem::forget(lock(map.as_ptr(), page::size())?);
    unlock(map.as_ptr(), page::size())
  }
}
