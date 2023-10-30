//! The ELF ABI. 🧝

#![allow(non_snake_case)]
#![cfg_attr(
    all(not(target_vendor = "mustang"), feature = "use-libc-auxv"),
    allow(dead_code)
)]

pub(super) const SELFMAG: usize = 4;
pub(super) const ELFMAG: [u8; SELFMAG] = [0x7f, b'E', b'L', b'F'];
pub(super) const EI_CLASS: usize = 4;
pub(super) const EI_DATA: usize = 5;
pub(super) const EI_VERSION: usize = 6;
pub(super) const EI_OSABI: usize = 7;
pub(super) const EI_ABIVERSION: usize = 8;
pub(super) const EV_CURRENT: u8 = 1;
#[cfg(target_pointer_width = "32")]
pub(super) const ELFCLASS: u8 = 1; // ELFCLASS32
#[cfg(target_pointer_width = "64")]
pub(super) const ELFCLASS: u8 = 2; // ELFCLASS64
#[cfg(target_endian = "little")]
pub(super) const ELFDATA: u8 = 1; // ELFDATA2LSB
#[cfg(target_endian = "big")]
pub(super) const ELFDATA: u8 = 2; // ELFDATA2MSB
pub(super) const ELFOSABI_SYSV: u8 = 0;
pub(super) const ELFOSABI_LINUX: u8 = 3;
// At present all of our supported platforms use 0.
pub(super) const ELFABIVERSION: u8 = 0;
pub(super) const ET_DYN: u16 = 3;
pub(super) const EI_NIDENT: usize = 16;
pub(super) const SHN_UNDEF: u16 = 0;
pub(super) const SHN_ABS: u16 = 0xfff1;
pub(super) const PN_XNUM: u16 = 0xffff;
pub(super) const PT_LOAD: u32 = 1;
pub(super) const PT_DYNAMIC: u32 = 2;
pub(super) const PT_INTERP: u32 = 3;
pub(super) const PT_PHDR: u32 = 6;
pub(super) const PT_TLS: u32 = 7;
pub(super) const PT_GNU_STACK: u32 = 0x6474_e551;
pub(super) const PT_GNU_RELRO: u32 = 0x6474_e552;
pub(super) const PF_X: u32 = 1;
pub(super) const PF_W: u32 = 2;
pub(super) const PF_R: u32 = 4;
pub(super) const DT_NULL: i32 = 0;
pub(super) const DT_HASH: i32 = 4;
pub(super) const DT_STRTAB: i32 = 5;
pub(super) const DT_SYMTAB: i32 = 6;
pub(super) const DT_SYMENT: i32 = 11;
pub(super) const DT_VERSYM: i32 = 0x6fff_fff0;
pub(super) const DT_VERDEF: i32 = 0x6fff_fffc;
pub(super) const STB_WEAK: u8 = 2;
pub(super) const STB_GLOBAL: u8 = 1;
pub(super) const STT_NOTYPE: u8 = 0;
pub(super) const STT_FUNC: u8 = 2;
pub(super) const STN_UNDEF: u32 = 0;
pub(super) const VER_FLG_BASE: u16 = 0x1;
pub(super) const VER_DEF_CURRENT: u16 = 1;
pub(super) const STV_DEFAULT: u8 = 0;
#[cfg(target_arch = "arm")]
pub(super) const EM_CURRENT: u16 = 40; // EM_ARM
#[cfg(target_arch = "x86")]
pub(super) const EM_CURRENT: u16 = 3; // EM_386
#[cfg(target_arch = "powerpc64")]
pub(super) const EM_CURRENT: u16 = 21; // EM_PPC64
#[cfg(any(target_arch = "mips", target_arch = "mips64"))]
pub(super) const EM_CURRENT: u16 = 8; // EM_MIPS
#[cfg(target_arch = "x86_64")]
pub(super) const EM_CURRENT: u16 = 62; // EM_X86_64
#[cfg(target_arch = "aarch64")]
pub(super) const EM_CURRENT: u16 = 183; // EM_AARCH64
#[cfg(target_arch = "riscv64")]
pub(super) const EM_CURRENT: u16 = 243; // EM_RISCV

#[inline]
pub(super) const fn ELF_ST_VISIBILITY(o: u8) -> u8 {
    o & 0x03
}

#[inline]
pub(super) const fn ELF_ST_BIND(val: u8) -> u8 {
    val >> 4
}

#[inline]
pub(super) const fn ELF_ST_TYPE(val: u8) -> u8 {
    val & 0xf
}

#[repr(C)]
pub(super) struct Elf_Ehdr {
    pub(super) e_ident: [u8; EI_NIDENT],
    pub(super) e_type: u16,
    pub(super) e_machine: u16,
    pub(super) e_version: u32,
    pub(super) e_entry: usize,
    pub(super) e_phoff: usize,
    pub(super) e_shoff: usize,
    pub(super) e_flags: u32,
    pub(super) e_ehsize: u16,
    pub(super) e_phentsize: u16,
    pub(super) e_phnum: u16,
    pub(super) e_shentsize: u16,
    pub(super) e_shnum: u16,
    pub(super) e_shstrndx: u16,
}

#[cfg(target_pointer_width = "32")]
#[repr(C)]
pub(super) struct Elf_Phdr {
    pub(super) p_type: u32,
    pub(super) p_offset: usize,
    pub(super) p_vaddr: usize,
    pub(super) p_paddr: usize,
    pub(super) p_filesz: usize,
    pub(super) p_memsz: usize,
    pub(super) p_flags: u32,
    pub(super) p_align: usize,
}

#[cfg(target_pointer_width = "64")]
#[repr(C)]
pub(super) struct Elf_Phdr {
    pub(super) p_type: u32,
    pub(super) p_flags: u32,
    pub(super) p_offset: usize,
    pub(super) p_vaddr: usize,
    pub(super) p_paddr: usize,
    pub(super) p_filesz: usize,
    pub(super) p_memsz: usize,
    pub(super) p_align: usize,
}

#[cfg(target_pointer_width = "32")]
#[repr(C)]
pub(super) struct Elf_Sym {
    pub(super) st_name: u32,
    pub(super) st_value: usize,
    pub(super) st_size: usize,
    pub(super) st_info: u8,
    pub(super) st_other: u8,
    pub(super) st_shndx: u16,
}

#[cfg(target_pointer_width = "64")]
#[repr(C)]
pub(super) struct Elf_Sym {
    pub(super) st_name: u32,
    pub(super) st_info: u8,
    pub(super) st_other: u8,
    pub(super) st_shndx: u16,
    pub(super) st_value: usize,
    pub(super) st_size: usize,
}

#[repr(C)]
pub(super) struct Elf_Dyn {
    pub(super) d_tag: i32,
    pub(super) d_val: usize,
}

#[repr(C)]
pub(super) struct Elf_Verdef {
    pub(super) vd_version: u16,
    pub(super) vd_flags: u16,
    pub(super) vd_ndx: u16,
    pub(super) vd_cnt: u16,
    pub(super) vd_hash: u32,
    pub(super) vd_aux: u32,
    pub(super) vd_next: u32,
}

#[repr(C)]
pub(super) struct Elf_Verdaux {
    pub(super) vda_name: u32,
    pub(super) _vda_next: u32,
}
