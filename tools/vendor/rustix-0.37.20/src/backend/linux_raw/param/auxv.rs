//! Linux auxv support.
//!
//! # Safety
//!
//! This uses raw pointers to locate and read the kernel-provided auxv array.
#![allow(unsafe_code)]

use super::super::c;
use super::super::elf::*;
use crate::fd::OwnedFd;
#[cfg(feature = "param")]
use crate::ffi::CStr;
#[cfg(not(target_vendor = "mustang"))]
use crate::fs::{Mode, OFlags};
use crate::utils::{as_ptr, check_raw_pointer};
use alloc::vec::Vec;
use core::ffi::c_void;
use core::mem::size_of;
use core::ptr::{null_mut, read_unaligned, NonNull};
#[cfg(feature = "runtime")]
use core::slice;
use core::sync::atomic::Ordering::Relaxed;
use core::sync::atomic::{AtomicPtr, AtomicUsize};
use linux_raw_sys::general::{
    AT_BASE, AT_CLKTCK, AT_EXECFN, AT_HWCAP, AT_HWCAP2, AT_NULL, AT_PAGESZ, AT_PHDR, AT_PHENT,
    AT_PHNUM, AT_SYSINFO_EHDR,
};

#[cfg(feature = "param")]
#[inline]
pub(crate) fn page_size() -> usize {
    let mut page_size = PAGE_SIZE.load(Relaxed);

    if page_size == 0 {
        init_from_proc_self_auxv();
        page_size = PAGE_SIZE.load(Relaxed);
    }

    page_size
}

#[cfg(feature = "param")]
#[inline]
pub(crate) fn clock_ticks_per_second() -> u64 {
    let mut ticks = CLOCK_TICKS_PER_SECOND.load(Relaxed);

    if ticks == 0 {
        init_from_proc_self_auxv();
        ticks = CLOCK_TICKS_PER_SECOND.load(Relaxed);
    }

    ticks as u64
}

#[cfg(feature = "param")]
#[inline]
pub(crate) fn linux_hwcap() -> (usize, usize) {
    let mut hwcap = HWCAP.load(Relaxed);
    let mut hwcap2 = HWCAP2.load(Relaxed);

    if hwcap == 0 || hwcap2 == 0 {
        init_from_proc_self_auxv();
        hwcap = HWCAP.load(Relaxed);
        hwcap2 = HWCAP2.load(Relaxed);
    }

    (hwcap, hwcap2)
}

#[cfg(feature = "param")]
#[inline]
pub(crate) fn linux_execfn() -> &'static CStr {
    let mut execfn = EXECFN.load(Relaxed);

    if execfn.is_null() {
        init_from_proc_self_auxv();
        execfn = EXECFN.load(Relaxed);
    }

    // SAFETY: We assume the `AT_EXECFN` value provided by the kernel is a
    // valid pointer to a valid NUL-terminated array of bytes.
    unsafe { CStr::from_ptr(execfn.cast()) }
}

#[cfg(feature = "runtime")]
#[inline]
pub(crate) fn exe_phdrs() -> (*const c::c_void, usize) {
    let mut phdr = PHDR.load(Relaxed);
    let mut phnum = PHNUM.load(Relaxed);

    if phdr.is_null() || phnum == 0 {
        init_from_proc_self_auxv();
        phdr = PHDR.load(Relaxed);
        phnum = PHNUM.load(Relaxed);
    }

    (phdr.cast(), phnum)
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
    let mut ehdr = SYSINFO_EHDR.load(Relaxed);

    if ehdr.is_null() {
        init_from_proc_self_auxv();
        ehdr = SYSINFO_EHDR.load(Relaxed);
    }

    ehdr
}

static PAGE_SIZE: AtomicUsize = AtomicUsize::new(0);
static CLOCK_TICKS_PER_SECOND: AtomicUsize = AtomicUsize::new(0);
static HWCAP: AtomicUsize = AtomicUsize::new(0);
static HWCAP2: AtomicUsize = AtomicUsize::new(0);
static SYSINFO_EHDR: AtomicPtr<Elf_Ehdr> = AtomicPtr::new(null_mut());
static PHDR: AtomicPtr<Elf_Phdr> = AtomicPtr::new(null_mut());
static PHNUM: AtomicUsize = AtomicUsize::new(0);
static EXECFN: AtomicPtr<c::c_char> = AtomicPtr::new(null_mut());

/// On non-Mustang platforms, we read the aux vector from /proc/self/auxv.
#[cfg(not(target_vendor = "mustang"))]
fn init_from_proc_self_auxv() {
    // Open "/proc/self/auxv", either because we trust "/proc", or because
    // we're running inside QEMU and `proc_self_auxv`'s extra checking foils
    // QEMU's emulation so we need to do a plain open to get the right
    // auxv records.
    let file = crate::fs::openat(
        crate::fs::cwd(),
        "/proc/self/auxv",
        OFlags::RDONLY,
        Mode::empty(),
    )
    .unwrap();

    let _ = init_from_auxv_file(file);
}

#[cfg(target_vendor = "mustang")]
fn init_from_proc_self_auxv() {
    panic!("mustang should have initialized the auxv values");
}

/// Process auxv entries from the open file `auxv`.
fn init_from_auxv_file(auxv: OwnedFd) -> Option<()> {
    let mut buffer = Vec::<u8>::with_capacity(512);
    loop {
        let cur = buffer.len();

        // Request one extra byte; `Vec` will often allocate more.
        buffer.reserve(1);

        // Use all the space it allocated.
        buffer.resize(buffer.capacity(), 0);

        // Read up to that many bytes.
        let n = match crate::io::read(&auxv, &mut buffer[cur..]) {
            Err(crate::io::Errno::INTR) => 0,
            Err(_err) => panic!(),
            Ok(0) => break,
            Ok(n) => n,
        };

        // Account for the number of bytes actually read.
        buffer.resize(cur + n, 0_u8);
    }

    // SAFETY: We loaded from an auxv file into the buffer.
    unsafe { init_from_auxp(buffer.as_ptr().cast()) }
}

/// Process auxv entries from the auxv array pointed to by `auxp`.
///
/// # Safety
///
/// This must be passed a pointer to an auxv array.
///
/// The buffer contains `Elf_aux_t` elements, though it need not be aligned;
/// function uses `read_unaligned` to read from it.
unsafe fn init_from_auxp(mut auxp: *const Elf_auxv_t) -> Option<()> {
    let mut pagesz = 0;
    let mut clktck = 0;
    let mut hwcap = 0;
    let mut hwcap2 = 0;
    let mut phdr = null_mut();
    let mut phnum = 0;
    let mut execfn = null_mut();
    let mut sysinfo_ehdr = null_mut();
    let mut phent = 0;

    loop {
        let Elf_auxv_t { a_type, a_val } = read_unaligned(auxp);

        match a_type as _ {
            AT_PAGESZ => pagesz = a_val as usize,
            AT_CLKTCK => clktck = a_val as usize,
            AT_HWCAP => hwcap = a_val as usize,
            AT_HWCAP2 => hwcap2 = a_val as usize,
            AT_PHDR => phdr = check_raw_pointer::<Elf_Phdr>(a_val as *mut _)?.as_ptr(),
            AT_PHNUM => phnum = a_val as usize,
            AT_PHENT => phent = a_val as usize,
            AT_EXECFN => execfn = check_raw_pointer::<c::c_char>(a_val as *mut _)?.as_ptr(),
            AT_BASE => check_interpreter_base(a_val.cast())?,
            AT_SYSINFO_EHDR => sysinfo_ehdr = check_vdso_base(a_val as *mut _)?.as_ptr(),
            AT_NULL => break,
            _ => (),
        }
        auxp = auxp.add(1);
    }

    assert_eq!(phent, size_of::<Elf_Phdr>());

    // The base and sysinfo_ehdr (if present) matches our platform. Accept
    // the aux values.
    PAGE_SIZE.store(pagesz, Relaxed);
    CLOCK_TICKS_PER_SECOND.store(clktck, Relaxed);
    HWCAP.store(hwcap, Relaxed);
    HWCAP2.store(hwcap2, Relaxed);
    PHDR.store(phdr, Relaxed);
    PHNUM.store(phnum, Relaxed);
    EXECFN.store(execfn, Relaxed);
    SYSINFO_EHDR.store(sysinfo_ehdr, Relaxed);

    Some(())
}

/// Check that `base` is a valid pointer to the program interpreter.
///
/// `base` is some value we got from a `AT_BASE` aux record somewhere,
/// which hopefully holds the value of the program interpreter in memory. Do a
/// series of checks to be as sure as we can that it's safe to use.
unsafe fn check_interpreter_base(base: *const Elf_Ehdr) -> Option<()> {
    check_elf_base(base)?;
    Some(())
}

/// Check that `base` is a valid pointer to the kernel-provided vDSO.
///
/// `base` is some value we got from a `AT_SYSINFO_EHDR` aux record somewhere,
/// which hopefully holds the value of the kernel-provided vDSO in memory. Do a
/// series of checks to be as sure as we can that it's safe to use.
unsafe fn check_vdso_base(base: *const Elf_Ehdr) -> Option<NonNull<Elf_Ehdr>> {
    // In theory, we could check that we're not attempting to parse our own ELF
    // image, as an additional check. However, older Linux toolchains don't
    // support this, and Rust's `#[linkage = "extern_weak"]` isn't stable yet,
    // so just disable this for now.
    /*
    {
        extern "C" {
            static __ehdr_start: c::c_void;
        }

        let ehdr_start: *const c::c_void = &__ehdr_start;
        if base == ehdr_start {
            return None;
        }
    }
    */

    let hdr = check_elf_base(base)?;

    // Check that the ELF is not writable, since that would indicate that this
    // isn't the ELF we think it is. Here we're just using `clock_getres` just
    // as an arbitrary system call which writes to a buffer and fails with
    // `EFAULT` if the buffer is not writable.
    {
        use super::super::conv::{c_uint, ret};
        if ret(syscall!(
            __NR_clock_getres,
            c_uint(linux_raw_sys::general::CLOCK_MONOTONIC),
            base
        )) != Err(crate::io::Errno::FAULT)
        {
            // We can't gracefully fail here because we would seem to have just
            // mutated some unknown memory.
            #[cfg(feature = "std")]
            {
                std::process::abort();
            }
            #[cfg(all(not(feature = "std"), feature = "rustc-dep-of-std"))]
            {
                core::intrinsics::abort();
            }
        }
    }

    Some(hdr)
}

/// Check that `base` is a valid pointer to an ELF image.
unsafe fn check_elf_base(base: *const Elf_Ehdr) -> Option<NonNull<Elf_Ehdr>> {
    // If we're reading a 64-bit auxv on a 32-bit platform, we'll see
    // a zero `a_val` because `AT_*` values are never greater than
    // `u32::MAX`. Zero is used by libc's `getauxval` to indicate
    // errors, so it should never be a valid value.
    if base.is_null() {
        return None;
    }

    let hdr = match check_raw_pointer::<Elf_Ehdr>(base as *mut _) {
        Some(hdr) => hdr,
        None => return None,
    };

    let hdr = hdr.as_ref();
    if hdr.e_ident[..SELFMAG] != ELFMAG {
        return None; // Wrong ELF magic
    }
    if !matches!(hdr.e_ident[EI_OSABI], ELFOSABI_SYSV | ELFOSABI_LINUX) {
        return None; // Unrecognized ELF OS ABI
    }
    if hdr.e_ident[EI_ABIVERSION] != ELFABIVERSION {
        return None; // Unrecognized ELF ABI version
    }
    if hdr.e_type != ET_DYN {
        return None; // Wrong ELF type
    }

    // If ELF is extended, we'll need to adjust.
    if hdr.e_ident[EI_VERSION] != EV_CURRENT
        || hdr.e_ehsize as usize != size_of::<Elf_Ehdr>()
        || hdr.e_phentsize as usize != size_of::<Elf_Phdr>()
    {
        return None;
    }
    // We don't currently support extra-large numbers of segments.
    if hdr.e_phnum == PN_XNUM {
        return None;
    }

    // If `e_phoff` is zero, it's more likely that we're looking at memory that
    // has been zeroed than that the kernel has somehow aliased the `Ehdr` and
    // the `Phdr`.
    if hdr.e_phoff < size_of::<Elf_Ehdr>() {
        return None;
    }

    // Verify that the `EI_CLASS`/`EI_DATA`/`e_machine` fields match the
    // architecture we're running as. This helps catch cases where we're
    // running under QEMU.
    if hdr.e_ident[EI_CLASS] != ELFCLASS {
        return None; // Wrong ELF class
    }
    if hdr.e_ident[EI_DATA] != ELFDATA {
        return None; // Wrong ELF data
    }
    if hdr.e_machine != EM_CURRENT {
        return None; // Wrong machine type
    }

    Some(NonNull::new_unchecked(as_ptr(hdr) as *mut _))
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
