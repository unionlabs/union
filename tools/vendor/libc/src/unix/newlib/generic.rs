//! Common types used by most newlib platforms

s! {
    pub struct sigset_t {
        __val: [::c_ulong; 16],
    }

    pub struct stat {
        pub st_dev: ::dev_t,
        pub st_ino: ::ino_t,
        pub st_mode: ::mode_t,
        pub st_nlink: ::nlink_t,
        pub st_uid: ::uid_t,
        pub st_gid: ::gid_t,
        pub st_rdev: ::dev_t,
        pub st_size: ::off_t,
        pub st_atime: ::time_t,
        pub st_spare1: ::c_long,
        pub st_mtime: ::time_t,
        pub st_spare2: ::c_long,
        pub st_ctime: ::time_t,
        pub st_spare3: ::c_long,
        pub st_blksize: ::blksize_t,
        pub st_blocks: ::blkcnt_t,
        pub st_spare4: [::c_long; 2usize],
    }
}
