pub type greg_t = ::c_long;

pub type Elf64_Addr = ::c_ulong;
pub type Elf64_Half = ::c_ushort;
pub type Elf64_Off = ::c_ulong;
pub type Elf64_Sword = ::c_int;
pub type Elf64_Sxword = ::c_long;
pub type Elf64_Word = ::c_uint;
pub type Elf64_Xword = ::c_ulong;
pub type Elf64_Lword = ::c_ulong;
pub type Elf64_Phdr = __c_anonymous_Elf64_Phdr;

s! {
    pub struct __c_anonymous_fpchip_state {
        pub cw: u16,
        pub sw: u16,
        pub fctw: u8,
        pub __fx_rsvd: u8,
        pub fop: u16,
        pub rip: u64,
        pub rdp: u64,
        pub mxcsr: u32,
        pub mxcsr_mask: u32,
        pub st: [::upad128_t; 8],
        pub xmm: [::upad128_t; 16],
        pub __fx_ign: [::upad128_t; 6],
        pub status: u32,
        pub xstatus: u32,
    }

    pub struct __c_anonymous_Elf64_Phdr {
        pub p_type: ::Elf64_Word,
        pub p_flags: ::Elf64_Word,
        pub p_offset: ::Elf64_Off,
        pub p_vaddr: ::Elf64_Addr,
        pub p_paddr: ::Elf64_Addr,
        pub p_filesz: ::Elf64_Xword,
        pub p_memsz: ::Elf64_Xword,
        pub p_align: ::Elf64_Xword,
    }

    pub struct dl_phdr_info {
        pub dlpi_addr: ::Elf64_Addr,
        pub dlpi_name: *const ::c_char,
        pub dlpi_phdr: *const ::Elf64_Phdr,
        pub dlpi_phnum: ::Elf64_Half,
        pub dlpi_adds: ::c_ulonglong,
        pub dlpi_subs: ::c_ulonglong,
    }
}

s_no_extra_traits! {
    #[cfg(libc_union)]
    pub union __c_anonymous_fp_reg_set {
        pub fpchip_state: __c_anonymous_fpchip_state,
        pub f_fpregs: [[u32; 13]; 10],
    }

    pub struct fpregset_t {
        pub fp_reg_set: __c_anonymous_fp_reg_set,
    }

    pub struct mcontext_t {
        pub gregs: [::greg_t; 28],
        pub fpregs: fpregset_t,
    }

    pub struct ucontext_t {
        pub uc_flags: ::c_ulong,
        pub uc_link: *mut ucontext_t,
        pub uc_sigmask: ::sigset_t,
        pub uc_stack: ::stack_t,
        pub uc_mcontext: mcontext_t,
        pub uc_filler: [::c_long; 5],
    }
}

cfg_if! {
    if #[cfg(feature = "extra_traits")] {
        #[cfg(libc_union)]
        impl PartialEq for __c_anonymous_fp_reg_set {
            fn eq(&self, other: &__c_anonymous_fp_reg_set) -> bool {
                unsafe {
                self.fpchip_state == other.fpchip_state ||
                    self.
                    f_fpregs.
                    iter().
                    zip(other.f_fpregs.iter()).
                    all(|(a, b)| a == b)
                }
            }
        }
        #[cfg(libc_union)]
        impl Eq for __c_anonymous_fp_reg_set {}
        #[cfg(libc_union)]
        impl ::fmt::Debug for __c_anonymous_fp_reg_set {
            fn fmt(&self, f: &mut ::fmt::Formatter) -> ::fmt::Result {
                unsafe {
                f.debug_struct("__c_anonymous_fp_reg_set")
                    .field("fpchip_state", &{self.fpchip_state})
                    .field("f_fpregs", &{self.f_fpregs})
                    .finish()
                }
            }
        }
        impl PartialEq for fpregset_t {
            fn eq(&self, other: &fpregset_t) -> bool {
                self.fp_reg_set == other.fp_reg_set
            }
        }
        impl Eq for fpregset_t {}
        impl ::fmt::Debug for fpregset_t {
            fn fmt(&self, f:&mut ::fmt::Formatter) -> ::fmt::Result {
                f.debug_struct("fpregset_t")
                    .field("fp_reg_set", &self.fp_reg_set)
                    .finish()
            }
        }
        impl PartialEq for mcontext_t {
            fn eq(&self, other: &mcontext_t) -> bool {
                self.gregs == other.gregs &&
                    self.fpregs == other.fpregs
            }
        }
        impl Eq for mcontext_t {}
        impl ::fmt::Debug for mcontext_t {
            fn fmt(&self, f:&mut ::fmt::Formatter) -> ::fmt::Result {
                f.debug_struct("mcontext_t")
                    .field("gregs", &self.gregs)
                    .field("fpregs", &self.fpregs)
                    .finish()
            }
        }
        impl PartialEq for ucontext_t {
            fn eq(&self, other: &ucontext_t) -> bool {
                self.uc_flags == other.uc_flags
                    && self.uc_link == other.uc_link
                    && self.uc_sigmask == other.uc_sigmask
                    && self.uc_stack == other.uc_stack
                    && self.uc_mcontext == other.uc_mcontext
                    && self.uc_filler == other.uc_filler
            }
        }
        impl Eq for ucontext_t {}
        impl ::fmt::Debug for ucontext_t {
            fn fmt(&self, f:&mut ::fmt::Formatter) -> ::fmt::Result {
                f.debug_struct("ucontext_t")
                    .field("uc_flags", &self.uc_flags)
                    .field("uc_link", &self.uc_link)
                    .field("uc_sigmask", &self.uc_sigmask)
                    .field("uc_stack", &self.uc_stack)
                    .field("uc_mcontext", &self.uc_mcontext)
                    .field("uc_filler", &self.uc_filler)
                    .finish()
            }
        }

    }
}

// sys/regset.h

pub const REG_GSBASE: ::c_int = 27;
pub const REG_FSBASE: ::c_int = 26;
pub const REG_DS: ::c_int = 25;
pub const REG_ES: ::c_int = 24;
pub const REG_GS: ::c_int = 23;
pub const REG_FS: ::c_int = 22;
pub const REG_SS: ::c_int = 21;
pub const REG_RSP: ::c_int = 20;
pub const REG_RFL: ::c_int = 19;
pub const REG_CS: ::c_int = 18;
pub const REG_RIP: ::c_int = 17;
pub const REG_ERR: ::c_int = 16;
pub const REG_TRAPNO: ::c_int = 15;
pub const REG_RAX: ::c_int = 14;
pub const REG_RCX: ::c_int = 13;
pub const REG_RDX: ::c_int = 12;
pub const REG_RBX: ::c_int = 11;
pub const REG_RBP: ::c_int = 10;
pub const REG_RSI: ::c_int = 9;
pub const REG_RDI: ::c_int = 8;
pub const REG_R8: ::c_int = 7;
pub const REG_R9: ::c_int = 6;
pub const REG_R10: ::c_int = 5;
pub const REG_R11: ::c_int = 4;
pub const REG_R12: ::c_int = 3;
pub const REG_R13: ::c_int = 2;
pub const REG_R14: ::c_int = 1;
pub const REG_R15: ::c_int = 0;
