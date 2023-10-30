//! Linux auxv support, for Mustang.
//!
//! # Safety
//!
//! This uses raw pointers to locate and read the kernel-provided auxv array.
#![allow(unsafe_code)]

use super::super::c;
use super::super::elf::*;
#[cfg(feature = "param")]
use crate::ffi::CStr;
use core::ffi::c_void;
use core::mem::size_of;
use core::ptr::{null, read};
#[cfg(feature = "runtime")]
use core::slice;
use linux_raw_sys::general::{
    AT_CLKTCK, AT_EXECFN, AT_HWCAP, AT_HWCAP2, AT_NULL, AT_PAGESZ, AT_PHDR, AT_PHENT, AT_PHNUM,
    AT_SYSINFO_EHDR,
};

#[cfg(feature = "param")]
#[inline]
pub(crate) fn page_size() -> usize {
    // SAFETY: This is initialized during program startup.
    unsafe { PAGE_SIZE }
}

#[cfg(feature = "param")]
#[inline]
pub(crate) fn clock_ticks_per_second() -> u64 {
    // SAFETY: This is initialized during program startup.
    unsafe { CLOCK_TICKS_PER_SECOND as u64 }
}

#[cfg(feature = "param")]
#[inline]
pub(crate) fn linux_hwcap() -> (usize, usize) {
    // SAFETY: This is initialized during program startup.
    unsafe { (HWCAP, HWCAP2) }
}

#[cfg(feature = "param")]
#[inline]
pub(crate) fn linux_execfn() -> &'static CStr {
    // SAFETY: This is initialized during program startup. And we
    // assume it's a valid pointer to a NUL-terminated string.
    unsafe { CStr::from_ptr(EXECFN.0.cast()) }
}

#[cfg(feature = "runtime")]
#[inline]
pub(crate) fn exe_phdrs() -> (*const c_void, usize) {
    // SAFETY: This is initialized during program startup.
    unsafe { (PHDR.0.cast(), PHNUM) }
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
    // SAFETY: This is initialized during program startup.
    unsafe { SYSINFO_EHDR.0 }
}

/// A const pointer to `T` that implements [`Sync`].
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SyncConstPtr<T>(*const T);
unsafe impl<T> Sync for SyncConstPtr<T> {}

impl<T> SyncConstPtr<T> {
    /// Creates a `SyncConstPointer` from a raw pointer.
    ///
    /// Behavior is undefined if `ptr` is actually not
    /// safe to share across threads.
    pub const unsafe fn new(ptr: *const T) -> Self {
        Self(ptr)
    }
}

static mut PAGE_SIZE: usize = 0;
static mut CLOCK_TICKS_PER_SECOND: usize = 0;
static mut HWCAP: usize = 0;
static mut HWCAP2: usize = 0;
static mut SYSINFO_EHDR: SyncConstPtr<Elf_Ehdr> = unsafe { SyncConstPtr::new(null()) };
static mut PHDR: SyncConstPtr<Elf_Phdr> = unsafe { SyncConstPtr::new(null()) };
static mut PHNUM: usize = 0;
static mut EXECFN: SyncConstPtr<c::c_char> = unsafe { SyncConstPtr::new(null()) };

/// On mustang, we export a function to be called during initialization, and
/// passed a pointer to the original environment variable block set up by the
/// OS.
pub(crate) unsafe fn init(envp: *mut *mut u8) {
    init_from_envp(envp);
}

/// # Safety
///
/// This must be passed a pointer to the environment variable buffer
/// provided by the kernel, which is followed in memory by the auxv array.
unsafe fn init_from_envp(mut envp: *mut *mut u8) {
    while !(*envp).is_null() {
        envp = envp.add(1);
    }
    init_from_auxp(envp.add(1).cast())
}

/// Process auxv entries from the auxv array pointed to by `auxp`.
///
/// # Safety
///
/// This must be passed a pointer to an auxv array.
///
/// The buffer contains `Elf_aux_t` elements, though it need not be aligned;
/// function uses `read_unaligned` to read from it.
unsafe fn init_from_auxp(mut auxp: *const Elf_auxv_t) {
    loop {
        let Elf_auxv_t { a_type, a_val } = read(auxp);

        match a_type as _ {
            AT_PAGESZ => PAGE_SIZE = a_val as usize,
            AT_CLKTCK => CLOCK_TICKS_PER_SECOND = a_val as usize,
            AT_HWCAP => HWCAP = a_val as usize,
            AT_HWCAP2 => HWCAP2 = a_val as usize,
            AT_PHDR => PHDR = SyncConstPtr::new(a_val.cast::<Elf_Phdr>()),
            AT_PHNUM => PHNUM = a_val as usize,
            AT_PHENT => assert_eq!(a_val as usize, size_of::<Elf_Phdr>()),
            AT_EXECFN => EXECFN = SyncConstPtr::new(a_val.cast::<c::c_char>()),
            AT_SYSINFO_EHDR => SYSINFO_EHDR = SyncConstPtr::new(a_val.cast::<Elf_Ehdr>()),
            AT_NULL => break,
            _ => (),
        }
        auxp = auxp.add(1);
    }
}

// ELF ABI

#[repr(C)]
#[derive(Copy, Clone)]
struct Elf_auxv_t {
    a_type: usize,

    // Some of the values in the auxv array are pointers, so we make `a_val` a
    // pointer, in order to preserve their provenance. For the values which are
    // integers, we cast this to `usize`.
    a_val: *const c_void,
}
