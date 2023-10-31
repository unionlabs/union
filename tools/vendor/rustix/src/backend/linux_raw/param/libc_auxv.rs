//! Linux auxv support, using libc.
//!
//! # Safety
//!
//! This uses raw pointers to locate and read the kernel-provided auxv array.
#![allow(unsafe_code)]

use crate::backend::c;
use crate::backend::elf::*;
#[cfg(feature = "param")]
use crate::ffi::CStr;
#[cfg(not(feature = "runtime"))]
use core::ptr::null;
#[cfg(feature = "runtime")]
use core::slice;

// `getauxval` wasn't supported in glibc until 2.16. Also this lets us use
// `*mut` as the return type to preserve strict provenance.
#[cfg(not(feature = "runtime"))]
weak!(fn getauxval(c::c_ulong) -> *mut c::c_void);

// With the "runtime" feature, go ahead and depend on `getauxval` existing
// so that we never fail.
#[cfg(feature = "runtime")]
extern "C" {
    fn getauxval(type_: c::c_ulong) -> *mut c::c_void;
}

const AT_PHDR: c::c_ulong = 3;
const AT_PHNUM: c::c_ulong = 5;
const AT_HWCAP: c::c_ulong = 16;
const AT_HWCAP2: c::c_ulong = 26;
const AT_EXECFN: c::c_ulong = 31;
const AT_SYSINFO_EHDR: c::c_ulong = 33;

// Declare `sysconf` ourselves so that we don't depend on all of libc
// just for this.
extern "C" {
    fn sysconf(name: c::c_int) -> c::c_long;
}

#[cfg(target_os = "android")]
const _SC_PAGESIZE: c::c_int = 39;
#[cfg(target_os = "emscripten")]
const _SC_PAGESIZE: c::c_int = 30;
#[cfg(target_os = "linux")]
const _SC_PAGESIZE: c::c_int = 30;
#[cfg(target_os = "android")]
const _SC_CLK_TCK: c::c_int = 6;
#[cfg(target_os = "emscripten")]
const _SC_CLK_TCK: c::c_int = 2;
#[cfg(target_os = "linux")]
const _SC_CLK_TCK: c::c_int = 2;

#[test]
fn test_abi() {
    assert_eq!(self::_SC_PAGESIZE, ::libc::_SC_PAGESIZE);
    assert_eq!(self::_SC_CLK_TCK, ::libc::_SC_CLK_TCK);
    assert_eq!(self::AT_PHDR, ::libc::AT_PHDR);
    assert_eq!(self::AT_PHNUM, ::libc::AT_PHNUM);
    assert_eq!(self::AT_HWCAP, ::libc::AT_HWCAP);
    assert_eq!(self::AT_HWCAP2, ::libc::AT_HWCAP2);
    assert_eq!(self::AT_EXECFN, ::libc::AT_EXECFN);
    assert_eq!(self::AT_SYSINFO_EHDR, ::libc::AT_SYSINFO_EHDR);
}

#[cfg(feature = "param")]
#[inline]
pub(crate) fn page_size() -> usize {
    unsafe { sysconf(_SC_PAGESIZE) as usize }
}

#[cfg(feature = "param")]
#[inline]
pub(crate) fn clock_ticks_per_second() -> u64 {
    unsafe { sysconf(_SC_CLK_TCK) as u64 }
}

#[cfg(feature = "param")]
#[inline]
pub(crate) fn linux_hwcap() -> (usize, usize) {
    #[cfg(not(feature = "runtime"))]
    unsafe {
        if let Some(libc_getauxval) = getauxval.get() {
            let hwcap = libc_getauxval(AT_HWCAP) as usize;
            let hwcap2 = libc_getauxval(AT_HWCAP2) as usize;
            (hwcap, hwcap2)
        } else {
            (0, 0)
        }
    }

    #[cfg(feature = "runtime")]
    unsafe {
        let hwcap = getauxval(AT_HWCAP) as usize;
        let hwcap2 = getauxval(AT_HWCAP2) as usize;
        (hwcap, hwcap2)
    }
}

#[cfg(feature = "param")]
#[inline]
pub(crate) fn linux_execfn() -> &'static CStr {
    #[cfg(not(feature = "runtime"))]
    unsafe {
        if let Some(libc_getauxval) = getauxval.get() {
            CStr::from_ptr(libc_getauxval(AT_EXECFN).cast())
        } else {
            cstr!("")
        }
    }

    #[cfg(feature = "runtime")]
    unsafe {
        CStr::from_ptr(getauxval(AT_EXECFN).cast())
    }
}

#[cfg(feature = "runtime")]
#[inline]
pub(crate) fn exe_phdrs() -> (*const c::c_void, usize) {
    unsafe {
        let phdr = getauxval(AT_PHDR) as *const c::c_void;
        let phnum = getauxval(AT_PHNUM) as usize;
        (phdr, phnum)
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
    #[cfg(not(feature = "runtime"))]
    unsafe {
        if let Some(libc_getauxval) = getauxval.get() {
            libc_getauxval(AT_SYSINFO_EHDR) as *const Elf_Ehdr
        } else {
            null()
        }
    }

    #[cfg(feature = "runtime")]
    unsafe {
        getauxval(AT_SYSINFO_EHDR) as *const Elf_Ehdr
    }
}
