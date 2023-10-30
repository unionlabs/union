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
}
