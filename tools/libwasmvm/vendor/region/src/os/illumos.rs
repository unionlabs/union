use crate::{Error, Protection, Region, Result};
use std::fs::File;
use std::io::Read;

pub struct QueryIter {
  vmmap: Vec<u8>,
  vmmap_index: usize,
  upper_bound: usize,
}

impl QueryIter {
  pub fn new(origin: *const (), size: usize) -> Result<QueryIter> {
    // Do not use a buffered reader here to avoid multiple read(2) calls to the
    // proc file, ensuring a consistent snapshot of the virtual memory.
    let mut file = File::open("/proc/self/map").map_err(Error::SystemCall)?;
    let mut vmmap: Vec<u8> = Vec::with_capacity(8 * PRMAP_SIZE);

    let bytes_read = file.read_to_end(&mut vmmap).map_err(Error::SystemCall)?;

    if bytes_read % PRMAP_SIZE != 0 {
      return Err(Error::ProcfsInput(format!(
        "file size {} is not a multiple of prmap_t size ({})",
        bytes_read, PRMAP_SIZE
      )));
    }

    Ok(QueryIter {
      vmmap,
      vmmap_index: 0,
      upper_bound: (origin as usize).saturating_add(size),
    })
  }

  pub fn upper_bound(&self) -> usize {
    self.upper_bound
  }
}

impl Iterator for QueryIter {
  type Item = Result<Region>;

  fn next(&mut self) -> Option<Self::Item> {
    let (pfx, maps, sfx) = unsafe { self.vmmap.align_to::<PrMap>() };

    if !pfx.is_empty() || !sfx.is_empty() {
      panic!(
        "data was not aligned ({}; {}/{}/{})?",
        self.vmmap.len(),
        pfx.len(),
        maps.len(),
        sfx.len()
      );
    }

    let map = maps.get(self.vmmap_index)?;

    self.vmmap_index += 1;
    Some(Ok(Region {
      base: map.pr_vaddr,
      protection: Protection::from_native(map.pr_mflags),
      shared: map.pr_mflags & MA_SHARED != 0,
      size: map.pr_size,
      ..Default::default()
    }))
  }
}

impl Protection {
  fn from_native(protection: i32) -> Self {
    const MAPPINGS: &[(i32, Protection)] = &[
      (MA_READ, Protection::READ),
      (MA_WRITE, Protection::WRITE),
      (MA_EXEC, Protection::EXECUTE),
    ];

    MAPPINGS
      .iter()
      .filter(|(flag, _)| protection & *flag == *flag)
      .fold(Protection::NONE, |acc, (_, prot)| acc | *prot)
  }
}

// As per proc(4), the file `/proc/$PID/map` contains an array of C structs of
// type `prmap_t`. The layout of this struct, and thus this file, is a stable
// interface.
#[repr(C)]
struct PrMap {
  pr_vaddr: *const (),
  pr_size: usize,
  pr_mapname: [i8; 64],
  pr_offset: isize,
  pr_mflags: i32,
  pr_pagesize: i32,
  pr_shmid: i32,
  _pr_filler: [i32; 1],
}

const PRMAP_SIZE: usize = std::mem::size_of::<PrMap>();

// These come from <sys/procfs.h>, describing bits in the pr_mflags member:
const MA_EXEC: i32 = 0x1;
const MA_WRITE: i32 = 0x2;
const MA_READ: i32 = 0x4;
const MA_SHARED: i32 = 0x8;

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn protection_flags_are_mapped_from_native() {
    let rw = MA_READ | MA_WRITE;
    let rwx = rw | MA_EXEC;

    assert_eq!(Protection::from_native(0), Protection::NONE);
    assert_eq!(Protection::from_native(MA_READ), Protection::READ);
    assert_eq!(Protection::from_native(rw), Protection::READ_WRITE);
    assert_eq!(Protection::from_native(rwx), Protection::READ_WRITE_EXECUTE);
  }
}
