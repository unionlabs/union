pub type c_char = i8;
pub type c_long = i64;
pub type c_ulong = u64;
pub type wchar_t = i32;
pub type time_t = i64;
pub type suseconds_t = i64;
pub type register_t = i64;

s! {
    pub struct reg32 {
        pub r_fs: u32,
        pub r_es: u32,
        pub r_ds: u32,
        pub r_edi: u32,
        pub r_esi: u32,
        pub r_ebp: u32,
        pub r_isp: u32,
        pub r_ebx: u32,
        pub r_edx: u32,
        pub r_ecx: u32,
        pub r_eax: u32,
        pub r_trapno: u32,
        pub r_err: u32,
        pub r_eip: u32,
        pub r_cs: u32,
        pub r_eflags: u32,
        pub r_esp: u32,
        pub r_ss: u32,
        pub r_gs: u32,
    }

    pub struct reg {
        pub r_r15: i64,
        pub r_r14: i64,
        pub r_r13: i64,
        pub r_r12: i64,
        pub r_r11: i64,
        pub r_r10: i64,
        pub r_r9: i64,
        pub r_r8: i64,
        pub r_rdi: i64,
        pub r_rsi: i64,
        pub r_rbp: i64,
        pub r_rbx: i64,
        pub r_rdx: i64,
        pub r_rcx: i64,
        pub r_rax: i64,
        pub r_trapno: u32,
        pub r_fs: u16,
        pub r_gs: u16,
        pub r_err: u32,
        pub r_es: u16,
        pub r_ds: u16,
        pub r_rip: i64,
        pub r_cs: i64,
        pub r_rflags: i64,
        pub r_rsp: i64,
        pub r_ss: i64,
    }
}

s_no_extra_traits! {
    pub struct fpreg32 {
        pub fpr_env: [u32; 7],
        pub fpr_acc: [[u8; 10]; 8],
        pub fpr_ex_sw: u32,
        pub fpr_pad: [u8; 64],
    }

    pub struct fpreg {
        pub fpr_env: [u64; 4],
        pub fpr_acc: [[u8; 16]; 8],
        pub fpr_xacc: [[u8; 16]; 16],
        pub fpr_spare: [u64; 12],
    }

    pub struct xmmreg {
        pub xmm_env: [u32; 8],
        pub xmm_acc: [[u8; 16]; 8],
        pub xmm_reg: [[u8; 16]; 8],
        pub xmm_pad: [u8; 224],
    }

    #[cfg(libc_union)]
    pub union __c_anonymous_elf64_auxv_union {
        pub a_val: ::c_long,
        pub a_ptr: *mut ::c_void,
        pub a_fcn: extern "C" fn(),
    }

    pub struct Elf64_Auxinfo {
        pub a_type: ::c_long,
        #[cfg(libc_union)]
        pub a_un: __c_anonymous_elf64_auxv_union,
    }

    pub struct kinfo_file {
        pub kf_structsize: ::c_int,
        pub kf_type: ::c_int,
        pub kf_fd: ::c_int,
        pub kf_ref_count: ::c_int,
        pub kf_flags: ::c_int,
        _kf_pad0: ::c_int,
        pub kf_offset: i64,
        _priv: [::uintptr_t; 38], // FIXME if needed
        pub kf_status: u16,
        _kf_pad1: u16,
        _kf_ispare0: ::c_int,
        pub kf_cap_rights: ::cap_rights_t,
        _kf_cap_spare: u64,
        pub kf_path: [::c_char; ::PATH_MAX as usize],
    }
}

cfg_if! {
    if #[cfg(feature = "extra_traits")] {
        impl PartialEq for fpreg32 {
            fn eq(&self, other: &fpreg32) -> bool {
                self.fpr_env == other.fpr_env &&
                    self.fpr_acc == other.fpr_acc &&
                    self.fpr_ex_sw == other.fpr_ex_sw &&
                    self.fpr_pad
                        .iter()
                        .zip(other.fpr_pad.iter())
                        .all(|(a,b)| a == b)
            }
        }
        impl Eq for fpreg32 {}
        impl ::fmt::Debug for fpreg32 {
            fn fmt(&self, f: &mut ::fmt::Formatter) -> ::fmt::Result {
                f.debug_struct("fpreg32")
                    .field("fpr_env", &&self.fpr_env[..])
                    .field("fpr_acc", &self.fpr_acc)
                    .field("fpr_ex_sw", &self.fpr_ex_sw)
                    .field("fpr_pad", &&self.fpr_pad[..])
                    .finish()
            }
        }
        impl ::hash::Hash for fpreg32 {
            fn hash<H: ::hash::Hasher>(&self, state: &mut H) {
                self.fpr_env.hash(state);
                self.fpr_acc.hash(state);
                self.fpr_ex_sw.hash(state);
                self.fpr_pad.hash(state);
            }
        }

        impl PartialEq for fpreg {
            fn eq(&self, other: &fpreg) -> bool {
                self.fpr_env == other.fpr_env &&
                    self.fpr_acc == other.fpr_acc &&
                    self.fpr_xacc == other.fpr_xacc &&
                    self.fpr_spare == other.fpr_spare
            }
        }
        impl Eq for fpreg {}
        impl ::fmt::Debug for fpreg {
            fn fmt(&self, f: &mut ::fmt::Formatter) -> ::fmt::Result {
                f.debug_struct("fpreg")
                    .field("fpr_env", &self.fpr_env)
                    .field("fpr_acc", &self.fpr_acc)
                    .field("fpr_xacc", &self.fpr_xacc)
                    .field("fpr_spare", &self.fpr_spare)
                    .finish()
            }
        }
        impl ::hash::Hash for fpreg {
            fn hash<H: ::hash::Hasher>(&self, state: &mut H) {
                self.fpr_env.hash(state);
                self.fpr_acc.hash(state);
                self.fpr_xacc.hash(state);
                self.fpr_spare.hash(state);
            }
        }

        impl PartialEq for xmmreg {
            fn eq(&self, other: &xmmreg) -> bool {
                self.xmm_env == other.xmm_env &&
                    self.xmm_acc == other.xmm_acc &&
                    self.xmm_reg == other.xmm_reg &&
                    self.xmm_pad
                        .iter()
                        .zip(other.xmm_pad.iter())
                        .all(|(a,b)| a == b)
            }
        }
        impl Eq for xmmreg {}
        impl ::fmt::Debug for xmmreg {
            fn fmt(&self, f: &mut ::fmt::Formatter) -> ::fmt::Result {
                f.debug_struct("xmmreg")
                    .field("xmm_env", &self.xmm_env)
                    .field("xmm_acc", &self.xmm_acc)
                    .field("xmm_reg", &self.xmm_reg)
                    .field("xmm_pad", &&self.xmm_pad[..])
                    .finish()
            }
        }
        impl ::hash::Hash for xmmreg {
            fn hash<H: ::hash::Hasher>(&self, state: &mut H) {
                self.xmm_env.hash(state);
                self.xmm_acc.hash(state);
                self.xmm_reg.hash(state);
                self.xmm_pad.hash(state);
            }
        }

        #[cfg(libc_union)]
        impl PartialEq for __c_anonymous_elf64_auxv_union {
            fn eq(&self, other: &__c_anonymous_elf64_auxv_union) -> bool {
                unsafe { self.a_val == other.a_val
                        || self.a_ptr == other.a_ptr
                        || self.a_fcn == other.a_fcn }
            }
        }
        #[cfg(libc_union)]
        impl Eq for __c_anonymous_elf64_auxv_union {}
        #[cfg(libc_union)]
        impl ::fmt::Debug for __c_anonymous_elf64_auxv_union {
            fn fmt(&self, f: &mut ::fmt::Formatter) -> ::fmt::Result {
                f.debug_struct("a_val")
                    .field("a_val", unsafe { &self.a_val })
                    .finish()
            }
        }
        #[cfg(not(libc_union))]
        impl PartialEq for Elf64_Auxinfo {
            fn eq(&self, other: &Elf64_Auxinfo) -> bool {
                self.a_type == other.a_type
            }
        }
        #[cfg(libc_union)]
        impl PartialEq for Elf64_Auxinfo {
            fn eq(&self, other: &Elf64_Auxinfo) -> bool {
                self.a_type == other.a_type
                    && self.a_un == other.a_un
            }
        }
        impl Eq for Elf64_Auxinfo {}
        #[cfg(not(libc_union))]
        impl ::fmt::Debug for Elf64_Auxinfo {
            fn fmt(&self, f: &mut ::fmt::Formatter) -> ::fmt::Result {
                f.debug_struct("Elf64_Auxinfo")
                    .field("a_type", &self.a_type)
                    .finish()
            }
        }
        #[cfg(libc_union)]
        impl ::fmt::Debug for Elf64_Auxinfo {
            fn fmt(&self, f: &mut ::fmt::Formatter) -> ::fmt::Result {
                f.debug_struct("Elf64_Auxinfo")
                    .field("a_type", &self.a_type)
                    .field("a_un", &self.a_un)
                    .finish()
            }
        }

        impl PartialEq for kinfo_file {
            fn eq(&self, other: &kinfo_file) -> bool {
                self.kf_structsize == other.kf_structsize &&
                    self.kf_type == other.kf_type &&
                    self.kf_fd == other.kf_fd &&
                    self.kf_ref_count == other.kf_ref_count &&
                    self.kf_flags == other.kf_flags &&
                    self.kf_offset == other.kf_offset &&
                    self.kf_status == other.kf_status &&
                    self.kf_cap_rights == other.kf_cap_rights &&
                    self.kf_path
                        .iter()
                        .zip(other.kf_path.iter())
                        .all(|(a,b)| a == b)
            }
        }
        impl Eq for kinfo_file {}
        impl ::fmt::Debug for kinfo_file {
            fn fmt(&self, f: &mut ::fmt::Formatter) -> ::fmt::Result {
                f.debug_struct("kinfo_file")
                    .field("kf_structsize", &self.kf_structsize)
                    .field("kf_type", &self.kf_type)
                    .field("kf_fd", &self.kf_fd)
                    .field("kf_ref_count", &self.kf_ref_count)
                    .field("kf_flags", &self.kf_flags)
                    .field("kf_offset", &self.kf_offset)
                    .field("kf_status", &self.kf_status)
                    .field("kf_cap_rights", &self.kf_cap_rights)
                    .field("kf_path", &&self.kf_path[..])
                    .finish()
            }
        }
        impl ::hash::Hash for kinfo_file {
            fn hash<H: ::hash::Hasher>(&self, state: &mut H) {
                self.kf_structsize.hash(state);
                self.kf_type.hash(state);
                self.kf_fd.hash(state);
                self.kf_ref_count.hash(state);
                self.kf_flags.hash(state);
                self.kf_offset.hash(state);
                self.kf_status.hash(state);
                self.kf_cap_rights.hash(state);
                self.kf_path.hash(state);
            }
        }
    }
}

// should be pub(crate), but that requires Rust 1.18.0
cfg_if! {
    if #[cfg(libc_const_size_of)] {
        #[doc(hidden)]
        pub const _ALIGNBYTES: usize = ::mem::size_of::<::c_long>() - 1;
    } else {
        #[doc(hidden)]
        pub const _ALIGNBYTES: usize = 8 - 1;
    }
}
pub const MAP_32BIT: ::c_int = 0x00080000;
pub const MINSIGSTKSZ: ::size_t = 2048; // 512 * 4

pub const _MC_HASSEGS: u32 = 0x1;
pub const _MC_HASBASES: u32 = 0x2;
pub const _MC_HASFPXSTATE: u32 = 0x4;
pub const _MC_FLAG_MASK: u32 = _MC_HASSEGS | _MC_HASBASES | _MC_HASFPXSTATE;

pub const _MC_FPFMT_NODEV: c_long = 0x10000;
pub const _MC_FPFMT_XMM: c_long = 0x10002;
pub const _MC_FPOWNED_NONE: c_long = 0x20000;
pub const _MC_FPOWNED_FPU: c_long = 0x20001;
pub const _MC_FPOWNED_PCB: c_long = 0x20002;

cfg_if! {
    if #[cfg(libc_align)] {
        mod align;
        pub use self::align::*;
    }
}
