use crate::{Error, Protection, Region, Result};
use libc::{c_int, c_uint, c_ulong, getpid, sysctl, CTL_KERN, KERN_PROC_VMMAP};
use std::io;

pub struct QueryIter {
  mib: [c_int; 3],
  vmentry: kinfo_vmentry,
  previous_boundary: usize,
  upper_bound: usize,
}

impl QueryIter {
  pub fn new(origin: *const (), size: usize) -> Result<QueryIter> {
    Ok(QueryIter {
      mib: [CTL_KERN, KERN_PROC_VMMAP, unsafe { getpid() }],
      vmentry: unsafe { std::mem::zeroed() },
      upper_bound: (origin as usize).saturating_add(size),
      previous_boundary: 0,
    })
  }

  pub fn upper_bound(&self) -> usize {
    self.upper_bound
  }
}

impl Iterator for QueryIter {
  type Item = Result<Region>;

  fn next(&mut self) -> Option<Self::Item> {
    let mut len = std::mem::size_of::<kinfo_vmentry>();

    // Albeit it would be preferred to query the information for all virtual
    // pages at once, the system call does not seem to respond consistently. If
    // called once during a process' lifetime, it returns all pages, but if
    // called again, it returns an empty buffer. This may be caused due to an
    // oversight, but in this case, the solution is to query one memory region
    // at a time.
    let result = unsafe {
      sysctl(
        self.mib.as_ptr(),
        self.mib.len() as c_uint,
        &mut self.vmentry as *mut _ as *mut _,
        &mut len,
        std::ptr::null_mut(),
        0,
      )
    };

    if result == -1 {
      return Some(Err(Error::SystemCall(io::Error::last_os_error())));
    }

    if len == 0 || self.vmentry.kve_end as usize == self.previous_boundary {
      return None;
    }

    let region = Region {
      base: self.vmentry.kve_start as *const _,
      protection: Protection::from_native(self.vmentry.kve_protection),
      shared: (self.vmentry.kve_etype & KVE_ET_COPYONWRITE) == 0,
      size: (self.vmentry.kve_end - self.vmentry.kve_start) as _,
      ..Default::default()
    };

    // Since OpenBSD returns the first region whose base address is at, or after
    // `kve_start`, the address can simply be incremented by one to retrieve the
    // next region.
    self.vmentry.kve_start += 1;
    self.previous_boundary = self.vmentry.kve_end as usize;
    Some(Ok(region))
  }
}

impl Protection {
  fn from_native(protection: c_int) -> Self {
    const MAPPINGS: &[(c_int, Protection)] = &[
      (KVE_PROT_READ, Protection::READ),
      (KVE_PROT_WRITE, Protection::WRITE),
      (KVE_PROT_EXEC, Protection::EXECUTE),
    ];

    MAPPINGS
      .iter()
      .filter(|(flag, _)| protection & *flag == *flag)
      .fold(Protection::NONE, |acc, (_, prot)| acc | *prot)
  }
}

// These defintions come from <sys/sysctl.h>, describing data returned by the
// `KERN_PROC_VMMAP` system call.
#[repr(C)]
struct kinfo_vmentry {
  kve_start: c_ulong,
  kve_end: c_ulong,
  kve_guard: c_ulong,
  kve_fspace: c_ulong,
  kve_fspace_augment: c_ulong,
  kve_offset: u64,
  kve_wired_count: c_int,
  kve_etype: c_int,
  kve_protection: c_int,
  kve_max_protection: c_int,
  kve_advice: c_int,
  kve_inheritance: c_int,
  kve_flags: u8,
}

const KVE_PROT_READ: c_int = 1;
const KVE_PROT_WRITE: c_int = 2;
const KVE_PROT_EXEC: c_int = 4;
const KVE_ET_COPYONWRITE: c_int = 4;

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn protection_flags_are_mapped_from_native() {
    let rw = KVE_PROT_READ | KVE_PROT_WRITE;
    let rwx = rw | KVE_PROT_EXEC;

    assert_eq!(Protection::from_native(0), Protection::NONE);
    assert_eq!(Protection::from_native(KVE_PROT_READ), Protection::READ);
    assert_eq!(Protection::from_native(rw), Protection::READ_WRITE);
    assert_eq!(Protection::from_native(rwx), Protection::READ_WRITE_EXECUTE);
  }
}
