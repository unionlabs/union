pub type pthread_t = *mut ::c_void;
pub type clock_t = c_long;
#[cfg_attr(
    not(feature = "rustc-dep-of-std"),
    deprecated(
        since = "0.2.80",
        note = "This type is changed to 64-bit in musl 1.2.0, \
                we'll follow that change in the future release. \
                See #1848 for more info."
    )
)]
pub type time_t = c_long;
pub type suseconds_t = c_long;
pub type ino_t = u64;
pub type off_t = i64;
pub type blkcnt_t = i64;

pub type shmatt_t = ::c_ulong;
pub type msgqnum_t = ::c_ulong;
pub type msglen_t = ::c_ulong;
pub type fsblkcnt_t = ::c_ulonglong;
pub type fsfilcnt_t = ::c_ulonglong;
pub type rlim_t = ::c_ulonglong;

cfg_if! {
    if #[cfg(doc)] {
        // Used in `linux::arch` to define ioctl constants.
        pub(crate) type Ioctl = ::c_int;
    } else {
        #[doc(hidden)]
        pub type Ioctl = ::c_int;
    }
}

impl siginfo_t {
    pub unsafe fn si_addr(&self) -> *mut ::c_void {
        #[repr(C)]
        struct siginfo_sigfault {
            _si_signo: ::c_int,
            _si_errno: ::c_int,
            _si_code: ::c_int,
            si_addr: *mut ::c_void,
        }
        (*(self as *const siginfo_t as *const siginfo_sigfault)).si_addr
    }

    pub unsafe fn si_value(&self) -> ::sigval {
        #[repr(C)]
        struct siginfo_si_value {
            _si_signo: ::c_int,
            _si_errno: ::c_int,
            _si_code: ::c_int,
            _si_timerid: ::c_int,
            _si_overrun: ::c_int,
            si_value: ::sigval,
        }
        (*(self as *const siginfo_t as *const siginfo_si_value)).si_value
    }
}

cfg_if! {
    if #[cfg(libc_union)] {
        // Internal, for casts to access union fields
        #[repr(C)]
        struct sifields_sigchld {
            si_pid: ::pid_t,
            si_uid: ::uid_t,
            si_status: ::c_int,
            si_utime: ::c_long,
            si_stime: ::c_long,
        }
        impl ::Copy for sifields_sigchld {}
        impl ::Clone for sifields_sigchld {
            fn clone(&self) -> sifields_sigchld {
                *self
            }
        }

        // Internal, for casts to access union fields
        #[repr(C)]
        union sifields {
            _align_pointer: *mut ::c_void,
            sigchld: sifields_sigchld,
        }

        // Internal, for casts to access union fields. Note that some variants
        // of sifields start with a pointer, which makes the alignment of
        // sifields vary on 32-bit and 64-bit architectures.
        #[repr(C)]
        struct siginfo_f {
            _siginfo_base: [::c_int; 3],
            sifields: sifields,
        }

        impl siginfo_t {
            unsafe fn sifields(&self) -> &sifields {
                &(*(self as *const siginfo_t as *const siginfo_f)).sifields
            }

            pub unsafe fn si_pid(&self) -> ::pid_t {
                self.sifields().sigchld.si_pid
            }

            pub unsafe fn si_uid(&self) -> ::uid_t {
                self.sifields().sigchld.si_uid
            }

            pub unsafe fn si_status(&self) -> ::c_int {
                self.sifields().sigchld.si_status
            }

            pub unsafe fn si_utime(&self) -> ::c_long {
                self.sifields().sigchld.si_utime
            }

            pub unsafe fn si_stime(&self) -> ::c_long {
                self.sifields().sigchld.si_stime
            }
        }
    }
}

s! {
    pub struct aiocb {
        pub aio_fildes: ::c_int,
        pub aio_lio_opcode: ::c_int,
        pub aio_reqprio: ::c_int,
        pub aio_buf: *mut ::c_void,
        pub aio_nbytes: ::size_t,
        pub aio_sigevent: ::sigevent,
        __td: *mut ::c_void,
        __lock: [::c_int; 2],
        __err: ::c_int,
        __ret: ::ssize_t,
        pub aio_offset: off_t,
        __next: *mut ::c_void,
        __prev: *mut ::c_void,
        #[cfg(target_pointer_width = "32")]
        __dummy4: [::c_char; 24],
        #[cfg(target_pointer_width = "64")]
        __dummy4: [::c_char; 16],
    }

    pub struct sigaction {
        pub sa_sigaction: ::sighandler_t,
        pub sa_mask: ::sigset_t,
        pub sa_flags: ::c_int,
        pub sa_restorer: ::Option<extern fn()>,
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
        #[cfg(target_endian = "little")]
        pub f_fsid: ::c_ulong,
        #[cfg(target_pointer_width = "32")]
        __f_unused: ::c_int,
        #[cfg(target_endian = "big")]
        pub f_fsid: ::c_ulong,
        pub f_flag: ::c_ulong,
        pub f_namemax: ::c_ulong,
        __f_spare: [::c_int; 6],
    }

    pub struct termios {
        pub c_iflag: ::tcflag_t,
        pub c_oflag: ::tcflag_t,
        pub c_cflag: ::tcflag_t,
        pub c_lflag: ::tcflag_t,
        pub c_line: ::cc_t,
        pub c_cc: [::cc_t; ::NCCS],
        pub __c_ispeed: ::speed_t,
        pub __c_ospeed: ::speed_t,
    }

    pub struct flock {
        pub l_type: ::c_short,
        pub l_whence: ::c_short,
        pub l_start: ::off_t,
        pub l_len: ::off_t,
        pub l_pid: ::pid_t,
    }

    pub struct flock64 {
        pub l_type: ::c_short,
        pub l_whence: ::c_short,
        pub l_start: ::off64_t,
        pub l_len: ::off64_t,
        pub l_pid: ::pid_t,
    }

    pub struct regex_t {
        __re_nsub: ::size_t,
        __opaque: *mut ::c_void,
        __padding: [*mut ::c_void; 4usize],
        __nsub2: ::size_t,
        __padding2: ::c_char,
    }

    pub struct rtentry {
        pub rt_pad1: ::c_ulong,
        pub rt_dst: ::sockaddr,
        pub rt_gateway: ::sockaddr,
        pub rt_genmask: ::sockaddr,
        pub rt_flags: ::c_ushort,
        pub rt_pad2: ::c_short,
        pub rt_pad3: ::c_ulong,
        pub rt_tos: ::c_uchar,
        pub rt_class: ::c_uchar,
        #[cfg(target_pointer_width = "64")]
        pub rt_pad4: [::c_short; 3usize],
        #[cfg(not(target_pointer_width = "64"))]
        pub rt_pad4: [::c_short; 1usize],
        pub rt_metric: ::c_short,
        pub rt_dev: *mut ::c_char,
        pub rt_mtu: ::c_ulong,
        pub rt_window: ::c_ulong,
        pub rt_irtt: ::c_ushort,
    }

    pub struct __exit_status {
        pub e_termination: ::c_short,
        pub e_exit: ::c_short,
    }

    pub struct Elf64_Chdr {
        pub ch_type: ::Elf64_Word,
        pub ch_reserved: ::Elf64_Word,
        pub ch_size: ::Elf64_Xword,
        pub ch_addralign: ::Elf64_Xword,
    }

    pub struct Elf32_Chdr {
        pub ch_type: ::Elf32_Word,
        pub ch_size: ::Elf32_Word,
        pub ch_addralign: ::Elf32_Word,
    }

    pub struct timex {
        pub modes: ::c_uint,
        pub offset: ::c_long,
        pub freq: ::c_long,
        pub maxerror: ::c_long,
        pub esterror: ::c_long,
        pub status: ::c_int,
        pub constant: ::c_long,
        pub precision: ::c_long,
        pub tolerance: ::c_long,
        pub time: ::timeval,
        pub tick: ::c_long,
        pub ppsfreq: ::c_long,
        pub jitter: ::c_long,
        pub shift: ::c_int,
        pub stabil: ::c_long,
        pub jitcnt: ::c_long,
        pub calcnt: ::c_long,
        pub errcnt: ::c_long,
        pub stbcnt: ::c_long,
        pub tai: ::c_int,
        pub __padding: [::c_int; 11],
    }

    pub struct ntptimeval {
        pub time: ::timeval,
        pub maxerror: ::c_long,
        pub esterror: ::c_long,
    }
}

s_no_extra_traits! {
    pub struct sysinfo {
        pub uptime: ::c_ulong,
        pub loads: [::c_ulong; 3],
        pub totalram: ::c_ulong,
        pub freeram: ::c_ulong,
        pub sharedram: ::c_ulong,
        pub bufferram: ::c_ulong,
        pub totalswap: ::c_ulong,
        pub freeswap: ::c_ulong,
        pub procs: ::c_ushort,
        pub pad: ::c_ushort,
        pub totalhigh: ::c_ulong,
        pub freehigh: ::c_ulong,
        pub mem_unit: ::c_uint,
        pub __reserved: [::c_char; 256],
    }

    // FIXME: musl added paddings and adjusted
    // layout in 1.2.0 but our CI is still 1.1.24.
    // So, I'm leaving some fields as cfg for now.
    // ref. https://github.com/bminor/musl/commit/
    // 1e7f0fcd7ff2096904fd93a2ee6d12a2392be392
    //
    // OpenHarmony uses the musl 1.2 layout.
    pub struct utmpx {
        pub ut_type: ::c_short,
        __ut_pad1: ::c_short,
        pub ut_pid: ::pid_t,
        pub ut_line: [::c_char; 32],
        pub ut_id: [::c_char; 4],
        pub ut_user: [::c_char; 32],
        pub ut_host: [::c_char; 256],
        pub ut_exit: __exit_status,

        #[cfg(target_env = "musl")]
        pub ut_session: ::c_long,

        #[cfg(target_env = "ohos")]
        #[cfg(target_endian = "little")]
        pub ut_session: ::c_int,
        #[cfg(target_env = "ohos")]
        #[cfg(target_endian = "little")]
        __ut_pad2: ::c_int,

        #[cfg(target_env = "ohos")]
        #[cfg(not(target_endian = "little"))]
        __ut_pad2: ::c_int,
        #[cfg(target_env = "ohos")]
        #[cfg(not(target_endian = "little"))]
        pub ut_session: ::c_int,

        pub ut_tv: ::timeval,
        pub ut_addr_v6: [::c_uint; 4],
        __unused: [::c_char; 20],
    }
}

cfg_if! {
    if #[cfg(feature = "extra_traits")] {
        impl PartialEq for sysinfo {
            fn eq(&self, other: &sysinfo) -> bool {
                self.uptime == other.uptime
                    && self.loads == other.loads
                    && self.totalram == other.totalram
                    && self.freeram == other.freeram
                    && self.sharedram == other.sharedram
                    && self.bufferram == other.bufferram
                    && self.totalswap == other.totalswap
                    && self.freeswap == other.freeswap
                    && self.procs == other.procs
                    && self.pad == other.pad
                    && self.totalhigh == other.totalhigh
                    && self.freehigh == other.freehigh
                    && self.mem_unit == other.mem_unit
                    && self
                        .__reserved
                        .iter()
                        .zip(other.__reserved.iter())
                        .all(|(a,b)| a == b)
            }
        }

        impl Eq for sysinfo {}

        impl ::fmt::Debug for sysinfo {
            fn fmt(&self, f: &mut ::fmt::Formatter) -> ::fmt::Result {
                f.debug_struct("sysinfo")
                    .field("uptime", &self.uptime)
                    .field("loads", &self.loads)
                    .field("totalram", &self.totalram)
                    .field("freeram", &self.freeram)
                    .field("sharedram", &self.sharedram)
                    .field("bufferram", &self.bufferram)
                    .field("totalswap", &self.totalswap)
                    .field("freeswap", &self.freeswap)
                    .field("procs", &self.procs)
                    .field("pad", &self.pad)
                    .field("totalhigh", &self.totalhigh)
                    .field("freehigh", &self.freehigh)
                    .field("mem_unit", &self.mem_unit)
                    // FIXME: .field("__reserved", &self.__reserved)
                    .finish()
            }
        }

        impl ::hash::Hash for sysinfo {
            fn hash<H: ::hash::Hasher>(&self, state: &mut H) {
                self.uptime.hash(state);
                self.loads.hash(state);
                self.totalram.hash(state);
                self.freeram.hash(state);
                self.sharedram.hash(state);
                self.bufferram.hash(state);
                self.totalswap.hash(state);
                self.freeswap.hash(state);
                self.procs.hash(state);
                self.pad.hash(state);
                self.totalhigh.hash(state);
                self.freehigh.hash(state);
                self.mem_unit.hash(state);
                self.__reserved.hash(state);
            }
        }

        impl PartialEq for utmpx {
            fn eq(&self, other: &utmpx) -> bool {
                self.ut_type == other.ut_type
                    //&& self.__ut_pad1 == other.__ut_pad1
                    && self.ut_pid == other.ut_pid
                    && self.ut_line == other.ut_line
                    && self.ut_id == other.ut_id
                    && self.ut_user == other.ut_user
                    && self
                        .ut_host
                        .iter()
                        .zip(other.ut_host.iter())
                        .all(|(a,b)| a == b)
                    && self.ut_exit == other.ut_exit
                    && self.ut_session == other.ut_session
                    //&& self.__ut_pad2 == other.__ut_pad2
                    && self.ut_tv == other.ut_tv
                    && self.ut_addr_v6 == other.ut_addr_v6
                    && self.__unused == other.__unused
            }
        }

        impl Eq for utmpx {}

        impl ::fmt::Debug for utmpx {
            fn fmt(&self, f: &mut ::fmt::Formatter) -> ::fmt::Result {
                f.debug_struct("utmpx")
                    .field("ut_type", &self.ut_type)
                    //.field("__ut_pad1", &self.__ut_pad1)
                    .field("ut_pid", &self.ut_pid)
                    .field("ut_line", &self.ut_line)
                    .field("ut_id", &self.ut_id)
                    .field("ut_user", &self.ut_user)
                    //FIXME: .field("ut_host", &self.ut_host)
                    .field("ut_exit", &self.ut_exit)
                    .field("ut_session", &self.ut_session)
                    //.field("__ut_pad2", &self.__ut_pad2)
                    .field("ut_tv", &self.ut_tv)
                    .field("ut_addr_v6", &self.ut_addr_v6)
                    .field("__unused", &self.__unused)
                    .finish()
            }
        }

        impl ::hash::Hash for utmpx {
            fn hash<H: ::hash::Hasher>(&self, state: &mut H) {
                self.ut_type.hash(state);
                //self.__ut_pad1.hash(state);
                self.ut_pid.hash(state);
                self.ut_line.hash(state);
                self.ut_id.hash(state);
                self.ut_user.hash(state);
                self.ut_host.hash(state);
                self.ut_exit.hash(state);
                self.ut_session.hash(state);
                //self.__ut_pad2.hash(state);
                self.ut_tv.hash(state);
                self.ut_addr_v6.hash(state);
                self.__unused.hash(state);
            }
        }
    }
}

// include/sys/mman.h
/*
 * Huge page size encoding when MAP_HUGETLB is specified, and a huge page
 * size other than the default is desired.  See hugetlb_encode.h.
 * All known huge page size encodings are provided here.  It is the
 * responsibility of the application to know which sizes are supported on
 * the running system.  See mmap(2) man page for details.
 */
pub const MAP_HUGE_SHIFT: ::c_int = 26;
pub const MAP_HUGE_MASK: ::c_int = 0x3f;

pub const MAP_HUGE_64KB: ::c_int = 16 << MAP_HUGE_SHIFT;
pub const MAP_HUGE_512KB: ::c_int = 19 << MAP_HUGE_SHIFT;
pub const MAP_HUGE_1MB: ::c_int = 20 << MAP_HUGE_SHIFT;
pub const MAP_HUGE_2MB: ::c_int = 21 << MAP_HUGE_SHIFT;
pub const MAP_HUGE_8MB: ::c_int = 23 << MAP_HUGE_SHIFT;
pub const MAP_HUGE_16MB: ::c_int = 24 << MAP_HUGE_SHIFT;
pub const MAP_HUGE_32MB: ::c_int = 25 << MAP_HUGE_SHIFT;
pub const MAP_HUGE_256MB: ::c_int = 28 << MAP_HUGE_SHIFT;
pub const MAP_HUGE_512MB: ::c_int = 29 << MAP_HUGE_SHIFT;
pub const MAP_HUGE_1GB: ::c_int = 30 << MAP_HUGE_SHIFT;
pub const MAP_HUGE_2GB: ::c_int = 31 << MAP_HUGE_SHIFT;
pub const MAP_HUGE_16GB: ::c_int = 34 << MAP_HUGE_SHIFT;

pub const MS_RMT_MASK: ::c_ulong = 0x02800051;

pub const SFD_CLOEXEC: ::c_int = 0x080000;

pub const NCCS: usize = 32;

pub const O_TRUNC: ::c_int = 512;
pub const O_NOATIME: ::c_int = 0o1000000;
pub const O_CLOEXEC: ::c_int = 0x80000;
pub const O_TMPFILE: ::c_int = 0o20000000 | O_DIRECTORY;

pub const EBFONT: ::c_int = 59;
pub const ENOSTR: ::c_int = 60;
pub const ENODATA: ::c_int = 61;
pub const ETIME: ::c_int = 62;
pub const ENOSR: ::c_int = 63;
pub const ENONET: ::c_int = 64;
pub const ENOPKG: ::c_int = 65;
pub const EREMOTE: ::c_int = 66;
pub const ENOLINK: ::c_int = 67;
pub const EADV: ::c_int = 68;
pub const ESRMNT: ::c_int = 69;
pub const ECOMM: ::c_int = 70;
pub const EPROTO: ::c_int = 71;
pub const EDOTDOT: ::c_int = 73;

pub const F_RDLCK: ::c_int = 0;
pub const F_WRLCK: ::c_int = 1;
pub const F_UNLCK: ::c_int = 2;

pub const SA_NODEFER: ::c_int = 0x40000000;
pub const SA_RESETHAND: ::c_int = 0x80000000;
pub const SA_RESTART: ::c_int = 0x10000000;
pub const SA_NOCLDSTOP: ::c_int = 0x00000001;

pub const EPOLL_CLOEXEC: ::c_int = 0x80000;

pub const EFD_CLOEXEC: ::c_int = 0x80000;

pub const BUFSIZ: ::c_uint = 1024;
pub const TMP_MAX: ::c_uint = 10000;
pub const FOPEN_MAX: ::c_uint = 1000;
pub const FILENAME_MAX: ::c_uint = 4096;
pub const O_PATH: ::c_int = 0o10000000;
pub const O_EXEC: ::c_int = 0o10000000;
pub const O_SEARCH: ::c_int = 0o10000000;
pub const O_ACCMODE: ::c_int = 0o10000003;
pub const O_NDELAY: ::c_int = O_NONBLOCK;
pub const NI_MAXHOST: ::socklen_t = 255;
pub const PTHREAD_STACK_MIN: ::size_t = 2048;

pub const POSIX_MADV_DONTNEED: ::c_int = 4;

pub const MAP_ANONYMOUS: ::c_int = MAP_ANON;

pub const SOCK_DCCP: ::c_int = 6;
pub const SOCK_PACKET: ::c_int = 10;

pub const SOMAXCONN: ::c_int = 128;

#[deprecated(since = "0.2.55", note = "Use SIGSYS instead")]
pub const SIGUNUSED: ::c_int = ::SIGSYS;

pub const __SIZEOF_PTHREAD_CONDATTR_T: usize = 4;
pub const __SIZEOF_PTHREAD_MUTEXATTR_T: usize = 4;
pub const __SIZEOF_PTHREAD_RWLOCKATTR_T: usize = 8;
pub const __SIZEOF_PTHREAD_BARRIERATTR_T: usize = 4;

pub const CPU_SETSIZE: ::c_int = 128;

pub const PTRACE_TRACEME: ::c_int = 0;
pub const PTRACE_PEEKTEXT: ::c_int = 1;
pub const PTRACE_PEEKDATA: ::c_int = 2;
pub const PTRACE_PEEKUSER: ::c_int = 3;
pub const PTRACE_POKETEXT: ::c_int = 4;
pub const PTRACE_POKEDATA: ::c_int = 5;
pub const PTRACE_POKEUSER: ::c_int = 6;
pub const PTRACE_CONT: ::c_int = 7;
pub const PTRACE_KILL: ::c_int = 8;
pub const PTRACE_SINGLESTEP: ::c_int = 9;
pub const PTRACE_GETREGS: ::c_int = 12;
pub const PTRACE_SETREGS: ::c_int = 13;
pub const PTRACE_GETFPREGS: ::c_int = 14;
pub const PTRACE_SETFPREGS: ::c_int = 15;
pub const PTRACE_ATTACH: ::c_int = 16;
pub const PTRACE_DETACH: ::c_int = 17;
pub const PTRACE_GETFPXREGS: ::c_int = 18;
pub const PTRACE_SETFPXREGS: ::c_int = 19;
pub const PTRACE_SYSCALL: ::c_int = 24;
pub const PTRACE_SETOPTIONS: ::c_int = 0x4200;
pub const PTRACE_GETEVENTMSG: ::c_int = 0x4201;
pub const PTRACE_GETSIGINFO: ::c_int = 0x4202;
pub const PTRACE_SETSIGINFO: ::c_int = 0x4203;
pub const PTRACE_GETREGSET: ::c_int = 0x4204;
pub const PTRACE_SETREGSET: ::c_int = 0x4205;
pub const PTRACE_SEIZE: ::c_int = 0x4206;
pub const PTRACE_INTERRUPT: ::c_int = 0x4207;
pub const PTRACE_LISTEN: ::c_int = 0x4208;
pub const PTRACE_PEEKSIGINFO: ::c_int = 0x4209;

pub const FAN_MARK_INODE: ::c_uint = 0x0000_0000;
pub const FAN_MARK_MOUNT: ::c_uint = 0x0000_0010;
// NOTE: FAN_MARK_FILESYSTEM requires Linux Kernel >= 4.20.0
pub const FAN_MARK_FILESYSTEM: ::c_uint = 0x0000_0100;

pub const AF_IB: ::c_int = 27;
pub const AF_MPLS: ::c_int = 28;
pub const AF_NFC: ::c_int = 39;
pub const AF_VSOCK: ::c_int = 40;
pub const AF_XDP: ::c_int = 44;
pub const PF_IB: ::c_int = AF_IB;
pub const PF_MPLS: ::c_int = AF_MPLS;
pub const PF_NFC: ::c_int = AF_NFC;
pub const PF_VSOCK: ::c_int = AF_VSOCK;
pub const PF_XDP: ::c_int = AF_XDP;

pub const EFD_NONBLOCK: ::c_int = ::O_NONBLOCK;

pub const SFD_NONBLOCK: ::c_int = ::O_NONBLOCK;

pub const PIDFD_NONBLOCK: ::c_uint = O_NONBLOCK as ::c_uint;

pub const TCSANOW: ::c_int = 0;
pub const TCSADRAIN: ::c_int = 1;
pub const TCSAFLUSH: ::c_int = 2;

pub const RTLD_GLOBAL: ::c_int = 0x100;
pub const RTLD_NOLOAD: ::c_int = 0x4;

pub const CLOCK_SGI_CYCLE: ::clockid_t = 10;

pub const B0: ::speed_t = 0o000000;
pub const B50: ::speed_t = 0o000001;
pub const B75: ::speed_t = 0o000002;
pub const B110: ::speed_t = 0o000003;
pub const B134: ::speed_t = 0o000004;
pub const B150: ::speed_t = 0o000005;
pub const B200: ::speed_t = 0o000006;
pub const B300: ::speed_t = 0o000007;
pub const B600: ::speed_t = 0o000010;
pub const B1200: ::speed_t = 0o000011;
pub const B1800: ::speed_t = 0o000012;
pub const B2400: ::speed_t = 0o000013;
pub const B4800: ::speed_t = 0o000014;
pub const B9600: ::speed_t = 0o000015;
pub const B19200: ::speed_t = 0o000016;
pub const B38400: ::speed_t = 0o000017;
pub const EXTA: ::speed_t = B19200;
pub const EXTB: ::speed_t = B38400;

pub const REG_OK: ::c_int = 0;

pub const PRIO_PROCESS: ::c_int = 0;
pub const PRIO_PGRP: ::c_int = 1;
pub const PRIO_USER: ::c_int = 2;

pub const ADJ_OFFSET: ::c_uint = 0x0001;
pub const ADJ_FREQUENCY: ::c_uint = 0x0002;
pub const ADJ_MAXERROR: ::c_uint = 0x0004;
pub const ADJ_ESTERROR: ::c_uint = 0x0008;
pub const ADJ_STATUS: ::c_uint = 0x0010;
pub const ADJ_TIMECONST: ::c_uint = 0x0020;
pub const ADJ_TAI: ::c_uint = 0x0080;
pub const ADJ_SETOFFSET: ::c_uint = 0x0100;
pub const ADJ_MICRO: ::c_uint = 0x1000;
pub const ADJ_NANO: ::c_uint = 0x2000;
pub const ADJ_TICK: ::c_uint = 0x4000;
pub const ADJ_OFFSET_SINGLESHOT: ::c_uint = 0x8001;
pub const ADJ_OFFSET_SS_READ: ::c_uint = 0xa001;
pub const MOD_OFFSET: ::c_uint = ADJ_OFFSET;
pub const MOD_FREQUENCY: ::c_uint = ADJ_FREQUENCY;
pub const MOD_MAXERROR: ::c_uint = ADJ_MAXERROR;
pub const MOD_ESTERROR: ::c_uint = ADJ_ESTERROR;
pub const MOD_STATUS: ::c_uint = ADJ_STATUS;
pub const MOD_TIMECONST: ::c_uint = ADJ_TIMECONST;
pub const MOD_CLKB: ::c_uint = ADJ_TICK;
pub const MOD_CLKA: ::c_uint = ADJ_OFFSET_SINGLESHOT;
pub const MOD_TAI: ::c_uint = ADJ_TAI;
pub const MOD_MICRO: ::c_uint = ADJ_MICRO;
pub const MOD_NANO: ::c_uint = ADJ_NANO;
pub const STA_PLL: ::c_int = 0x0001;
pub const STA_PPSFREQ: ::c_int = 0x0002;
pub const STA_PPSTIME: ::c_int = 0x0004;
pub const STA_FLL: ::c_int = 0x0008;
pub const STA_INS: ::c_int = 0x0010;
pub const STA_DEL: ::c_int = 0x0020;
pub const STA_UNSYNC: ::c_int = 0x0040;
pub const STA_FREQHOLD: ::c_int = 0x0080;
pub const STA_PPSSIGNAL: ::c_int = 0x0100;
pub const STA_PPSJITTER: ::c_int = 0x0200;
pub const STA_PPSWANDER: ::c_int = 0x0400;
pub const STA_PPSERROR: ::c_int = 0x0800;
pub const STA_CLOCKERR: ::c_int = 0x1000;
pub const STA_NANO: ::c_int = 0x2000;
pub const STA_MODE: ::c_int = 0x4000;
pub const STA_CLK: ::c_int = 0x8000;
pub const STA_RONLY: ::c_int = STA_PPSSIGNAL
    | STA_PPSJITTER
    | STA_PPSWANDER
    | STA_PPSERROR
    | STA_CLOCKERR
    | STA_NANO
    | STA_MODE
    | STA_CLK;

pub const TIME_OK: ::c_int = 0;
pub const TIME_INS: ::c_int = 1;
pub const TIME_DEL: ::c_int = 2;
pub const TIME_OOP: ::c_int = 3;
pub const TIME_WAIT: ::c_int = 4;
pub const TIME_ERROR: ::c_int = 5;
pub const TIME_BAD: ::c_int = TIME_ERROR;
pub const MAXTC: ::c_long = 6;

cfg_if! {
    if #[cfg(target_arch = "s390x")] {
        pub const POSIX_FADV_DONTNEED: ::c_int = 6;
        pub const POSIX_FADV_NOREUSE: ::c_int = 7;
    } else {
        pub const POSIX_FADV_DONTNEED: ::c_int = 4;
        pub const POSIX_FADV_NOREUSE: ::c_int = 5;
    }
}

extern "C" {
    pub fn sendmmsg(
        sockfd: ::c_int,
        msgvec: *mut ::mmsghdr,
        vlen: ::c_uint,
        flags: ::c_uint,
    ) -> ::c_int;
    pub fn recvmmsg(
        sockfd: ::c_int,
        msgvec: *mut ::mmsghdr,
        vlen: ::c_uint,
        flags: ::c_uint,
        timeout: *mut ::timespec,
    ) -> ::c_int;

    pub fn getrlimit(resource: ::c_int, rlim: *mut ::rlimit) -> ::c_int;
    pub fn setrlimit(resource: ::c_int, rlim: *const ::rlimit) -> ::c_int;
    pub fn prlimit(
        pid: ::pid_t,
        resource: ::c_int,
        new_limit: *const ::rlimit,
        old_limit: *mut ::rlimit,
    ) -> ::c_int;
    pub fn ioctl(fd: ::c_int, request: ::c_int, ...) -> ::c_int;
    pub fn gettimeofday(tp: *mut ::timeval, tz: *mut ::c_void) -> ::c_int;
    pub fn ptrace(request: ::c_int, ...) -> ::c_long;
    pub fn getpriority(which: ::c_int, who: ::id_t) -> ::c_int;
    pub fn setpriority(which: ::c_int, who: ::id_t, prio: ::c_int) -> ::c_int;
    // Musl targets need the `mask` argument of `fanotify_mark` be specified
    // `::c_ulonglong` instead of `u64` or there will be a type mismatch between
    // `long long unsigned int` and the expected `uint64_t`.
    pub fn fanotify_mark(
        fd: ::c_int,
        flags: ::c_uint,
        mask: ::c_ulonglong,
        dirfd: ::c_int,
        path: *const ::c_char,
    ) -> ::c_int;
    pub fn getauxval(type_: ::c_ulong) -> ::c_ulong;

    // Added in `musl` 1.1.20
    pub fn explicit_bzero(s: *mut ::c_void, len: ::size_t);
    // Added in `musl` 1.2.2
    pub fn reallocarray(ptr: *mut ::c_void, nmemb: ::size_t, size: ::size_t) -> *mut ::c_void;

    pub fn adjtimex(buf: *mut ::timex) -> ::c_int;
    pub fn clock_adjtime(clk_id: ::clockid_t, buf: *mut ::timex) -> ::c_int;

    pub fn ctermid(s: *mut ::c_char) -> *mut ::c_char;

    pub fn memfd_create(name: *const ::c_char, flags: ::c_uint) -> ::c_int;
    pub fn mlock2(addr: *const ::c_void, len: ::size_t, flags: ::c_uint) -> ::c_int;
    pub fn malloc_usable_size(ptr: *mut ::c_void) -> ::size_t;

    pub fn euidaccess(pathname: *const ::c_char, mode: ::c_int) -> ::c_int;
    pub fn eaccess(pathname: *const ::c_char, mode: ::c_int) -> ::c_int;

    pub fn asctime_r(tm: *const ::tm, buf: *mut ::c_char) -> *mut ::c_char;

    pub fn strftime(
        s: *mut ::c_char,
        max: ::size_t,
        format: *const ::c_char,
        tm: *const ::tm,
    ) -> ::size_t;
    pub fn strptime(s: *const ::c_char, format: *const ::c_char, tm: *mut ::tm) -> *mut ::c_char;

    pub fn dirname(path: *mut ::c_char) -> *mut ::c_char;
    pub fn basename(path: *mut ::c_char) -> *mut ::c_char;
}

// Alias <foo> to <foo>64 to mimic glibc's LFS64 support
mod lfs64;
pub use self::lfs64::*;

cfg_if! {
    if #[cfg(any(target_arch = "x86_64",
                 target_arch = "aarch64",
                 target_arch = "mips64",
                 target_arch = "powerpc64",
                 target_arch = "s390x",
                 target_arch = "riscv64"))] {
        mod b64;
        pub use self::b64::*;
    } else if #[cfg(any(target_arch = "x86",
                        target_arch = "mips",
                        target_arch = "powerpc",
                        target_arch = "hexagon",
                        target_arch = "riscv32",
                        target_arch = "arm"))] {
        mod b32;
        pub use self::b32::*;
    } else { }
}
