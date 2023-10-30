pub type c_long = i64;
pub type c_ulong = u64;

s! {
    pub struct sigset_t {
        pub ss_set: [c_ulong; 4],
    }

    pub struct fd_set {
        pub fds_bits: [c_long; 1024],
    }

    pub struct flock {
        pub l_type: ::c_short,
        pub l_whence: ::c_short,
        pub l_sysid: ::c_uint,
        pub l_pid: ::pid_t,
        pub l_vfs: ::c_int,
        pub l_start: ::off_t,
        pub l_len: ::off_t,
    }

    pub struct statvfs {
        pub f_bsize: ::c_ulong,
        pub f_frsize: ::c_ulong,
        pub f_blocks: ::fsblkcnt_t,
        pub f_bfree: ::fsblkcnt_t,
        pub f_bavail: ::fsblkcnt_t,
        pub f_files: ::fsfilcnt_t,
        pub f_ffree: ::fsfilcnt_t,
        pub f_favail: ::fsfilcnt_t,
        pub f_fsid: ::c_ulong,
        pub f_basetype: [::c_char; 16],
        pub f_flag: ::c_ulong,
        pub f_namemax: ::c_ulong,
        pub f_fstr: [::c_char; 32],
        pub f_filler: [::c_ulong; 16]
    }

    pub struct pthread_rwlock_t {
        __rw_word: [::c_long; 10],
    }

    pub struct pthread_cond_t {
        __cv_word: [::c_long; 6],
    }

    pub struct pthread_mutex_t {
        __mt_word: [::c_long; 8],
    }

    pub struct stat {
        pub st_dev: ::dev_t,
        pub st_ino: ::ino_t,
        pub st_mode: ::mode_t,
        pub st_nlink: ::nlink_t,
        pub st_flag: ::c_ushort,
        pub st_uid: ::uid_t,
        pub st_gid: ::gid_t,
        pub st_rdev: ::dev_t,
        pub st_ssize: ::c_int,
        pub st_atime: ::st_timespec,
        pub st_mtime: ::st_timespec,
        pub st_ctime: ::st_timespec,
        pub st_blksize: ::blksize_t,
        pub st_blocks: ::blkcnt_t,
        pub st_vfstype: ::c_int,
        pub st_vfs: ::c_uint,
        pub st_type: ::c_uint,
        pub st_gen: ::c_uint,
        pub st_reserved: [::c_uint; 9],
        pub st_padto_ll: ::c_uint,
        pub st_size: ::off_t,
    }

    pub struct statfs {
        pub f_version: ::c_int,
        pub f_type: ::c_int,
        pub f_bsize: ::c_ulong,
        pub f_blocks: ::fsblkcnt_t,
        pub f_bfree: ::fsblkcnt_t,
        pub f_bavail: ::fsblkcnt_t,
        pub f_files: ::fsblkcnt_t,
        pub f_ffree: ::fsblkcnt_t,
        pub f_fsid: ::fsid64_t,
        pub f_vfstype: ::c_int,
        pub f_fsize: ::c_ulong,
        pub f_vfsnumber: ::c_int,
        pub f_vfsoff: ::c_int,
        pub f_vfslen: ::c_int,
        pub f_vfsvers: ::c_int,
        pub f_fname: [::c_char; 32],
        pub f_fpack: [::c_char; 32],
        pub f_name_max: ::c_int,
    }

    pub struct aiocb {
        pub aio_lio_opcode: ::c_int,
        pub aio_fildes: ::c_int,
        pub aio_word1: ::c_int,
        pub aio_offset: ::off_t,
        pub aio_buf: *mut ::c_void,
        pub aio_return: ::ssize_t,
        pub aio_errno: ::c_int,
        pub aio_nbytes: ::size_t,
        pub aio_reqprio: ::c_int,
        pub aio_sigevent: ::sigevent,
        pub aio_word2: ::c_int,
        pub aio_fp: ::c_int,
        pub aio_handle: *mut aiocb,
        pub aio_reserved: [::c_uint; 2],
        pub aio_sigev_tid: c_long,
    }

    pub struct ucontext_t {
        pub __sc_onstack: ::c_int,
        pub uc_sigmask: ::sigset_t,
        pub __sc_uerror: ::c_int,
        pub uc_mcontext: ::mcontext_t,
        pub uc_link: *mut ucontext_t,
        pub uc_stack: ::stack_t,
        // Should be pointer to __extctx_t
        pub __extctx: *mut ::c_void,
        pub __extctx_magic: ::c_int,
        pub __pad: [::c_int; 1],
    }

    pub struct mcontext_t {
        pub gpr: [::c_ulonglong; 32],
        pub msr: ::c_ulonglong,
        pub iar: ::c_ulonglong,
        pub lr: ::c_ulonglong,
        pub ctr: ::c_ulonglong,
        pub cr: ::c_uint,
        pub xer: ::c_uint,
        pub fpscr: ::c_uint,
        pub fpscrx: ::c_uint,
        pub except: [::c_ulonglong; 1],
        // Should be array of double type
        pub fpr: [::uint64_t; 32],
        pub fpeu: ::c_char,
        pub fpinfo: ::c_char,
        pub fpscr24_31: ::c_char,
        pub pad: [::c_char; 1],
        pub excp_type: ::c_int,
    }

    pub struct utmpx {
        pub ut_user: [::c_char; 256],
        pub ut_id: [::c_char; 14],
        pub ut_line: [::c_char; 64],
        pub ut_pid: ::pid_t,
        pub ut_type: ::c_short,
        pub ut_tv: ::timeval,
        pub ut_host: [::c_char; 256],
        pub __dbl_word_pad: ::c_int,
        pub __reservedA: [::c_int; 2],
        pub __reservedV: [::c_int; 6],
    }

    pub struct pthread_spinlock_t {
        pub __sp_word: [::c_long; 3],
    }

    pub struct pthread_barrier_t {
        pub __br_word: [::c_long; 5],
    }

    pub struct msqid_ds {
        pub msg_perm: ::ipc_perm,
        pub msg_first: ::c_uint,
        pub msg_last: ::c_uint,
        pub msg_cbytes: ::c_uint,
        pub msg_qnum: ::c_uint,
        pub msg_qbytes: ::c_ulong,
        pub msg_lspid: ::pid_t,
        pub msg_lrpid: ::pid_t,
        pub msg_stime: ::time_t,
        pub msg_rtime: ::time_t,
        pub msg_ctime: ::time_t,
        pub msg_rwait: ::c_int,
        pub msg_wwait: ::c_int,
        pub msg_reqevents: ::c_ushort,
    }
}

s_no_extra_traits! {
    pub struct siginfo_t {
        pub si_signo: ::c_int,
        pub si_errno: ::c_int,
        pub si_code: ::c_int,
        pub si_pid: ::pid_t,
        pub si_uid: ::uid_t,
        pub si_status: ::c_int,
        pub si_addr: *mut ::c_void,
        pub si_band: ::c_long,
        pub si_value: ::sigval,
        pub __si_flags: ::c_int,
        pub __pad: [::c_int; 3],
    }

    #[cfg(libc_union)]
    pub union _kernel_simple_lock {
        pub _slock: ::c_long,
        // Should be pointer to 'lock_data_instrumented'
        pub _slockp: *mut ::c_void,
    }

    pub struct fileops_t {
        pub fo_rw: extern fn(file: *mut file, rw: ::uio_rw, io: *mut ::c_void, ext: ::c_long,
                             secattr: *mut ::c_void) -> ::c_int,
        pub fo_ioctl: extern fn(file: *mut file, a: ::c_long, b: ::caddr_t, c: ::c_long,
                                d: ::c_long) -> ::c_int,
        pub fo_select: extern fn(file: *mut file, a: ::c_int, b: *mut ::c_ushort,
                                 c: extern fn()) -> ::c_int,
        pub fo_close: extern fn(file: *mut file) -> ::c_int,
        pub fo_fstat: extern fn(file: *mut file, sstat: *mut ::stat) -> ::c_int,
    }

    pub struct file {
        pub f_flag: ::c_long,
        pub f_count: ::c_int,
        pub f_options: ::c_short,
        pub f_type: ::c_short,
        // Should be pointer to 'vnode'
        pub f_data: *mut ::c_void,
        pub f_offset: ::c_longlong,
        pub f_dir_off: ::c_long,
        // Should be pointer to 'cred'
        pub f_cred: *mut ::c_void,
        #[cfg(libc_union)]
        pub f_lock: _kernel_simple_lock,
        #[cfg(libc_union)]
        pub f_offset_lock: _kernel_simple_lock,
        pub f_vinfo: ::caddr_t,
        pub f_ops: *mut fileops_t,
        pub f_parentp: ::caddr_t,
        pub f_fnamep: ::caddr_t,
        pub f_fdata: [::c_char; 160],
    }

    #[cfg(libc_union)]
    pub union __ld_info_file {
        pub _ldinfo_fd: ::c_int,
        pub _ldinfo_fp: *mut file,
        pub _core_offset: ::c_long,
    }

    pub struct ld_info {
        pub ldinfo_next: ::c_uint,
        pub ldinfo_flags: ::c_uint,
        #[cfg(libc_union)]
        pub _file: __ld_info_file,
        pub ldinfo_textorg: *mut ::c_void,
        pub ldinfo_textsize: ::c_ulong,
        pub ldinfo_dataorg: *mut ::c_void,
        pub ldinfo_datasize: ::c_ulong,
        pub ldinfo_filename: [::c_char; 2],
    }

    #[cfg(libc_union)]
    pub union __pollfd_ext_u {
        pub addr: *mut ::c_void,
        pub data32: u32,
        pub data: u64,
    }

    pub struct pollfd_ext {
        pub fd: ::c_int,
        pub events: ::c_ushort,
        pub revents: ::c_ushort,
        #[cfg(libc_union)]
        pub data: __pollfd_ext_u,
    }
}

impl siginfo_t {
    pub unsafe fn si_addr(&self) -> *mut ::c_void {
        self.si_addr
    }

    pub unsafe fn si_value(&self) -> ::sigval {
        self.si_value
    }

    pub unsafe fn si_pid(&self) -> ::pid_t {
        self.si_pid
    }

    pub unsafe fn si_uid(&self) -> ::uid_t {
        self.si_uid
    }

    pub unsafe fn si_status(&self) -> ::c_int {
        self.si_status
    }
}

cfg_if! {
    if #[cfg(feature = "extra_traits")] {
        impl PartialEq for siginfo_t {
            fn eq(&self, other: &siginfo_t) -> bool {
                #[cfg(libc_union)]
                let value_eq = self.si_value == other.si_value;
                #[cfg(not(libc_union))]
                let value_eq = true;
                self.si_signo == other.si_signo
                    && self.si_errno == other.si_errno
                    && self.si_code == other.si_code
                    && self.si_pid == other.si_pid
                    && self.si_uid == other.si_uid
                    && self.si_status == other.si_status
                    && self.si_addr == other.si_addr
                    && self.si_band == other.si_band
                    && self.__si_flags == other.__si_flags
                    && value_eq
            }
        }
        impl Eq for siginfo_t {}
        impl ::fmt::Debug for siginfo_t {
            fn fmt(&self, f: &mut ::fmt::Formatter) -> ::fmt::Result {
                let mut struct_formatter = f.debug_struct("siginfo_t");
                struct_formatter.field("si_signo", &self.si_signo);
                struct_formatter.field("si_errno", &self.si_errno);
                struct_formatter.field("si_code", &self.si_code);
                struct_formatter.field("si_pid", &self.si_pid);
                struct_formatter.field("si_uid", &self.si_uid);
                struct_formatter.field("si_status", &self.si_status);
                struct_formatter.field("si_addr", &self.si_addr);
                struct_formatter.field("si_band", &self.si_band);
                #[cfg(libc_union)]
                struct_formatter.field("si_value", &self.si_value);
                struct_formatter.field("__si_flags", &self.__si_flags);
                struct_formatter.finish()
            }
        }
        impl ::hash::Hash for siginfo_t {
            fn hash<H: ::hash::Hasher>(&self, state: &mut H) {
                self.si_signo.hash(state);
                self.si_errno.hash(state);
                self.si_code.hash(state);
                self.si_pid.hash(state);
                self.si_uid.hash(state);
                self.si_status.hash(state);
                self.si_addr.hash(state);
                self.si_band.hash(state);
                #[cfg(libc_union)]
                self.si_value.hash(state);
                self.__si_flags.hash(state);
            }
        }

        #[cfg(libc_union)]
        impl PartialEq for _kernel_simple_lock {
            fn eq(&self, other: &_kernel_simple_lock) -> bool {
                unsafe {
                    self._slock == other._slock
                        && self._slockp == other._slockp
                }
            }
        }
        #[cfg(libc_union)]
        impl Eq for _kernel_simple_lock {}
        #[cfg(libc_union)]
        impl ::fmt::Debug for _kernel_simple_lock {
            fn fmt(&self, f: &mut ::fmt::Formatter) -> ::fmt::Result {
                f.debug_struct("_kernel_simple_lock")
                    .field("_slock", unsafe { &self._slock })
                    .field("_slockp", unsafe { &self._slockp })
                    .finish()
            }
        }
        #[cfg(libc_union)]
        impl ::hash::Hash for _kernel_simple_lock {
            fn hash<H: ::hash::Hasher>(&self, state: &mut H) {
                unsafe {
                    self._slock.hash(state);
                    self._slockp.hash(state);
                }
            }
        }

        impl PartialEq for fileops_t {
            fn eq(&self, other: &fileops_t) -> bool {
                self.fo_rw == other.fo_rw
                    && self.fo_ioctl == other.fo_ioctl
                    && self.fo_select == other.fo_select
                    && self.fo_close == other.fo_close
                    && self.fo_fstat == other.fo_fstat
            }
        }
        impl Eq for fileops_t {}
        impl ::fmt::Debug for fileops_t {
            fn fmt(&self, f: &mut ::fmt::Formatter) -> ::fmt::Result {
                let mut struct_formatter = f.debug_struct("fileops_t");
                struct_formatter.field("fo_rw", &self.fo_rw);
                struct_formatter.field("fo_ioctl", &self.fo_ioctl);
                struct_formatter.field("fo_select", &self.fo_select);
                struct_formatter.field("fo_close", &self.fo_close);
                struct_formatter.field("fo_fstat", &self.fo_fstat);
                struct_formatter.finish()
            }
        }
        impl ::hash::Hash for fileops_t {
            fn hash<H: ::hash::Hasher>(&self, state: &mut H) {
                self.fo_rw.hash(state);
                self.fo_ioctl.hash(state);
                self.fo_select.hash(state);
                self.fo_close.hash(state);
                self.fo_fstat.hash(state);
            }
        }

        impl PartialEq for file {
            fn eq(&self, other: &file) -> bool {
                #[cfg(libc_union)]
                let lock_eq = self.f_lock == other.f_lock
                    && self.f_offset_lock == other.f_offset_lock;
                #[cfg(not(libc_union))]
                let lock_eq = true;
                self.f_flag == other.f_flag
                    && self.f_count == other.f_count
                    && self.f_options == other.f_options
                    && self.f_type == other.f_type
                    && self.f_data == other.f_data
                    && self.f_offset == other.f_offset
                    && self.f_dir_off == other.f_dir_off
                    && self.f_cred == other.f_cred
                    && self.f_vinfo == other.f_vinfo
                    && self.f_ops == other.f_ops
                    && self.f_parentp == other.f_parentp
                    && self.f_fnamep == other.f_fnamep
                    && self.f_fdata == other.f_fdata
                    && lock_eq
            }
        }
        impl Eq for file {}
        impl ::fmt::Debug for file {
            fn fmt(&self, f: &mut ::fmt::Formatter) -> ::fmt::Result {
                let mut struct_formatter = f.debug_struct("file");
                struct_formatter.field("f_flag", &self.f_flag);
                struct_formatter.field("f_count", &self.f_count);
                struct_formatter.field("f_options", &self.f_options);
                struct_formatter.field("f_type", &self.f_type);
                struct_formatter.field("f_data", &self.f_data);
                struct_formatter.field("f_offset", &self.f_offset);
                struct_formatter.field("f_dir_off", &self.f_dir_off);
                struct_formatter.field("f_cred", &self.f_cred);
                #[cfg(libc_union)]
                struct_formatter.field("f_lock", &self.f_lock);
                #[cfg(libc_union)]
                struct_formatter.field("f_offset_lock", &self.f_offset_lock);
                struct_formatter.field("f_vinfo", &self.f_vinfo);
                struct_formatter.field("f_ops", &self.f_ops);
                struct_formatter.field("f_parentp", &self.f_parentp);
                struct_formatter.field("f_fnamep", &self.f_fnamep);
                struct_formatter.field("f_fdata", &self.f_fdata);
                struct_formatter.finish()
            }
        }
        impl ::hash::Hash for file {
            fn hash<H: ::hash::Hasher>(&self, state: &mut H) {
                self.f_flag.hash(state);
                self.f_count.hash(state);
                self.f_options.hash(state);
                self.f_type.hash(state);
                self.f_data.hash(state);
                self.f_offset.hash(state);
                self.f_dir_off.hash(state);
                self.f_cred.hash(state);
                #[cfg(libc_union)]
                self.f_lock.hash(state);
                #[cfg(libc_union)]
                self.f_offset_lock.hash(state);
                self.f_vinfo.hash(state);
                self.f_ops.hash(state);
                self.f_parentp.hash(state);
                self.f_fnamep.hash(state);
                self.f_fdata.hash(state);
            }
        }

        #[cfg(libc_union)]
        impl PartialEq for __ld_info_file {
            fn eq(&self, other: &__ld_info_file) -> bool {
                unsafe {
                    self._ldinfo_fd == other._ldinfo_fd
                        && self._ldinfo_fp == other._ldinfo_fp
                        && self._core_offset == other._core_offset
                }
            }
        }
        #[cfg(libc_union)]
        impl Eq for __ld_info_file {}
        #[cfg(libc_union)]
        impl ::fmt::Debug for __ld_info_file {
            fn fmt(&self, f: &mut ::fmt::Formatter) -> ::fmt::Result {
                f.debug_struct("__ld_info_file")
                    .field("_ldinfo_fd", unsafe { &self._ldinfo_fd })
                    .field("_ldinfo_fp", unsafe { &self._ldinfo_fp })
                    .field("_core_offset", unsafe { &self._core_offset })
                    .finish()
            }
        }
        #[cfg(libc_union)]
        impl ::hash::Hash for __ld_info_file {
            fn hash<H: ::hash::Hasher>(&self, state: &mut H) {
                unsafe {
                    self._ldinfo_fd.hash(state);
                    self._ldinfo_fp.hash(state);
                    self._core_offset.hash(state);
                }
            }
        }

        impl PartialEq for ld_info {
            fn eq(&self, other: &ld_info) -> bool {
                #[cfg(libc_union)]
                let file_eq = self._file == other._file;
                #[cfg(not(libc_union))]
                let file_eq = true;
                self.ldinfo_next == other.ldinfo_next
                    && self.ldinfo_flags == other.ldinfo_flags
                    && self.ldinfo_textorg == other.ldinfo_textorg
                    && self.ldinfo_textsize == other.ldinfo_textsize
                    && self.ldinfo_dataorg == other.ldinfo_dataorg
                    && self.ldinfo_datasize == other.ldinfo_datasize
                    && self.ldinfo_filename == other.ldinfo_filename
                    && file_eq
            }
        }
        impl Eq for ld_info {}
        impl ::fmt::Debug for ld_info {
            fn fmt(&self, f: &mut ::fmt::Formatter) -> ::fmt::Result {
                let mut struct_formatter = f.debug_struct("ld_info");
                struct_formatter.field("ldinfo_next", &self.ldinfo_next);
                struct_formatter.field("ldinfo_flags", &self.ldinfo_flags);
                struct_formatter.field("ldinfo_textorg", &self.ldinfo_textorg);
                struct_formatter.field("ldinfo_textsize", &self.ldinfo_textsize);
                struct_formatter.field("ldinfo_dataorg", &self.ldinfo_dataorg);
                struct_formatter.field("ldinfo_datasize", &self.ldinfo_datasize);
                struct_formatter.field("ldinfo_filename", &self.ldinfo_filename);
                #[cfg(libc_union)]
                struct_formatter.field("_file", &self._file);
                struct_formatter.finish()
            }
        }
        impl ::hash::Hash for ld_info {
            fn hash<H: ::hash::Hasher>(&self, state: &mut H) {
                self.ldinfo_next.hash(state);
                self.ldinfo_flags.hash(state);
                self.ldinfo_textorg.hash(state);
                self.ldinfo_textsize.hash(state);
                self.ldinfo_dataorg.hash(state);
                self.ldinfo_datasize.hash(state);
                self.ldinfo_filename.hash(state);
                #[cfg(libc_union)]
                self._file.hash(state);
            }
        }

        #[cfg(libc_union)]
        impl PartialEq for __pollfd_ext_u {
            fn eq(&self, other: &__pollfd_ext_u) -> bool {
                unsafe {
                    self.addr == other.addr
                        && self.data32 == other.data32
                        && self.data == other.data
                }
            }
        }
        #[cfg(libc_union)]
        impl Eq for __pollfd_ext_u {}
        #[cfg(libc_union)]
        impl ::fmt::Debug for __pollfd_ext_u {
            fn fmt(&self, f: &mut ::fmt::Formatter) -> ::fmt::Result {
                f.debug_struct("__pollfd_ext_u")
                    .field("addr", unsafe { &self.addr })
                    .field("data32", unsafe { &self.data32 })
                    .field("data", unsafe { &self.data })
                    .finish()
            }
        }
        #[cfg(libc_union)]
        impl ::hash::Hash for __pollfd_ext_u {
            fn hash<H: ::hash::Hasher>(&self, state: &mut H) {
                unsafe {
                    self.addr.hash(state);
                    self.data.hash(state);
                    self.data32.hash(state);
                }
            }
        }

        impl PartialEq for pollfd_ext {
            fn eq(&self, other: &pollfd_ext) -> bool {
                #[cfg(libc_union)]
                let data_eq = self.data == other.data;
                #[cfg(not(libc_union))]
                let data_eq = true;
                self.fd == other.fd
                    && self.events == other.events
                    && self.revents == other.revents
                    && data_eq
            }
        }
        impl Eq for pollfd_ext {}
        impl ::fmt::Debug for pollfd_ext {
            fn fmt(&self, f: &mut ::fmt::Formatter) -> ::fmt::Result {
                let mut struct_formatter = f.debug_struct("pollfd_ext");
                struct_formatter.field("fd", &self.fd);
                struct_formatter.field("events", &self.events);
                struct_formatter.field("revents", &self.revents);
                #[cfg(libc_union)]
                struct_formatter.field("data", &self.data);
                struct_formatter.finish()
            }
        }
        impl ::hash::Hash for pollfd_ext {
            fn hash<H: ::hash::Hasher>(&self, state: &mut H) {
                self.fd.hash(state);
                self.events.hash(state);
                self.revents.hash(state);
                #[cfg(libc_union)]
                self.data.hash(state);
            }
        }
    }
}

pub const PTHREAD_MUTEX_INITIALIZER: pthread_mutex_t = pthread_mutex_t {
    __mt_word: [0, 2, 0, 0, 0, 0, 0, 0],
};
pub const PTHREAD_COND_INITIALIZER: pthread_cond_t = pthread_cond_t {
    __cv_word: [0, 0, 0, 0, 2, 0],
};
pub const PTHREAD_RWLOCK_INITIALIZER: pthread_rwlock_t = pthread_rwlock_t {
    __rw_word: [2, 0, 0, 0, 0, 0, 0, 0, 0, 0],
};
pub const RLIM_INFINITY: ::c_ulong = 0x7fffffffffffffff;

extern "C" {
    pub fn getsystemcfg(label: ::c_int) -> ::c_ulong;
}
