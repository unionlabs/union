s_no_extra_traits! {
    #[allow(missing_debug_implementations)]
    #[repr(align(16))]
    pub struct max_align_t {
        priv_: [f64; 4]
    }

}

s! {
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
