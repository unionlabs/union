use PT_FIRSTMACH;

pub type c_long = i64;
pub type c_ulong = u64;
pub type c_char = u8;
pub type greg_t = u64;
pub type __cpu_simple_lock_nv_t = ::c_uchar;

s! {
    pub struct __fregset {
        #[cfg(libc_union)]
        pub __qregs: [__c_anonymous__freg; 32],
        pub __fpcr: u32,
        pub __fpsr: u32,
    }

    pub struct mcontext_t {
        pub __gregs: [::greg_t; 32],
        pub __fregs: __fregset,
        __spare: [::greg_t; 8],
    }

    pub struct ucontext_t {
        pub uc_flags: ::c_uint,
        pub uc_link: *mut ucontext_t,
        pub uc_sigmask: ::sigset_t,
        pub uc_stack: ::stack_t,
        pub uc_mcontext: mcontext_t,
    }
}

s_no_extra_traits! {
    #[cfg(libc_union)]
    #[repr(align(16))]
    pub union __c_anonymous__freg {
        pub __b8: [u8; 16],
        pub __h16: [u16; 8],
        pub __s32: [u32; 4],
        pub __d64: [u64; 2],
        pub __q128: [u128; 1],
    }
}

cfg_if! {
    if #[cfg(feature = "extra_traits")] {
        #[cfg(libc_union)]
        impl PartialEq for __c_anonymous__freg {
            fn eq(&self, other: &__c_anonymous__freg) -> bool {
                unsafe {
                self.__b8 == other.__b8
                    || self.__h16 == other.__h16
                    || self.__s32 == other.__s32
                    || self.__d64 == other.__d64
                    || self.__q128 == other.__q128
                }
            }
        }
        #[cfg(libc_union)]
        impl Eq for __c_anonymous__freg {}
        #[cfg(libc_union)]
        impl ::fmt::Debug for __c_anonymous__freg {
            fn fmt(&self, f: &mut ::fmt::Formatter) -> ::fmt::Result {
                unsafe {
                f.debug_struct("__c_anonymous__freg")
                    .field("__b8", &self.__b8)
                    .field("__h16", &self.__h16)
                    .field("__s32", &self.__s32)
                    .field("__d64", &self.__d64)
                    .field("__q128", &self.__q128)
                    .finish()
                }
            }
        }
        #[cfg(libc_union)]
        impl ::hash::Hash for __c_anonymous__freg {
            fn hash<H: ::hash::Hasher>(&self, state: &mut H) {
                unsafe {
                self.__b8.hash(state);
                self.__h16.hash(state);
                self.__s32.hash(state);
                self.__d64.hash(state);
                self.__q128.hash(state);
                }
            }
        }
    }
}

// should be pub(crate), but that requires Rust 1.18.0
cfg_if! {
    if #[cfg(libc_const_size_of)] {
        #[doc(hidden)]
        pub const _ALIGNBYTES: usize = ::mem::size_of::<::c_int>() - 1;
    } else {
        #[doc(hidden)]
        pub const _ALIGNBYTES: usize = 4 - 1;
    }
}

pub const PT_GETREGS: ::c_int = PT_FIRSTMACH + 0;
pub const PT_SETREGS: ::c_int = PT_FIRSTMACH + 1;
pub const PT_GETFPREGS: ::c_int = PT_FIRSTMACH + 2;
pub const PT_SETFPREGS: ::c_int = PT_FIRSTMACH + 3;
