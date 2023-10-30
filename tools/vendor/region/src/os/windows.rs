use crate::{Error, Protection, Region, Result};
use std::cmp::{max, min};
use std::io;
use std::mem::{size_of, MaybeUninit};
use std::sync::Once;
use winapi::um::memoryapi::{
  VirtualAlloc, VirtualFree, VirtualLock, VirtualProtect, VirtualQuery, VirtualUnlock,
};
use winapi::um::sysinfoapi::{GetNativeSystemInfo, SYSTEM_INFO};
use winapi::um::winnt::{MEMORY_BASIC_INFORMATION, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE};

pub struct QueryIter {
  region_address: usize,
  upper_bound: usize,
}

impl QueryIter {
  pub fn new(origin: *const (), size: usize) -> Result<QueryIter> {
    let system = system_info();

    Ok(QueryIter {
      region_address: max(origin as usize, system.lpMinimumApplicationAddress as usize),
      upper_bound: min(
        (origin as usize).saturating_add(size),
        system.lpMaximumApplicationAddress as usize,
      ),
    })
  }

  pub fn upper_bound(&self) -> usize {
    self.upper_bound
  }
}

impl Iterator for QueryIter {
  type Item = Result<Region>;

  fn next(&mut self) -> Option<Self::Item> {
    let mut info: MEMORY_BASIC_INFORMATION = unsafe { std::mem::zeroed() };

    while self.region_address < self.upper_bound {
      let bytes = unsafe {
        VirtualQuery(
          self.region_address as winapi::um::winnt::PVOID,
          &mut info,
          size_of::<MEMORY_BASIC_INFORMATION>() as winapi::shared::basetsd::SIZE_T,
        )
      };

      if bytes == 0 {
        return Some(Err(Error::SystemCall(io::Error::last_os_error())));
      }

      self.region_address = (info.BaseAddress as usize).saturating_add(info.RegionSize);

      // Only mapped memory regions are of interest
      if info.State == MEM_RESERVE || info.State == MEM_COMMIT {
        let mut region = Region {
          base: info.BaseAddress as *const _,
          reserved: info.State != MEM_COMMIT,
          guarded: (info.Protect & winapi::um::winnt::PAGE_GUARD) != 0,
          shared: (info.Type & winapi::um::winnt::MEM_PRIVATE) == 0,
          size: info.RegionSize as usize,
          ..Default::default()
        };

        if region.is_committed() {
          region.protection = Protection::from_native(info.Protect);
        }

        return Some(Ok(region));
      }
    }

    None
  }
}

pub fn page_size() -> usize {
  system_info().dwPageSize as usize
}

pub unsafe fn alloc(base: *const (), size: usize, protection: Protection) -> Result<*const ()> {
  let allocation = VirtualAlloc(
    base as winapi::um::winnt::PVOID,
    size,
    MEM_COMMIT | MEM_RESERVE,
    protection.to_native(),
  );

  if allocation.is_null() {
    return Err(Error::SystemCall(io::Error::last_os_error()));
  }

  Ok(allocation as *const ())
}

pub unsafe fn free(base: *const (), _size: usize) -> Result<()> {
  match VirtualFree(base as winapi::um::winnt::PVOID, 0, MEM_RELEASE) {
    winapi::shared::minwindef::FALSE => Err(Error::SystemCall(io::Error::last_os_error())),
    _ => Ok(()),
  }
}

pub unsafe fn protect(base: *const (), size: usize, protection: Protection) -> Result<()> {
  let result = VirtualProtect(
    base as winapi::um::winnt::PVOID,
    size as winapi::shared::basetsd::SIZE_T,
    protection.to_native(),
    &mut 0,
  );

  if result == winapi::shared::minwindef::FALSE {
    Err(Error::SystemCall(io::Error::last_os_error()))
  } else {
    Ok(())
  }
}

pub fn lock(base: *const (), size: usize) -> Result<()> {
  let result = unsafe {
    VirtualLock(
      base as winapi::um::winnt::PVOID,
      size as winapi::shared::basetsd::SIZE_T,
    )
  };

  if result == winapi::shared::minwindef::FALSE {
    Err(Error::SystemCall(io::Error::last_os_error()))
  } else {
    Ok(())
  }
}

pub fn unlock(base: *const (), size: usize) -> Result<()> {
  let result = unsafe {
    VirtualUnlock(
      base as winapi::um::winnt::PVOID,
      size as winapi::shared::basetsd::SIZE_T,
    )
  };

  if result == winapi::shared::minwindef::FALSE {
    Err(Error::SystemCall(io::Error::last_os_error()))
  } else {
    Ok(())
  }
}

fn system_info() -> &'static SYSTEM_INFO {
  static INIT: Once = Once::new();
  static mut INFO: MaybeUninit<SYSTEM_INFO> = MaybeUninit::uninit();

  unsafe {
    INIT.call_once(|| GetNativeSystemInfo(INFO.as_mut_ptr()));
    &*INFO.as_ptr()
  }
}

impl Protection {
  fn from_native(protection: winapi::shared::minwindef::DWORD) -> Self {
    // Ignore unsupported flags (TODO: Preserve this information?)
    let ignored = winapi::um::winnt::PAGE_GUARD
      | winapi::um::winnt::PAGE_NOCACHE
      | winapi::um::winnt::PAGE_WRITECOMBINE;

    match protection & !ignored {
      winapi::um::winnt::PAGE_EXECUTE => Protection::EXECUTE,
      winapi::um::winnt::PAGE_EXECUTE_READ => Protection::READ_EXECUTE,
      winapi::um::winnt::PAGE_EXECUTE_READWRITE => Protection::READ_WRITE_EXECUTE,
      winapi::um::winnt::PAGE_EXECUTE_WRITECOPY => Protection::READ_WRITE_EXECUTE,
      winapi::um::winnt::PAGE_NOACCESS => Protection::NONE,
      winapi::um::winnt::PAGE_READONLY => Protection::READ,
      winapi::um::winnt::PAGE_READWRITE => Protection::READ_WRITE,
      winapi::um::winnt::PAGE_WRITECOPY => Protection::READ_WRITE,
      _ => unreachable!("Protection: 0x{:X}", protection),
    }
  }

  pub(crate) fn to_native(self) -> winapi::shared::minwindef::DWORD {
    match self {
      Protection::NONE => winapi::um::winnt::PAGE_NOACCESS,
      Protection::READ => winapi::um::winnt::PAGE_READONLY,
      Protection::EXECUTE => winapi::um::winnt::PAGE_EXECUTE,
      Protection::READ_EXECUTE => winapi::um::winnt::PAGE_EXECUTE_READ,
      Protection::READ_WRITE => winapi::um::winnt::PAGE_READWRITE,
      Protection::READ_WRITE_EXECUTE => winapi::um::winnt::PAGE_EXECUTE_READWRITE,
      Protection::WRITE_EXECUTE => winapi::um::winnt::PAGE_EXECUTE_READWRITE,
      _ => unreachable!("Protection: {:?}", self),
    }
  }
}
