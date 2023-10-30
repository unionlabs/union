use crate::{page, Error, Result};

/// Validates & rounds an address-size pair to their respective page boundary.
pub fn round_to_page_boundaries<T>(address: *const T, size: usize) -> Result<(*const T, usize)> {
  if size == 0 {
    return Err(Error::InvalidParameter("size"));
  }

  let size = (address as usize % page::size()).saturating_add(size);
  let size = page::ceil(size as *const T) as usize;
  Ok((page::floor(address), size))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn round_to_page_boundaries_works() -> Result<()> {
    let pz = page::size();
    let values = &[
      ((1, pz), (0, pz * 2)),
      ((0, pz - 1), (0, pz)),
      ((0, pz + 1), (0, pz * 2)),
      ((pz - 1, 1), (0, pz)),
      ((pz + 1, pz), (pz, pz * 2)),
      ((pz, pz), (pz, pz)),
    ];

    for ((before_address, before_size), (after_address, after_size)) in values {
      let (address, size) = round_to_page_boundaries(*before_address as *const (), *before_size)?;
      assert_eq!((address, size), (*after_address as *const (), *after_size));
    }
    Ok(())
  }
}
