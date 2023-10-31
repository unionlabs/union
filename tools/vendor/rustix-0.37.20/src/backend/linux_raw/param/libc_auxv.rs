//! Linux auxv support, using libc.
//!
//! # Safety
//!
//! This uses raw pointers to locate and read the kernel-provided auxv array.
#![allow(unsafe_code)]

use super::super::elf::*;
#[cfg(feature = "param")]
use crate::ffi::CStr;
use core::ptr::null;
#[cfg(feature = "runtime")]
use core::slice;

// `getauxval` wasn't supported in glibc until 2.16.
weak!(fn getauxval(libc::c_ulong) -> *mut libc::c_void);

#[cfg(feature = "param")]
#[inline]
pub(crate) fn page_size() -> usize {
    unsafe { libc::sysconf(libc::_SC_PAGESIZE) as usize }
}

#[cfg(feature = "param")]
#[inline]
pub(crate) fn clock_ticks_per_second() -> u64 {
    unsafe { libc::sysconf(libc::_SC_CLK_TCK) as u64 }
}

#[cfg(feature = "param")]
#[inline]
pub(crate) fn linux_hwcap() -> (usize, usize) {
    if let Some(libc_getauxval) = getauxval.get() {
        unsafe {
            let hwcap = libc_getauxval(libc::AT_HWCAP) as usize;
            let hwcap2 = libc_getauxval(libc::AT_HWCAP2) as usize;
            (hwcap, hwcap2)
        }
    } else {
        (0, 0)
    }
}

#[cfg(feature = "param")]
#[inline]
pub(crate) fn linux_execfn() -> &'static CStr {
    if let Some(libc_getauxval) = getauxval.get() {
        unsafe { CStr::from_ptr(libc_getauxval(libc::AT_EXECFN).cast()) }
    } else {
        cstr!("")
    }
}

#[cfg(feature = "runtime")]
#[inline]
pub(crate) fn exe_phdrs() -> (*const libc::c_void, usize) {
    unsafe {
        (
            libc::getauxval(libc::AT_PHDR) as *const libc::c_void,
            libc::getauxval(libc::AT_PHNUM) as usize,
        )
    }
}

#[cfg(feature = "runtime")]
#[inline]
pub(in super::super) fn exe_phdrs_slice() -> &'static [Elf_Phdr] {
    let (phdr, phnum) = exe_phdrs();

    // SAFETY: We assume the `AT_PHDR` and `AT_PHNUM` values provided by the
    // kernel form a valid slice.
    unsafe { slice::from_raw_parts(phdr.cast(), phnum) }
}

/// `AT_SYSINFO_EHDR` isn't present on all platforms in all configurations,
/// so if we don't see it, this function returns a null pointer.
#[inline]
pub(in super::super) fn sysinfo_ehdr() -> *const Elf_Ehdr {
    if let Some(libc_getauxval) = getauxval.get() {
        unsafe { libc_getauxval(linux_raw_sys::general::AT_SYSINFO_EHDR.into()) as *const Elf_Ehdr }
    } else {
        null()
    }
}
