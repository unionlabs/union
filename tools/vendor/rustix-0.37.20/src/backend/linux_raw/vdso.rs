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
use crate::ffi::CStr;
use crate::utils::check_raw_pointer;
use core::ffi::c_void;
use core::mem::size_of;
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

/// Create a `Vdso` value by parsing the vDSO at the `sysinfo_ehdr` address.
fn init_from_sysinfo_ehdr() -> Option<Vdso> {
    // SAFETY: the auxv initialization code does extensive checks to ensure
    // that the value we get really is an `AT_SYSINFO_EHDR` value from the
    // kernel.
    unsafe {
        let hdr = super::param::auxv::sysinfo_ehdr();

        // If the platform doesn't provide a `AT_SYSINFO_EHDR`, we can't locate
        // the vDSO.
        if hdr.is_null() {
            return None;
        }

        let mut vdso = Vdso {
            load_addr: hdr,
            load_end: hdr.cast(),
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

        let hdr = &*hdr;
        let pt = check_raw_pointer::<Elf_Phdr>(vdso.base_plus(hdr.e_phoff)? as *mut _)?.as_ptr();
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

                dyn_ = check_raw_pointer::<Elf_Dyn>(vdso.base_plus(phdr.p_offset)? as *mut _)?
                    .as_ptr();
                num_dyn = phdr.p_memsz / size_of::<Elf_Dyn>();
            } else if phdr.p_type == PT_INTERP || phdr.p_type == PT_GNU_RELRO {
                // Don't trust any ELF image that has an “interpreter” or that uses
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
                    vdso.symstrings =
                        check_raw_pointer::<u8>(vdso.addr_from_elf(d.d_val)? as *mut _)?.as_ptr();
                }
                DT_SYMTAB => {
                    vdso.symtab =
                        check_raw_pointer::<Elf_Sym>(vdso.addr_from_elf(d.d_val)? as *mut _)?
                            .as_ptr();
                }
                DT_HASH => {
                    hash =
                        check_raw_pointer::<u32>(vdso.addr_from_elf(d.d_val)? as *mut _)?.as_ptr();
                }
                DT_VERSYM => {
                    vdso.versym =
                        check_raw_pointer::<u16>(vdso.addr_from_elf(d.d_val)? as *mut _)?.as_ptr();
                }
                DT_VERDEF => {
                    vdso.verdef =
                        check_raw_pointer::<Elf_Verdef>(vdso.addr_from_elf(d.d_val)? as *mut _)?
                            .as_ptr();
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
        // The upstream code checks `symstrings`, `symtab`, and `hash` for null;
        // here, `check_raw_pointer` has already done that.

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
}

impl Vdso {
    /// Parse the vDSO.
    ///
    /// Returns `None` if the vDSO can't be located or if it doesn't conform
    /// to our expectations.
    #[inline]
    pub(super) fn new() -> Option<Self> {
        init_from_sysinfo_ehdr()
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

        // SAFETY: The pointers in `self` must be valid.
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
