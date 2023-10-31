//! TLS utilities.
//!
//! # Safety
//!
//! This file contains code that reads the raw phdr array pointed to by the
//! kernel-provided AUXV values.
#![allow(unsafe_code)]

use super::super::c;
use super::super::elf::*;
use super::super::param::auxv::exe_phdrs_slice;
use core::ptr::null;

/// For use with [`set_thread_area`].
///
/// [`set_thread_area`]: crate::runtime::set_thread_area
#[cfg(target_arch = "x86")]
pub type UserDesc = linux_raw_sys::general::user_desc;

pub(crate) fn startup_tls_info() -> StartupTlsInfo {
    let mut base = null();
    let mut tls_phdr = null();
    let mut stack_size = 0;

    let phdrs = exe_phdrs_slice();

    // SAFETY: We assume the phdr array pointer and length the kernel provided
    // to the process describe a valid phdr array.
    unsafe {
        for phdr in phdrs {
            match phdr.p_type {
                PT_PHDR => base = phdrs.as_ptr().cast::<u8>().sub(phdr.p_vaddr),
                PT_TLS => tls_phdr = phdr,
                PT_GNU_STACK => stack_size = phdr.p_memsz,
                _ => {}
            }
        }

        StartupTlsInfo {
            addr: base.cast::<u8>().add((*tls_phdr).p_vaddr).cast(),
            mem_size: (*tls_phdr).p_memsz,
            file_size: (*tls_phdr).p_filesz,
            align: (*tls_phdr).p_align,
            stack_size,
        }
    }
}

/// The values returned from [`startup_tls_info`].
///
/// [`startup_tls_info`]: crate::runtime::startup_tls_info
pub struct StartupTlsInfo {
    /// The base address of the TLS segment.
    pub addr: *const c::c_void,
    /// The size of the memory region.
    pub mem_size: usize,
    /// The size beyond which all memory is zero-initialized.
    pub file_size: usize,
    /// The required alignment for the TLS segment.
    pub align: usize,
    /// The requested minimum size for stacks.
    pub stack_size: usize,
}
