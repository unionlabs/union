//! Linux auxv support.
//!
//! # Safety
//!
//! This uses raw pointers to locate and read the kernel-provided auxv array.
#![allow(unsafe_code)]

use super::super::c;
use super::super::elf::{Elf_Ehdr, Elf_Phdr};
#[cfg(feature = "param")]
use crate::ffi::CStr;
use core::mem::size_of;
use core::ptr::null;
#[cfg(feature = "runtime")]
use core::slice;
use linux_raw_sys::general::{
    AT_CLKTCK, AT_EXECFN, AT_HWCAP, AT_HWCAP2, AT_NULL, AT_PAGESZ, AT_PHDR, AT_PHENT, AT_PHNUM,
    AT_SYSINFO_EHDR,
};

#[cfg(feature = "param")]
#[inline]
pub(crate) fn page_size() -> usize {
    auxv().page_size
}

#[cfg(feature = "param")]
#[inline]
pub(crate) fn clock_ticks_per_second() -> u64 {
    auxv().clock_ticks_per_second as u64
}

#[cfg(feature = "param")]
#[inline]
pub(crate) fn linux_hwcap() -> (usize, usize) {
    let auxv = auxv();
    (auxv.hwcap, auxv.hwcap2)
}

#[cfg(feature = "param")]
#[inline]
pub(crate) fn linux_execfn() -> &'static CStr {
    let execfn = auxv().execfn;

    // Safety: We assume the `AT_EXECFN` value provided by the kernel is a
    // valid pointer to a valid NUL-terminated array of bytes.
    unsafe { CStr::from_ptr(execfn.cast()) }
}

#[cfg(feature = "runtime")]
#[inline]
pub(crate) fn exe_phdrs() -> (*const c::c_void, usize) {
    let auxv = auxv();
    (auxv.phdr.cast(), auxv.phnum)
}

#[cfg(feature = "runtime")]
#[inline]
pub(in super::super) fn exe_phdrs_slice() -> &'static [Elf_Phdr] {
    let auxv = auxv();

    // Safety: We assume the `AT_PHDR` and `AT_PHNUM` values provided by the
    // kernel form a valid slice.
    unsafe { slice::from_raw_parts(auxv.phdr, auxv.phnum) }
}

#[inline]
pub(in super::super) fn sysinfo_ehdr() -> *const Elf_Ehdr {
    auxv().sysinfo_ehdr
}

#[inline]
fn auxv() -> &'static Auxv {
    // Safety: `AUXV` is initialized from the `.init_array`, and we never
    // mutate it thereafter, so it's effectively initialized read-only in all
    // other code.
    unsafe {
        // Assert that the initialization has happened. On glibc and musl, this
        // is handled automatically by `.init_array` functions. Otherwise,
        // `rustix::process::init` must be called explicitly.
        debug_assert_ne!(AUXV.page_size, 0);

        &AUXV
    }
}

/// A struct for holding fields obtained from the kernel-provided auxv array.
struct Auxv {
    page_size: usize,
    clock_ticks_per_second: usize,
    hwcap: usize,
    hwcap2: usize,
    sysinfo_ehdr: *const Elf_Ehdr,
    phdr: *const Elf_Phdr,
    phnum: usize,
    execfn: *const c::c_char,
}

/// Data obtained from the kernel-provided auxv array. This is initialized at
/// program startup below.
static mut AUXV: Auxv = Auxv {
    page_size: 0,
    clock_ticks_per_second: 0,
    hwcap: 0,
    hwcap2: 0,
    sysinfo_ehdr: null(),
    phdr: null(),
    phnum: 0,
    execfn: null(),
};

/// GLIBC passes argc, argv, and envp to functions in .init_array, as a
/// non-standard extension. Use priority 99 so that we run before any
/// normal user-defined constructor functions.
#[cfg(all(target_env = "gnu", not(target_vendor = "mustang")))]
#[used]
#[link_section = ".init_array.00099"]
static INIT_ARRAY: unsafe extern "C" fn(c::c_int, *mut *mut u8, *mut *mut u8) = {
    unsafe extern "C" fn function(_argc: c::c_int, _argv: *mut *mut u8, envp: *mut *mut u8) {
        init_from_envp(envp);
    }
    function
};

/// For musl, assume that `__environ` is available and points to the original
/// environment from the kernel, so we can find the auxv array in memory after
/// it. Use priority 99 so that we run before any normal user-defined
/// constructor functions.
///
/// <https://refspecs.linuxbase.org/LSB_5.0.0/LSB-Core-generic/LSB-Core-generic/baselib---environ.html>
#[cfg(all(target_env = "musl", not(target_vendor = "mustang")))]
#[used]
#[link_section = ".init_array.00099"]
static INIT_ARRAY: unsafe extern "C" fn() = {
    unsafe extern "C" fn function() {
        extern "C" {
            static __environ: *mut *mut u8;
        }

        init_from_envp(__environ)
    }
    function
};

/// On mustang or any non-musl non-glibic platform where we don't know that we
/// have `.init_array`, we export a function to be called during
/// initialization, and passed a pointer to the original environment variable
/// block set up by the OS.
#[cfg(any(
    target_vendor = "mustang",
    not(any(target_env = "gnu", target_env = "musl")),
))]
#[inline]
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

/// # Safety
///
/// This must be passed a pointer to the auxv array provided by the kernel.
unsafe fn init_from_auxp(mut auxp: *const Elf_auxv_t) {
    loop {
        let Elf_auxv_t { a_type, a_val } = *auxp;
        match a_type as _ {
            AT_PAGESZ => AUXV.page_size = a_val as usize,
            AT_CLKTCK => AUXV.clock_ticks_per_second = a_val as usize,
            AT_HWCAP => AUXV.hwcap = a_val as usize,
            AT_HWCAP2 => AUXV.hwcap2 = a_val as usize,
            AT_SYSINFO_EHDR => AUXV.sysinfo_ehdr = a_val.cast(),
            AT_PHDR => AUXV.phdr = a_val.cast(),
            AT_PHNUM => AUXV.phnum = a_val as usize,
            AT_PHENT => assert_eq!(a_val as usize, size_of::<Elf_Phdr>()),
            AT_EXECFN => AUXV.execfn = a_val.cast(),
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
    a_val: *const (),
}
