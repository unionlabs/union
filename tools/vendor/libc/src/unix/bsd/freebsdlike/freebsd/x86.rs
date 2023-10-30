pub type c_char = i8;
pub type c_long = i32;
pub type c_ulong = u32;
pub type wchar_t = i32;
pub type time_t = i32;
pub type suseconds_t = i32;
pub type register_t = i32;

s_no_extra_traits! {
    pub struct mcontext_t {
        pub mc_onstack: register_t,
        pub mc_gs: register_t,
        pub mc_fs: register_t,
        pub mc_es: register_t,
        pub mc_ds: register_t,
        pub mc_edi: register_t,
        pub mc_esi: register_t,
        pub mc_ebp: register_t,
        pub mc_isp: register_t,
        pub mc_ebx: register_t,
        pub mc_edx: register_t,
        pub mc_ecx: register_t,
        pub mc_eax: register_t,
        pub mc_trapno: register_t,
        pub mc_err: register_t,
        pub mc_eip: register_t,
        pub mc_cs: register_t,
        pub mc_eflags: register_t,
        pub mc_esp: register_t,
        pub mc_ss: register_t,
        pub mc_len: ::c_int,
        pub mc_fpformat: ::c_int,
        pub mc_ownedfp: ::c_int,
        pub mc_flags: register_t,
        pub mc_fpstate: [[::c_int; 32]; 4],
        pub mc_fsbase: register_t,
        pub mc_gsbase: register_t,
        pub mc_xfpustate: register_t,
        pub mc_xfpustate_len: register_t,
        pub mc_spare2: [::c_int; 4],
    }
}

s! {
    pub struct stat {
        pub st_dev: ::dev_t,
        pub st_ino: ::ino_t,
        pub st_mode: ::mode_t,
        pub st_nlink: ::nlink_t,
        pub st_uid: ::uid_t,
        pub st_gid: ::gid_t,
        pub st_rdev: ::dev_t,
        pub st_atime: ::time_t,
        pub st_atime_nsec: ::c_long,
        pub st_mtime: ::time_t,
        pub st_mtime_nsec: ::c_long,
        pub st_ctime: ::time_t,
        pub st_ctime_nsec: ::c_long,
        pub st_size: ::off_t,
        pub st_blocks: ::blkcnt_t,
        pub st_blksize: ::blksize_t,
        pub st_flags: ::fflags_t,
        pub st_gen: u32,
        pub st_lspare: i32,
        pub st_birthtime: ::time_t,
        pub st_birthtime_nsec: ::c_long,
        __unused: [u8; 8],
    }

    pub struct ucontext_t {
        pub uc_sigmask: ::sigset_t,
        pub uc_mcontext: ::mcontext_t,
        pub uc_link: *mut ::ucontext_t,
        pub uc_stack: ::stack_t,
        pub uc_flags: ::c_int,
        __spare__: [::c_int; 4],
    }
}

// should be pub(crate), but that requires Rust 1.18.0
cfg_if! {
    if #[cfg(libc_const_size_of)] {
        #[doc(hidden)]
        pub const _ALIGNBYTES: usize = ::mem::size_of::<::c_long>() - 1;
    } else {
        #[doc(hidden)]
        pub const _ALIGNBYTES: usize = 4 - 1;
    }
}

cfg_if! {
    if #[cfg(feature = "extra_traits")] {
        impl PartialEq for mcontext_t {
            fn eq(&self, other: &mcontext_t) -> bool {
                self.mc_onstack == other.mc_onstack &&
                self.mc_gs == other.mc_gs &&
                self.mc_fs == other.mc_fs &&
                self.mc_es == other.mc_es &&
                self.mc_ds == other.mc_ds &&
                self.mc_edi == other.mc_edi &&
                self.mc_esi == other.mc_esi &&
                self.mc_ebp == other.mc_ebp &&
                self.mc_isp == other.mc_isp &&
                self.mc_ebx == other.mc_ebx &&
                self.mc_edx == other.mc_edx &&
                self.mc_ecx == other.mc_ecx &&
                self.mc_eax == other.mc_eax &&
                self.mc_trapno == other.mc_trapno &&
                self.mc_err == other.mc_err &&
                self.mc_eip == other.mc_eip &&
                self.mc_cs == other.mc_cs &&
                self.mc_eflags == other.mc_eflags &&
                self.mc_esp == other.mc_esp &&
                self.mc_ss == other.mc_ss &&
                self.mc_len == other.mc_len &&
                self.mc_fpformat == other.mc_fpformat &&
                self.mc_ownedfp == other.mc_ownedfp &&
                self.mc_flags == other.mc_flags &&
                self.mc_fpstate.iter().zip(other.mc_fpstate.iter()).all(|(a, b)| a == b) &&
                self.mc_fsbase == other.mc_fsbase &&
                self.mc_gsbase == other.mc_gsbase &&
                self.mc_xfpustate == other.mc_xfpustate &&
                self.mc_xfpustate_len == other.mc_xfpustate_len &&
                self.mc_spare2.iter().zip(other.mc_spare2.iter()).all(|(a, b)| a == b)
            }
        }
        impl Eq for mcontext_t {}
        impl ::fmt::Debug for mcontext_t {
            fn fmt(&self, f: &mut ::fmt::Formatter) -> ::fmt::Result {
                f.debug_struct("mcontext_t")
                    .field("mc_onstack", &self.mc_onstack)
                    .field("mc_gs", &self.mc_gs)
                    .field("mc_fs", &self.mc_fs)
                    .field("mc_es", &self.mc_es)
                    .field("mc_ds", &self.mc_ds)
                    .field("mc_edi", &self.mc_edi)
                    .field("mc_esi", &self.mc_esi)
                    .field("mc_ebp", &self.mc_ebp)
                    .field("mc_isp", &self.mc_isp)
                    .field("mc_ebx", &self.mc_ebx)
                    .field("mc_edx", &self.mc_edx)
                    .field("mc_ecx", &self.mc_ecx)
                    .field("mc_eax", &self.mc_eax)
                    .field("mc_trapno", &self.mc_trapno)
                    .field("mc_err", &self.mc_err)
                    .field("mc_eip", &self.mc_eip)
                    .field("mc_cs", &self.mc_cs)
                    .field("mc_eflags", &self.mc_eflags)
                    .field("mc_esp", &self.mc_esp)
                    .field("mc_ss", &self.mc_ss)
                    .field("mc_len", &self.mc_len)
                    .field("mc_fpformat", &self.mc_fpformat)
                    .field("mc_ownedfp", &self.mc_ownedfp)
                    .field("mc_flags", &self.mc_flags)
                    .field("mc_fpstate", &self.mc_fpstate)
                    .field("mc_fsbase", &self.mc_fsbase)
                    .field("mc_gsbase", &self.mc_gsbase)
                    .field("mc_xfpustate", &self.mc_xfpustate)
                    .field("mc_xfpustate_len", &self.mc_xfpustate_len)
                    .field("mc_spare2", &self.mc_spare2)
                    .finish()
            }
        }
        impl ::hash::Hash for mcontext_t {
            fn hash<H: ::hash::Hasher>(&self, state: &mut H) {
                self.mc_onstack.hash(state);
                self.mc_gs.hash(state);
                self.mc_fs.hash(state);
                self.mc_es.hash(state);
                self.mc_ds.hash(state);
                self.mc_edi.hash(state);
                self.mc_esi.hash(state);
                self.mc_ebp.hash(state);
                self.mc_isp.hash(state);
                self.mc_ebx.hash(state);
                self.mc_edx.hash(state);
                self.mc_ecx.hash(state);
                self.mc_eax.hash(state);
                self.mc_trapno.hash(state);
                self.mc_err.hash(state);
                self.mc_eip.hash(state);
                self.mc_cs.hash(state);
                self.mc_eflags.hash(state);
                self.mc_esp.hash(state);
                self.mc_ss.hash(state);
                self.mc_len.hash(state);
                self.mc_fpformat.hash(state);
                self.mc_ownedfp.hash(state);
                self.mc_flags.hash(state);
                self.mc_fpstate.hash(state);
                self.mc_fsbase.hash(state);
                self.mc_gsbase.hash(state);
                self.mc_xfpustate.hash(state);
                self.mc_xfpustate_len.hash(state);
                self.mc_spare2.hash(state);
            }
        }
    }
}

pub const MINSIGSTKSZ: ::size_t = 2048; // 512 * 4
