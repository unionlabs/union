pub type mcontext_t = *mut __darwin_mcontext64;

s_no_extra_traits! {
    #[allow(missing_debug_implementations)]
    pub struct max_align_t {
        priv_: f64
    }
}

s! {
    pub struct ucontext_t {
        pub uc_onstack: ::c_int,
        pub uc_sigmask: ::sigset_t,
        pub uc_stack: ::stack_t,
        pub uc_link: *mut ::ucontext_t,
        pub uc_mcsize: usize,
        pub uc_mcontext: mcontext_t,
        __mcontext_data: __darwin_mcontext64,
    }

    pub struct __darwin_mcontext64 {
        pub __es: __darwin_arm_exception_state64,
        pub __ss: __darwin_arm_thread_state64,
        pub __ns: __darwin_arm_neon_state64,
    }

    pub struct __darwin_arm_exception_state64 {
        pub __far: u64,
        pub __esr: u32,
        pub __exception: u32,
    }

    pub struct __darwin_arm_thread_state64 {
        pub __x: [u64; 29],
        pub __fp: u64,
        pub __lr: u64,
        pub __sp: u64,
        pub __pc: u64,
        pub __cpsr: u32,
        pub __pad: u32,
    }

    // This type natively uses a uint128, but for a while we hacked
    // it in with repr(align) and `[u64; 2]`. uint128 isn't available
    // all the way back to our earliest supported versions so we
    // preserver the old shim.
    #[cfg_attr(not(libc_int128), repr(align(16)))]
    pub struct __darwin_arm_neon_state64 {
        #[cfg(libc_int128)]
        pub __v: [::__uint128_t; 32],
        #[cfg(not(libc_int128))]
        pub __v: [[u64; 2]; 32],
        pub __fpsr: u32,
        pub __fpcr: u32,
    }
}
