use crate::{Error, Protection, Region, Result};
use mach::vm_prot::*;

pub struct QueryIter {
  region_address: mach::vm_types::mach_vm_address_t,
  upper_bound: usize,
}

impl QueryIter {
  pub fn new(origin: *const (), size: usize) -> Result<QueryIter> {
    Ok(QueryIter {
      region_address: origin as _,
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
    // The possible memory share modes
    const SHARE_MODES: [u8; 3] = [
      mach::vm_region::SM_SHARED,
      mach::vm_region::SM_TRUESHARED,
      mach::vm_region::SM_SHARED_ALIASED,
    ];

    // Check if the search area has been passed
    if self.region_address as usize >= self.upper_bound {
      return None;
    }

    let mut region_size: mach::vm_types::mach_vm_size_t = 0;
    let mut info: mach::vm_region::vm_region_extended_info = unsafe { std::mem::zeroed() };

    let result = unsafe {
      // This returns the closest region that is at, or after, the address.
      mach::vm::mach_vm_region(
        mach::traps::mach_task_self(),
        &mut self.region_address,
        &mut region_size,
        mach::vm_region::VM_REGION_EXTENDED_INFO,
        (&mut info as *mut _) as mach::vm_region::vm_region_info_t,
        &mut mach::vm_region::vm_region_extended_info::count(),
        &mut 0,
      )
    };

    match result {
      // The end of the process' address space has been reached
      mach::kern_return::KERN_INVALID_ADDRESS => None,
      mach::kern_return::KERN_SUCCESS => {
        // The returned region may have a different address than the request
        if self.region_address as usize >= self.upper_bound {
          return None;
        }

        let region = Region {
          base: self.region_address as *const _,
          guarded: (info.user_tag == mach::vm_statistics::VM_MEMORY_GUARD),
          protection: Protection::from_native(info.protection),
          shared: SHARE_MODES.contains(&info.share_mode),
          size: region_size as usize,
          ..Default::default()
        };

        self.region_address = self.region_address.saturating_add(region_size);
        Some(Ok(region))
      }
      _ => Some(Err(Error::MachCall(result))),
    }
  }
}

impl Protection {
  fn from_native(protection: vm_prot_t) -> Self {
    const MAPPINGS: &[(vm_prot_t, Protection)] = &[
      (VM_PROT_READ, Protection::READ),
      (VM_PROT_WRITE, Protection::WRITE),
      (VM_PROT_EXECUTE, Protection::EXECUTE),
    ];

    MAPPINGS
      .iter()
      .filter(|(flag, _)| protection & *flag == *flag)
      .fold(Protection::NONE, |acc, (_, prot)| acc | *prot)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn protection_flags_are_mapped_from_native() {
    let rw = VM_PROT_READ | VM_PROT_WRITE;
    let rwx = rw | VM_PROT_EXECUTE;

    assert_eq!(Protection::from_native(0), Protection::NONE);
    assert_eq!(Protection::from_native(VM_PROT_READ), Protection::READ);
    assert_eq!(Protection::from_native(rw), Protection::READ_WRITE);
    assert_eq!(Protection::from_native(rwx), Protection::READ_WRITE_EXECUTE);
  }
}
