//! Page related functions.

use crate::os;
use std::sync::Once;

/// Returns the operating system's page size.
///
/// This function uses an internally cached page size, and can be called
/// repeatedly without incurring a significant performance penalty.
///
/// # Examples
///
/// ```
/// # use region::page;
/// let size = page::size(); // Most likely 4096
/// ```
#[inline]
pub fn size() -> usize {
  static INIT: Once = Once::new();
  static mut PAGE_SIZE: usize = 0;

  unsafe {
    INIT.call_once(|| PAGE_SIZE = os::page_size());
    PAGE_SIZE
  }
}

/// Rounds an address down to its closest page boundary.
///
/// # Examples
///
/// ```
/// # use region::page;
/// let unaligned_pointer = (page::size() + 1) as *const ();
///
/// assert_eq!(page::floor(unaligned_pointer), page::size() as *const _);
/// ```
#[inline]
pub fn floor<T>(address: *const T) -> *const T {
  (address as usize & !(size() - 1)) as *const T
}

/// Rounds an address up to its closest page boundary.
///
/// # Examples
///
/// ```
/// # use region::page;
/// let unaligned_pointer = (page::size() - 1) as *const ();
///
/// assert_eq!(page::ceil(unaligned_pointer), page::size() as *const _);
/// ```
#[inline]
pub fn ceil<T>(address: *const T) -> *const T {
  match (address as usize).checked_add(size()) {
    Some(offset) => ((offset - 1) & !(size() - 1)) as *const T,
    None => floor(address),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn page_size_is_reasonable() {
    let pz = size();

    assert!(pz > 0);
    assert_eq!(pz % 2, 0);
    assert_eq!(pz, size());
  }

  #[test]
  fn page_rounding_works() {
    let pz = size();
    let point = 1 as *const ();

    assert_eq!(floor(point) as usize, 0);
    assert_eq!(floor(pz as *const ()) as usize, pz);
    assert_eq!(floor(usize::max_value() as *const ()) as usize % pz, 0);

    assert_eq!(ceil(point) as usize, pz);
    assert_eq!(ceil(pz as *const ()) as usize, pz);
    assert_eq!(ceil(usize::max_value() as *const ()) as usize % pz, 0);
  }
}
