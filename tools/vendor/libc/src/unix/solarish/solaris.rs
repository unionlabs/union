pub type door_attr_t = ::c_uint;
pub type door_id_t = ::c_ulonglong;

s! {
    pub struct shmid_ds {
        pub shm_perm: ::ipc_perm,
        pub shm_segsz: ::size_t,
        pub shm_flags: ::uintptr_t,
        pub shm_lkcnt: ::c_ushort,
        pub shm_lpid: ::pid_t,
        pub shm_cpid: ::pid_t,
        pub shm_nattch: ::shmatt_t,
        pub shm_cnattch: ::c_ulong,
        pub shm_atime: ::time_t,
        pub shm_dtime: ::time_t,
        pub shm_ctime: ::time_t,
        pub shm_amp: *mut ::c_void,
        pub shm_gransize: u64,
        pub shm_allocated: u64,
        pub shm_pad4: [i64; 1],
    }

    pub struct door_desc_t__d_data__d_desc {
        pub d_descriptor: ::c_int,
        pub d_id: ::door_id_t
    }
}

s_no_extra_traits! {
    #[cfg_attr(feature = "extra_traits", allow(missing_debug_implementations))]
    pub union door_desc_t__d_data {
        pub d_desc: door_desc_t__d_data__d_desc,
        d_resv: [::c_int; 5], /* Check out /usr/include/sys/door.h */
    }

    #[cfg_attr(feature = "extra_traits", allow(missing_debug_implementations))]
    pub struct door_desc_t {
        pub d_attributes: door_attr_t,
        pub d_data: door_desc_t__d_data,
    }

    #[cfg_attr(feature = "extra_traits", allow(missing_debug_implementations))]
    pub struct door_arg_t {
        pub data_ptr: *const ::c_char,
        pub data_size: ::size_t,
        pub desc_ptr: *const door_desc_t,
        pub dec_num: ::c_uint,
        pub rbuf: *const ::c_char,
        pub rsize: ::size_t,
    }
}

pub const PORT_SOURCE_POSTWAIT: ::c_int = 8;
pub const PORT_SOURCE_SIGNAL: ::c_int = 9;

pub const AF_LOCAL: ::c_int = 0;
pub const AF_FILE: ::c_int = 0;

pub const TCP_KEEPIDLE: ::c_int = 0x1d;
pub const TCP_KEEPINTVL: ::c_int = 0x1e;
pub const TCP_KEEPCNT: ::c_int = 0x1f;

pub const F_DUPFD_CLOEXEC: ::c_int = 47;
pub const F_DUPFD_CLOFORK: ::c_int = 49;
pub const F_DUP2FD_CLOEXEC: ::c_int = 48;
pub const F_DUP2FD_CLOFORK: ::c_int = 50;

extern "C" {
    pub fn fexecve(
        fd: ::c_int,
        argv: *const *const ::c_char,
        envp: *const *const ::c_char,
    ) -> ::c_int;

    pub fn mincore(addr: *const ::c_void, len: ::size_t, vec: *mut ::c_char) -> ::c_int;

    pub fn door_call(d: ::c_int, params: *const door_arg_t) -> ::c_int;
    pub fn door_return(
        data_ptr: *const ::c_char,
        data_size: ::size_t,
        desc_ptr: *const door_desc_t,
        num_desc: ::c_uint,
    );
    pub fn door_create(
        server_procedure: extern "C" fn(
            cookie: *const ::c_void,
            argp: *const ::c_char,
            arg_size: ::size_t,
            dp: *const door_desc_t,
            n_desc: ::c_uint,
        ),
        cookie: *const ::c_void,
        attributes: door_attr_t,
    ) -> ::c_int;

    pub fn fattach(fildes: ::c_int, path: *const ::c_char) -> ::c_int;

    pub fn pthread_getattr_np(thread: ::pthread_t, attr: *mut ::pthread_attr_t) -> ::c_int;

    pub fn euidaccess(path: *const ::c_char, amode: ::c_int) -> ::c_int;
}
