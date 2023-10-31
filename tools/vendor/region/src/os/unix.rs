use crate::{Error, Protection, Result};
use libc::{MAP_ANON, MAP_FAILED, MAP_FIXED, MAP_PRIVATE};
use libc::{PROT_EXEC, PROT_NONE, PROT_READ, PROT_WRITE};
use std::io;

pub fn page_size() -> usize {
  unsafe { libc::sysconf(libc::_SC_PAGESIZE) as usize }
}

pub unsafe fn alloc(base: *const (), size: usize, protection: Protection) -> Result<*const ()> {
  let mut flags = MAP_PRIVATE | MAP_ANON;

  if !base.is_null() {
    flags |= MAP_FIXED;
  }

  match libc::mmap(base as *mut _, size, protection.to_native(), flags, -1, 0) {
    MAP_FAILED => Err(Error::SystemCall(io::Error::last_os_error())),
    address => Ok(address as *const ()),
  }
}

pub unsafe fn free(base: *const (), size: usize) -> Result<()> {
  match libc::munmap(base as *mut _, size) {
    0 => Ok(()),
    _ => Err(Error::SystemCall(io::Error::last_os_error())),
  }
}

pub unsafe fn protect(base: *const (), size: usize, protection: Protection) -> Result<()> {
  match libc::mprotect(base as *mut _, size, protection.to_native()) {
    0 => Ok(()),
    _ => Err(Error::SystemCall(io::Error::last_os_error())),
  }
}

pub fn lock(base: *const (), size: usize) -> Result<()> {
  match unsafe { libc::mlock(base.cast(), size) } {
    0 => Ok(()),
    _ => Err(Error::SystemCall(io::Error::last_os_error())),
  }
}

pub fn unlock(base: *const (), size: usize) -> Result<()> {
  match unsafe { libc::munlock(base.cast(), size) } {
    0 => Ok(()),
    _ => Err(Error::SystemCall(io::Error::last_os_error())),
  }
}

impl Protection {
  fn to_native(self) -> libc::c_int {
    const MAPPINGS: &[(Protection, libc::c_int)] = &[
      (Protection::READ, PROT_READ),
      (Protection::WRITE, PROT_WRITE),
      (Protection::EXECUTE, PROT_EXEC),
    ];

    MAPPINGS
      .iter()
      .filter(|(flag, _)| self & *flag == *flag)
      .fold(PROT_NONE, |acc, (_, prot)| acc | *prot)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn protection_flags_are_mapped_to_native() {
    let rwx = PROT_READ | PROT_WRITE | PROT_EXEC;

    assert_eq!(Protection::NONE.to_native(), 0);
    assert_eq!(Protection::READ.to_native(), PROT_READ);
    assert_eq!(Protection::READ_WRITE.to_native(), PROT_READ | PROT_WRITE);
    assert_eq!(Protection::READ_WRITE_EXECUTE.to_native(), rwx);
  }
}
