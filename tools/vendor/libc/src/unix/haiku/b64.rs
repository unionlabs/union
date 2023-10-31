pub type c_ulong = u64;
pub type c_long = i64;
pub type time_t = i64;

pub type Elf_Addr = ::Elf64_Addr;
pub type Elf_Half = ::Elf64_Half;
pub type Elf_Phdr = ::Elf64_Phdr;

s! {
    pub struct Elf64_Phdr {
        pub p_type: ::Elf64_Word,
        pub p_flags: ::Elf64_Word,
        pub p_offset: ::Elf64_Off,
        pub p_vaddr: ::Elf64_Addr,
        pub p_paddr: ::Elf64_Addr,
        pub p_filesz: ::Elf64_Xword,
        pub p_memsz: ::Elf64_Xword,
        pub p_align: ::Elf64_Xword,
    }
}
