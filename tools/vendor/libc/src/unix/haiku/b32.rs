pub type c_long = i32;
pub type c_ulong = u32;
pub type time_t = i32;

pub type Elf_Addr = ::Elf32_Addr;
pub type Elf_Half = ::Elf32_Half;
pub type Elf_Phdr = ::Elf32_Phdr;

s! {
    pub struct Elf32_Phdr {
        pub p_type: ::Elf32_Word,
        pub p_offset: ::Elf32_Off,
        pub p_vaddr: ::Elf32_Addr,
        pub p_paddr: ::Elf32_Addr,
        pub p_filesz: ::Elf32_Word,
        pub p_memsz: ::Elf32_Word,
        pub p_flags: ::Elf32_Word,
        pub p_align: ::Elf32_Word,
    }
}
