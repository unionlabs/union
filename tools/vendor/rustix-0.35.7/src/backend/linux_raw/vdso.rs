//! Parse the Linux vDSO.
//!
//! The following code is transliterated from
//! tools/testing/selftests/vDSO/parse_vdso.c in Linux 5.11, which is licensed
//! with Creative Commons Zero License, version 1.0,
//! available at <https://creativecommons.org/publicdomain/zero/1.0/legalcode>
//!
//! # Safety
//!
//! Parsing the vDSO involves a lot of raw pointer manipulation. This
//! implementation follows Linux's reference implementation, and adds several
//! additional safety checks.
#![allow(unsafe_code)]

use super::c;
use super::elf::*;
use super::mm::syscalls::madvise;
use super::mm::types::Advice;
use crate::ffi::CStr;
use crate::io;
use core::ffi::c_void;
use core::mem::{align_of, size_of};
use core::ptr::{null, null_mut};

pub(super) struct Vdso {
    // Load information
    load_addr: *const Elf_Ehdr,
    load_end: *const c_void, // the end of the `PT_LOAD` segment
    pv_offset: usize,        // recorded paddr - recorded vaddr

    // Symbol table
    symtab: *const Elf_Sym,
    symstrings: *const u8,
    bucket: *const u32,
    chain: *const u32,
    nbucket: u32,
    //nchain: u32,

    // Version table
    versym: *const u16,
    verdef: *const Elf_Verdef,
}

// Straight from the ELF specification.
fn elf_hash(name: &CStr) -> u32 {
    let mut h: u32 = 0;
    for b in name.to_bytes() {
        h = (h << 4).wrapping_add(u32::from(*b));
        let g = h & 0xf000_0000;
        if g != 0 {
            h ^= g >> 24;
        }
        h &= !g;
    }
    h
}

/// Cast `value` to a pointer type, doing some checks for validity.
fn make_pointer<T>(value: *const c_void) -> Option<*const T> {
    if value.is_null()
        || (value as usize).checked_add(size_of::<T>()).is_none()
        || (value as usize) % align_of::<T>() != 0
    {
        return None;
    }

    Some(value.cast())
}

/// Create a `Vdso` value by parsing the vDSO at the given address.
///
/// # Safety
///
/// `base` must be a valid pointer to an ELF image in memory.
unsafe fn init_from_sysinfo_ehdr(base: *const Elf_Ehdr) -> Option<Vdso> {
    // Check that `base` is a valid pointer to the kernel-provided vDSO.
    let hdr = check_vdso_base(base)?;

    let mut vdso = Vdso {
        load_addr: base,
        load_end: base.cast(),
        pv_offset: 0,
        symtab: null(),
        symstrings: null(),
        bucket: null(),
        chain: null(),
        nbucket: 0,
        //nchain: 0,
        versym: null(),
        verdef: null(),
    };

    let pt = make_pointer::<Elf_Phdr>(vdso.base_plus(hdr.e_phoff)?)?;
    let mut dyn_: *const Elf_Dyn = null();
    let mut num_dyn = 0;

    // We need two things from the segment table: the load offset
    // and the dynamic table.
    let mut found_vaddr = false;
    for i in 0..hdr.e_phnum {
        let phdr = &*pt.add(i as usize);
        if phdr.p_flags & PF_W != 0 {
            // Don't trust any vDSO that claims to be loading writable
            // segments into memory.
            return None;
        }
        if phdr.p_type == PT_LOAD && !found_vaddr {
            // The segment should be readable and executable, because it
            // contains the symbol table and the function bodies.
            if phdr.p_flags & (PF_R | PF_X) != (PF_R | PF_X) {
                return None;
            }
            found_vaddr = true;
            vdso.load_end = vdso.base_plus(phdr.p_offset.checked_add(phdr.p_memsz)?)?;
            vdso.pv_offset = phdr.p_offset.wrapping_sub(phdr.p_vaddr);
        } else if phdr.p_type == PT_DYNAMIC {
            // If `p_offset` is zero, it's more likely that we're looking at memory
            // that has been zeroed than that the kernel has somehow aliased the
            // `Ehdr` and the `Elf_Dyn` array.
            if phdr.p_offset < size_of::<Elf_Ehdr>() {
                return None;
            }

            dyn_ = make_pointer::<Elf_Dyn>(vdso.base_plus(phdr.p_offset)?)?;
            num_dyn = phdr.p_memsz / size_of::<Elf_Dyn>();
        } else if phdr.p_type == PT_INTERP || phdr.p_type == PT_GNU_RELRO {
            // Don't trust any ELF image that has an "interpreter" or that uses
            // RELRO, which is likely to be a user ELF image rather and not the
            // kernel vDSO.
            return None;
        }
    }

    if !found_vaddr || dyn_.is_null() {
        return None; // Failed
    }

    // Fish out the useful bits of the dynamic table.
    let mut hash: *const u32 = null();
    vdso.symstrings = null();
    vdso.symtab = null();
    vdso.versym = null();
    vdso.verdef = null();
    let mut i = 0;
    loop {
        if i == num_dyn {
            return None;
        }
        let d = &*dyn_.add(i);
        match d.d_tag {
            DT_STRTAB => {
                vdso.symstrings = make_pointer::<u8>(vdso.addr_from_elf(d.d_val)?)?;
            }
            DT_SYMTAB => {
                vdso.symtab = make_pointer::<Elf_Sym>(vdso.addr_from_elf(d.d_val)?)?;
            }
            DT_HASH => {
                hash = make_pointer::<u32>(vdso.addr_from_elf(d.d_val)?)?;
            }
            DT_VERSYM => {
                vdso.versym = make_pointer::<u16>(vdso.addr_from_elf(d.d_val)?)?;
            }
            DT_VERDEF => {
                vdso.verdef = make_pointer::<Elf_Verdef>(vdso.addr_from_elf(d.d_val)?)?;
            }
            DT_SYMENT => {
                if d.d_val != size_of::<Elf_Sym>() {
                    return None; // Failed
                }
            }
            DT_NULL => break,
            _ => {}
        }
        i = i.checked_add(1)?;
    }
    if vdso.symstrings.is_null() || vdso.symtab.is_null() || hash.is_null() {
        return None; // Failed
    }

    if vdso.verdef.is_null() {
        vdso.versym = null();
    }

    // Parse the hash table header.
    vdso.nbucket = *hash.add(0);
    //vdso.nchain = *hash.add(1);
    vdso.bucket = hash.add(2);
    vdso.chain = hash.add(vdso.nbucket as usize + 2);

    // That's all we need.
    Some(vdso)
}

/// Check that `base` is a valid pointer to the kernel-provided vDSO.
///
/// `base` is some value we got from a `AT_SYSINFO_EHDR` aux record somewhere,
/// which hopefully holds the value of the kernel-provided vDSO in memory. Do a
/// series of checks to be as sure as we can that it's safe to use.
unsafe fn check_vdso_base<'vdso>(base: *const Elf_Ehdr) -> Option<&'vdso Elf_Ehdr> {
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

    let hdr = &*make_pointer::<Elf_Ehdr>(base.cast())?;

    // Check that the vDSO is page-aligned and appropriately mapped. We call
    // this after `make_pointer` so that we don't do a syscall if there's no
    // chance the pointer is valid.
    madvise(base as *mut c_void, size_of::<Elf_Ehdr>(), Advice::Normal).ok()?;

    if hdr.e_ident[..SELFMAG] != ELFMAG {
        return None; // Wrong ELF magic
    }
    if hdr.e_ident[EI_CLASS] != ELFCLASS {
        return None; // Wrong ELF class
    }
    if hdr.e_ident[EI_DATA] != ELFDATA {
        return None; // Wrong ELF data
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
    // Verify that the `e_machine` matches the architecture we're running as.
    // This helps catch cases where we're running under qemu.
    if hdr.e_machine != EM_CURRENT {
        return None; // Wrong machine type
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

    // Check that the vDSO is not writable, since that would indicate that this
    // isn't the kernel vDSO. Here we're just using `clock_getres` just as an
    // arbitrary system call which writes to a buffer and fails with `EFAULT`
    // if the buffer is not writable.
    {
        use super::conv::ret;
        use super::time::types::ClockId;
        if ret(syscall!(__NR_clock_getres, ClockId::Monotonic, base)) != Err(io::Errno::FAULT) {
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

impl Vdso {
    /// Parse the vDSO.
    ///
    /// Returns None if the vDSO can't be located or if it doesn't conform
    /// to our expectations.
    #[inline]
    pub(super) fn new() -> Option<Self> {
        init_from_auxv()
    }

    /// Check the version for a symbol.
    ///
    /// # Safety
    ///
    /// The raw pointers inside `self` must be valid.
    unsafe fn match_version(&self, mut ver: u16, name: &CStr, hash: u32) -> bool {
        // This is a helper function to check if the version indexed by
        // ver matches name (which hashes to hash).
        //
        // The version definition table is a mess, and I don't know how
        // to do this in better than linear time without allocating memory
        // to build an index. I also don't know why the table has
        // variable size entries in the first place.
        //
        // For added fun, I can't find a comprehensible specification of how
        // to parse all the weird flags in the table.
        //
        // So I just parse the whole table every time.

        // First step: find the version definition
        ver &= 0x7fff; // Apparently bit 15 means "hidden"
        let mut def = self.verdef;
        loop {
            if (*def).vd_version != VER_DEF_CURRENT {
                return false; // Failed
            }

            if ((*def).vd_flags & VER_FLG_BASE) == 0 && ((*def).vd_ndx & 0x7fff) == ver {
                break;
            }

            if (*def).vd_next == 0 {
                return false; // No definition.
            }

            def = def
                .cast::<u8>()
                .add((*def).vd_next as usize)
                .cast::<Elf_Verdef>();
        }

        // Now figure out whether it matches.
        let aux = &*(def.cast::<u8>())
            .add((*def).vd_aux as usize)
            .cast::<Elf_Verdaux>();
        (*def).vd_hash == hash
            && (name == CStr::from_ptr(self.symstrings.add(aux.vda_name as usize).cast()))
    }

    /// Look up a symbol in the vDSO.
    pub(super) fn sym(&self, version: &CStr, name: &CStr) -> *mut c::c_void {
        let ver_hash = elf_hash(version);
        let name_hash = elf_hash(name);

        // Safety: The pointers in `self` must be valid.
        unsafe {
            let mut chain = *self.bucket.add((name_hash % self.nbucket) as usize);

            while chain != STN_UNDEF {
                let sym = &*self.symtab.add(chain as usize);

                // Check for a defined global or weak function w/ right name.
                //
                // The reference parser in Linux's parse_vdso.c requires
                // symbols to have type `STT_FUNC`, but on powerpc64, the vDSO
                // uses `STT_NOTYPE`, so allow that too.
                if (ELF_ST_TYPE(sym.st_info) != STT_FUNC &&
                        ELF_ST_TYPE(sym.st_info) != STT_NOTYPE)
                    || (ELF_ST_BIND(sym.st_info) != STB_GLOBAL
                        && ELF_ST_BIND(sym.st_info) != STB_WEAK)
                    || sym.st_shndx == SHN_UNDEF
                    || sym.st_shndx == SHN_ABS
                    || ELF_ST_VISIBILITY(sym.st_other) != STV_DEFAULT
                    || (name != CStr::from_ptr(self.symstrings.add(sym.st_name as usize).cast()))
                    // Check symbol version.
                    || (!self.versym.is_null()
                        && !self.match_version(*self.versym.add(chain as usize), version, ver_hash))
                {
                    chain = *self.chain.add(chain as usize);
                    continue;
                }

                let sum = self.addr_from_elf(sym.st_value).unwrap();
                assert!(
                    sum as usize >= self.load_addr as usize
                        && sum as usize <= self.load_end as usize
                );
                return sum as *mut c::c_void;
            }
        }

        null_mut()
    }

    /// Add the given address to the vDSO base address.
    unsafe fn base_plus(&self, offset: usize) -> Option<*const c_void> {
        // Check for overflow.
        let _ = (self.load_addr as usize).checked_add(offset)?;
        // Add the offset to the base.
        Some(self.load_addr.cast::<u8>().add(offset).cast())
    }

    /// Translate an ELF-address-space address into a usable virtual address.
    unsafe fn addr_from_elf(&self, elf_addr: usize) -> Option<*const c_void> {
        self.base_plus(elf_addr.wrapping_add(self.pv_offset))
    }
}

// Find the `AT_SYSINFO_EHDR` in auxv records in memory. We have our own code
// for reading the auxv records in memory, so we don't currently use this.
//
// # Safety
//
// `elf_auxv` must point to a valid array of AUXV records terminated by an
// `AT_NULL` record.
/*
unsafe fn init_from_auxv(elf_auxv: *const Elf_auxv_t) -> Option<Vdso> {
    let mut i = 0;
    while (*elf_auxv.add(i)).a_type != AT_NULL {
        if (*elf_auxv.add(i)).a_type == AT_SYSINFO_EHDR {
            return init_from_sysinfo_ehdr((*elf_auxv.add(i)).a_val);
        }
        i += 1;
    }

    None
}
*/

/// Find the vDSO image by following the `AT_SYSINFO_EHDR` auxv record pointer.
fn init_from_auxv() -> Option<Vdso> {
    // Safety: `sysinfo_ehdr` does extensive checks to ensure that the value
    // we get really is an `AT_SYSINFO_EHDR` value from the kernel.
    unsafe { init_from_sysinfo_ehdr(super::param::auxv::sysinfo_ehdr()) }
}
