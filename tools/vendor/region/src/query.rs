use crate::{os, util, Error, Region, Result};

/// An iterator over the [`Region`]s that encompass an address range.
///
/// This `struct` is created by [`query_range`]. See its documentation for more.
pub struct QueryIter {
  iterator: Option<os::QueryIter>,
  origin: *const (),
}

impl QueryIter {
  pub(crate) fn new<T>(origin: *const T, size: usize) -> Result<Self> {
    let origin = origin.cast();

    os::QueryIter::new(origin, size).map(|iterator| Self {
      iterator: Some(iterator),
      origin,
    })
  }
}

impl Iterator for QueryIter {
  type Item = Result<Region>;

  /// Advances the iterator and returns the next region.
  ///
  /// If the iterator has been exhausted (i.e. all [`Region`]s have been
  /// queried), or if an error is encountered during iteration, all further
  /// invocations will return [`None`] (in the case of an error, the error will
  /// be the last item that is yielded before the iterator is fused).
  #[allow(clippy::missing_inline_in_public_items)]
  fn next(&mut self) -> Option<Self::Item> {
    let regions = self.iterator.as_mut()?;

    while let Some(result) = regions.next() {
      match result {
        Ok(region) => {
          let range = region.as_range();

          // Skip the region if it is prior to the queried range
          if range.end <= self.origin as usize {
            continue;
          }

          // Stop iteration if the region is after the queried range
          if range.start >= regions.upper_bound() {
            break;
          }

          return Some(Ok(region));
        }
        Err(error) => {
          self.iterator.take();
          return Some(Err(error));
        }
      }
    }

    self.iterator.take();
    None
  }
}

impl std::iter::FusedIterator for QueryIter {}

unsafe impl Send for QueryIter {}
unsafe impl Sync for QueryIter {}

/// Queries the OS with an address, returning the region it resides within.
///
/// If the queried address does not reside within any mapped region, or if it's
/// outside the process' address space, the function will error with
/// [`Error::UnmappedRegion`].
///
/// # Parameters
///
/// - The enclosing region can be of multiple page sizes.
/// - The address is rounded down to the closest page boundary.
///
/// # Errors
///
/// - If an interaction with the underlying operating system fails, an error
/// will be returned.
///
/// # Examples
///
/// ```
/// # fn main() -> region::Result<()> {
/// use region::Protection;
///
/// let data = [0; 100];
/// let region = region::query(data.as_ptr())?;
///
/// assert_eq!(region.protection(), Protection::READ_WRITE);
/// # Ok(())
/// # }
/// ```
#[inline]
pub fn query<T>(address: *const T) -> Result<Region> {
  // For UNIX systems, the address must be aligned to the closest page boundary
  let (address, size) = util::round_to_page_boundaries(address, 1)?;

  QueryIter::new(address, size)?
    .next()
    .ok_or(Error::UnmappedRegion)?
}

/// Queries the OS for mapped regions that overlap with the specified range.
///
/// The implementation clamps any input that exceeds the boundaries of a
/// process' address space. Therefore it's safe to, e.g., pass in
/// [`std::ptr::null`] and [`usize::max_value`] to iterate the mapped memory
/// pages of an entire process.
///
/// If an error is encountered during iteration, the error will be the last item
/// that is yielded. Thereafter the iterator becomes fused.
///
/// A 2-byte range straddling a page boundary, will return both pages (or one
/// region, if the pages share the same properties).
///
/// This function only returns mapped regions. If required, unmapped regions can
/// be manually identified by inspecting the potential gaps between two
/// neighboring regions.
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
/// - If size is zero, [`Error::InvalidParameter`] will be returned.
///
/// # Examples
///
/// ```
/// # use region::Result;
/// # fn main() -> Result<()> {
/// let data = [0; 100];
/// let region = region::query_range(data.as_ptr(), data.len())?
///   .collect::<Result<Vec<_>>>()?;
///
/// assert_eq!(region.len(), 1);
/// assert_eq!(region[0].protection(), region::Protection::READ_WRITE);
/// # Ok(())
/// # }
/// ```
#[inline]
pub fn query_range<T>(address: *const T, size: usize) -> Result<QueryIter> {
  let (address, size) = util::round_to_page_boundaries(address, size)?;
  QueryIter::new(address, size)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::tests::util::alloc_pages;
  use crate::{page, Protection};

  #[test]
  fn query_returns_unmapped_for_oob_address() {
    let (min, max) = (std::ptr::null::<()>(), usize::max_value() as *const ());
    assert!(matches!(query(min), Err(Error::UnmappedRegion)));
    assert!(matches!(query(max), Err(Error::UnmappedRegion)));
  }

  #[test]
  fn query_returns_correct_descriptor_for_text_segment() -> Result<()> {
    let region = query(query_returns_correct_descriptor_for_text_segment as *const ())?;
    assert_eq!(region.protection(), Protection::READ_EXECUTE);
    assert_eq!(region.is_shared(), cfg!(windows));
    assert!(!region.is_guarded());
    Ok(())
  }

  #[test]
  fn query_returns_one_region_for_multiple_page_allocation() -> Result<()> {
    let alloc = crate::alloc(page::size() + 1, Protection::READ_EXECUTE)?;
    let region = query(alloc.as_ptr::<()>())?;

    assert_eq!(region.protection(), Protection::READ_EXECUTE);
    assert_eq!(region.as_ptr::<()>(), alloc.as_ptr());
    assert_eq!(region.len(), alloc.len());
    assert!(!region.is_guarded());
    Ok(())
  }

  #[test]
  fn query_is_not_off_by_one() -> Result<()> {
    let pages = [Protection::READ, Protection::READ_EXECUTE, Protection::READ];
    let map = alloc_pages(&pages);

    let page_mid = unsafe { map.as_ptr().add(page::size()) };
    let region = query(page_mid)?;

    assert_eq!(region.protection(), Protection::READ_EXECUTE);
    assert_eq!(region.len(), page::size());

    let region = query(unsafe { page_mid.offset(-1) })?;

    assert_eq!(region.protection(), Protection::READ);
    assert_eq!(region.len(), page::size());
    Ok(())
  }

  #[test]
  fn query_range_does_not_return_unmapped_regions() -> Result<()> {
    let regions = query_range(std::ptr::null::<()>(), 1)?.collect::<Result<Vec<_>>>()?;
    assert!(regions.is_empty());
    Ok(())
  }

  #[test]
  fn query_range_returns_both_regions_for_straddling_range() -> Result<()> {
    let pages = [Protection::READ_EXECUTE, Protection::READ_WRITE];
    let map = alloc_pages(&pages);

    // Query an area that overlaps both pages
    let address = unsafe { map.as_ptr().offset(page::size() as isize - 1) };
    let regions = query_range(address, 2)?.collect::<Result<Vec<_>>>()?;

    assert_eq!(regions.len(), pages.len());
    for (page, region) in pages.iter().zip(regions.iter()) {
      assert_eq!(*page, region.protection);
    }
    Ok(())
  }

  #[test]
  fn query_range_has_inclusive_lower_and_exclusive_upper_bound() -> Result<()> {
    let pages = [Protection::READ, Protection::READ_WRITE, Protection::READ];
    let map = alloc_pages(&pages);

    let regions = query_range(map.as_ptr(), page::size())?.collect::<Result<Vec<_>>>()?;
    assert_eq!(regions.len(), 1);
    assert_eq!(regions[0].protection(), Protection::READ);

    let regions = query_range(map.as_ptr(), page::size() + 1)?.collect::<Result<Vec<_>>>()?;
    assert_eq!(regions.len(), 2);
    assert_eq!(regions[0].protection(), Protection::READ);
    assert_eq!(regions[1].protection(), Protection::READ_WRITE);
    Ok(())
  }

  #[test]
  fn query_range_can_iterate_over_entire_process() -> Result<()> {
    let regions =
      query_range(std::ptr::null::<()>(), usize::max_value())?.collect::<Result<Vec<_>>>()?;
    let (r, rw, rx) = (
      Protection::READ,
      Protection::READ_WRITE,
      Protection::READ_EXECUTE,
    );

    // This test is a bit rough around the edges
    assert!(regions.iter().any(|region| region.protection() == r));
    assert!(regions.iter().any(|region| region.protection() == rw));
    assert!(regions.iter().any(|region| region.protection() == rx));
    assert!(regions.len() > 5);
    Ok(())
  }

  #[test]
  fn query_range_iterator_is_fused_after_exhaustion() -> Result<()> {
    let pages = [Protection::READ, Protection::READ_WRITE];
    let map = alloc_pages(&pages);
    let mut iter = query_range(map.as_ptr(), page::size() + 1)?;

    assert_eq!(
      iter.next().transpose()?.map(|r| r.protection()),
      Some(Protection::READ)
    );
    assert_eq!(
      iter.next().transpose()?.map(|r| r.protection()),
      Some(Protection::READ_WRITE)
    );
    assert_eq!(iter.next().transpose()?, None);
    assert_eq!(iter.next().transpose()?, None);
    Ok(())
  }
}
