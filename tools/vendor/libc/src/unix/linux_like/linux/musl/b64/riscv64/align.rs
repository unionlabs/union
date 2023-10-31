s_no_extra_traits! {
    #[allow(missing_debug_implementations)]
    pub struct ucontext_t {
        pub __uc_flags: ::c_ulong,
        pub uc_link: *mut ucontext_t,
        pub uc_stack: ::stack_t,
        pub uc_sigmask: ::sigset_t,
        pub uc_mcontext: mcontext_t,
    }

    #[allow(missing_debug_implementations)]
    #[repr(align(16))]
    pub struct mcontext_t {
        pub __gregs: [::c_ulong; 32],
        pub __fpregs: __riscv_mc_fp_state,
    }

    #[allow(missing_debug_implementations)]
    pub union __riscv_mc_fp_state {
        pub __f: __riscv_mc_f_ext_state,
        pub __d: __riscv_mc_d_ext_state,
        pub __q: __riscv_mc_q_ext_state,
    }

    #[allow(missing_debug_implementations)]
    pub struct __riscv_mc_f_ext_state {
        pub __f: [::c_uint; 32],
        pub __fcsr: ::c_uint,
    }

    #[allow(missing_debug_implementations)]
    pub struct __riscv_mc_d_ext_state {
        pub __f: [::c_ulonglong; 32],
        pub __fcsr: ::c_uint,
    }

    #[allow(missing_debug_implementations)]
    #[repr(align(16))]
    pub struct __riscv_mc_q_ext_state {
        pub __f: [::c_ulonglong; 64],
        pub __fcsr: ::c_uint,
        pub __glibc_reserved: [::c_uint; 3],
    }
}
