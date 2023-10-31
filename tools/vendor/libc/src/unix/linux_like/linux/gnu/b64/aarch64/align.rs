s_no_extra_traits! {
    #[allow(missing_debug_implementations)]
    #[repr(align(16))]
    pub struct max_align_t {
        priv_: [f32; 8]
    }
}

s! {
    pub struct ucontext_t {
        pub uc_flags: ::c_ulong,
        pub uc_link: *mut ucontext_t,
        pub uc_stack: ::stack_t,
        pub uc_sigmask: ::sigset_t,
        pub uc_mcontext: mcontext_t,
    }

    #[repr(align(16))]
    pub struct mcontext_t {
        pub fault_address: ::c_ulonglong,
        pub regs: [::c_ulonglong; 31],
        pub sp: ::c_ulonglong,
        pub pc: ::c_ulonglong,
        pub pstate: ::c_ulonglong,
        // nested arrays to get the right size/length while being able to
        // auto-derive traits like Debug
        __reserved: [[u64; 32]; 16],
    }

    #[repr(align(16))]
    pub struct user_fpsimd_struct {
        pub vregs: [[u64; 2]; 32],
        pub fpsr: ::c_uint,
        pub fpcr: ::c_uint,
    }

    #[repr(align(8))]
    pub struct clone_args {
        pub flags: ::c_ulonglong,
        pub pidfd: ::c_ulonglong,
        pub child_tid: ::c_ulonglong,
        pub parent_tid: ::c_ulonglong,
        pub exit_signal: ::c_ulonglong,
        pub stack: ::c_ulonglong,
        pub stack_size: ::c_ulonglong,
        pub tls: ::c_ulonglong,
        pub set_tid: ::c_ulonglong,
        pub set_tid_size: ::c_ulonglong,
        pub cgroup: ::c_ulonglong,
    }
}

extern "C" {
    pub fn getcontext(ucp: *mut ucontext_t) -> ::c_int;
    pub fn setcontext(ucp: *const ucontext_t) -> ::c_int;
    pub fn makecontext(ucp: *mut ucontext_t, func: extern "C" fn(), argc: ::c_int, ...);
    pub fn swapcontext(uocp: *mut ucontext_t, ucp: *const ucontext_t) -> ::c_int;
}
