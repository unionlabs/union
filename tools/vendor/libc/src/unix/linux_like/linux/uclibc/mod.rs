pub type shmatt_t = ::c_ulong;
pub type msgqnum_t = ::c_ulong;
pub type msglen_t = ::c_ulong;
pub type regoff_t = ::c_int;
pub type rlim_t = ::c_ulong;
pub type __rlimit_resource_t = ::c_ulong;
pub type __priority_which_t = ::c_uint;

cfg_if! {
    if #[cfg(doc)] {
        // Used in `linux::arch` to define ioctl constants.
        pub(crate) type Ioctl = ::c_ulong;
    } else {
        #[doc(hidden)]
        pub type Ioctl = ::c_ulong;
    }
}

s! {
    pub struct statvfs {  // Different than GNU!
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

    pub struct regex_t {
        __buffer: *mut ::c_void,
        __allocated: ::size_t,
        __used: ::size_t,
        __syntax: ::c_ulong,
        __fastmap: *mut ::c_char,
        __translate: *mut ::c_char,
        __re_nsub: ::size_t,
        __bitfield: u8,
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
        pub rt_pad4: ::c_short,
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

    pub struct ptrace_peeksiginfo_args {
        pub off: ::__u64,
        pub flags: ::__u32,
        pub nr: ::__s32,
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

pub const MCL_CURRENT: ::c_int = 0x0001;
pub const MCL_FUTURE: ::c_int = 0x0002;

pub const SIGEV_THREAD_ID: ::c_int = 4;

pub const AF_VSOCK: ::c_int = 40;

// Most `*_SUPER_MAGIC` constants are defined at the `linux_like` level; the
// following are only available on newer Linux versions than the versions
// currently used in CI in some configurations, so we define them here.
pub const BINDERFS_SUPER_MAGIC: ::c_long = 0x6c6f6f70;
pub const XFS_SUPER_MAGIC: ::c_long = 0x58465342;

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

pub const POSIX_FADV_DONTNEED: ::c_int = 4;
pub const POSIX_FADV_NOREUSE: ::c_int = 5;

// These are different than GNU!
pub const LC_CTYPE: ::c_int = 0;
pub const LC_NUMERIC: ::c_int = 1;
pub const LC_TIME: ::c_int = 3;
pub const LC_COLLATE: ::c_int = 4;
pub const LC_MONETARY: ::c_int = 2;
pub const LC_MESSAGES: ::c_int = 5;
pub const LC_ALL: ::c_int = 6;
// end different section

// MS_ flags for mount(2)
pub const MS_RMT_MASK: ::c_ulong = ::MS_RDONLY | ::MS_SYNCHRONOUS | ::MS_MANDLOCK | ::MS_I_VERSION;

pub const ENOTSUP: ::c_int = EOPNOTSUPP;

pub const IPV6_JOIN_GROUP: ::c_int = 20;
pub const IPV6_LEAVE_GROUP: ::c_int = 21;

// These are different from GNU
pub const ABDAY_1: ::nl_item = 0x300;
pub const ABDAY_2: ::nl_item = 0x301;
pub const ABDAY_3: ::nl_item = 0x302;
pub const ABDAY_4: ::nl_item = 0x303;
pub const ABDAY_5: ::nl_item = 0x304;
pub const ABDAY_6: ::nl_item = 0x305;
pub const ABDAY_7: ::nl_item = 0x306;
pub const DAY_1: ::nl_item = 0x307;
pub const DAY_2: ::nl_item = 0x308;
pub const DAY_3: ::nl_item = 0x309;
pub const DAY_4: ::nl_item = 0x30A;
pub const DAY_5: ::nl_item = 0x30B;
pub const DAY_6: ::nl_item = 0x30C;
pub const DAY_7: ::nl_item = 0x30D;
pub const ABMON_1: ::nl_item = 0x30E;
pub const ABMON_2: ::nl_item = 0x30F;
pub const ABMON_3: ::nl_item = 0x310;
pub const ABMON_4: ::nl_item = 0x311;
pub const ABMON_5: ::nl_item = 0x312;
pub const ABMON_6: ::nl_item = 0x313;
pub const ABMON_7: ::nl_item = 0x314;
pub const ABMON_8: ::nl_item = 0x315;
pub const ABMON_9: ::nl_item = 0x316;
pub const ABMON_10: ::nl_item = 0x317;
pub const ABMON_11: ::nl_item = 0x318;
pub const ABMON_12: ::nl_item = 0x319;
pub const MON_1: ::nl_item = 0x31A;
pub const MON_2: ::nl_item = 0x31B;
pub const MON_3: ::nl_item = 0x31C;
pub const MON_4: ::nl_item = 0x31D;
pub const MON_5: ::nl_item = 0x31E;
pub const MON_6: ::nl_item = 0x31F;
pub const MON_7: ::nl_item = 0x320;
pub const MON_8: ::nl_item = 0x321;
pub const MON_9: ::nl_item = 0x322;
pub const MON_10: ::nl_item = 0x323;
pub const MON_11: ::nl_item = 0x324;
pub const MON_12: ::nl_item = 0x325;
pub const AM_STR: ::nl_item = 0x326;
pub const PM_STR: ::nl_item = 0x327;
pub const D_T_FMT: ::nl_item = 0x328;
pub const D_FMT: ::nl_item = 0x329;
pub const T_FMT: ::nl_item = 0x32A;
pub const T_FMT_AMPM: ::nl_item = 0x32B;
pub const ERA: ::nl_item = 0x32C;
pub const ERA_D_FMT: ::nl_item = 0x32E;
pub const ALT_DIGITS: ::nl_item = 0x32F;
pub const ERA_D_T_FMT: ::nl_item = 0x330;
pub const ERA_T_FMT: ::nl_item = 0x331;
pub const CODESET: ::nl_item = 10;
pub const CRNCYSTR: ::nl_item = 0x215;
pub const RADIXCHAR: ::nl_item = 0x100;
pub const THOUSEP: ::nl_item = 0x101;
pub const NOEXPR: ::nl_item = 0x501;
pub const YESSTR: ::nl_item = 0x502;
pub const NOSTR: ::nl_item = 0x503;

// Different than Gnu.
pub const FILENAME_MAX: ::c_uint = 4095;

pub const PRIO_PROCESS: ::c_int = 0;
pub const PRIO_PGRP: ::c_int = 1;
pub const PRIO_USER: ::c_int = 2;

pub const SOMAXCONN: ::c_int = 128;

pub const ST_RELATIME: ::c_ulong = 4096;

pub const AF_NFC: ::c_int = PF_NFC;
pub const BUFSIZ: ::c_int = 4096;
pub const EDEADLOCK: ::c_int = EDEADLK;
pub const EXTA: ::c_uint = B19200;
pub const EXTB: ::c_uint = B38400;
pub const EXTPROC: ::tcflag_t = 0200000;
pub const FAN_MARK_FILESYSTEM: ::c_int = 0x00000100;
pub const FAN_MARK_INODE: ::c_int = 0x00000000;
pub const FAN_MARK_MOUNT: ::c_int = 0x10;
pub const FOPEN_MAX: ::c_int = 16;
pub const F_GETOWN: ::c_int = 9;
pub const F_OFD_GETLK: ::c_int = 36;
pub const F_OFD_SETLK: ::c_int = 37;
pub const F_OFD_SETLKW: ::c_int = 38;
pub const F_RDLCK: ::c_int = 0;
pub const F_SETOWN: ::c_int = 8;
pub const F_UNLCK: ::c_int = 2;
pub const F_WRLCK: ::c_int = 1;
pub const IPV6_MULTICAST_ALL: ::c_int = 29;
pub const IPV6_ROUTER_ALERT_ISOLATE: ::c_int = 30;
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
pub const MINSIGSTKSZ: ::c_int = 2048;
pub const MSG_COPY: ::c_int = 040000;
pub const NI_MAXHOST: ::socklen_t = 1025;
pub const O_TMPFILE: ::c_int = 0o20000000 | O_DIRECTORY;
pub const PACKET_MR_UNICAST: ::c_int = 3;
pub const PF_NFC: ::c_int = 39;
pub const PF_VSOCK: ::c_int = 40;
pub const POSIX_MADV_DONTNEED: ::c_int = 4;
pub const PTRACE_EVENT_STOP: ::c_int = 128;
pub const PTRACE_PEEKSIGINFO: ::c_int = 0x4209;
pub const RTLD_NOLOAD: ::c_int = 0x00004;
pub const RUSAGE_THREAD: ::c_int = 1;
pub const SHM_EXEC: ::c_int = 0100000;
pub const SIGPOLL: ::c_int = SIGIO;
pub const SOCK_DCCP: ::c_int = 6;
pub const SOCK_PACKET: ::c_int = 10;
pub const TCP_COOKIE_TRANSACTIONS: ::c_int = 15;
pub const UDP_GRO: ::c_int = 104;
pub const UDP_SEGMENT: ::c_int = 103;
pub const YESEXPR: ::c_int = ((5) << 8) | (0);

extern "C" {
    pub fn ioctl(fd: ::c_int, request: ::c_ulong, ...) -> ::c_int;
    pub fn gettimeofday(tp: *mut ::timeval, tz: *mut ::timezone) -> ::c_int;

    pub fn pthread_rwlockattr_getkind_np(
        attr: *const ::pthread_rwlockattr_t,
        val: *mut ::c_int,
    ) -> ::c_int;
    pub fn pthread_rwlockattr_setkind_np(
        attr: *mut ::pthread_rwlockattr_t,
        val: ::c_int,
    ) -> ::c_int;

    pub fn ptrace(request: ::c_uint, ...) -> ::c_long;

    pub fn sendmmsg(
        sockfd: ::c_int,
        msgvec: *mut ::mmsghdr,
        vlen: ::c_uint,
        flags: ::c_int,
    ) -> ::c_int;
    pub fn recvmmsg(
        sockfd: ::c_int,
        msgvec: *mut ::mmsghdr,
        vlen: ::c_uint,
        flags: ::c_int,
        timeout: *mut ::timespec,
    ) -> ::c_int;

    pub fn openpty(
        amaster: *mut ::c_int,
        aslave: *mut ::c_int,
        name: *mut ::c_char,
        termp: *mut termios,
        winp: *mut ::winsize,
    ) -> ::c_int;
    pub fn forkpty(
        amaster: *mut ::c_int,
        name: *mut ::c_char,
        termp: *mut termios,
        winp: *mut ::winsize,
    ) -> ::pid_t;

    pub fn getnameinfo(
        sa: *const ::sockaddr,
        salen: ::socklen_t,
        host: *mut ::c_char,
        hostlen: ::socklen_t,
        serv: *mut ::c_char,
        sevlen: ::socklen_t,
        flags: ::c_int,
    ) -> ::c_int;

    pub fn pwritev(
        fd: ::c_int,
        iov: *const ::iovec,
        iovcnt: ::c_int,
        offset: ::off64_t,
    ) -> ::ssize_t;
    pub fn preadv(
        fd: ::c_int,
        iov: *const ::iovec,
        iovcnt: ::c_int,
        offset: ::off64_t,
    ) -> ::ssize_t;

    pub fn sethostid(hostid: ::c_long) -> ::c_int;
    pub fn fanotify_mark(
        fd: ::c_int,
        flags: ::c_uint,
        mask: u64,
        dirfd: ::c_int,
        path: *const ::c_char,
    ) -> ::c_int;
    pub fn getrlimit64(resource: ::__rlimit_resource_t, rlim: *mut ::rlimit64) -> ::c_int;
    pub fn setrlimit64(resource: ::__rlimit_resource_t, rlim: *const ::rlimit64) -> ::c_int;
    pub fn getrlimit(resource: ::__rlimit_resource_t, rlim: *mut ::rlimit) -> ::c_int;
    pub fn setrlimit(resource: ::__rlimit_resource_t, rlim: *const ::rlimit) -> ::c_int;
    pub fn getpriority(which: ::__priority_which_t, who: ::id_t) -> ::c_int;
    pub fn setpriority(which: ::__priority_which_t, who: ::id_t, prio: ::c_int) -> ::c_int;
}

cfg_if! {
    if #[cfg(any(target_arch = "mips", target_arch = "mips64"))] {
        mod mips;
        pub use self::mips::*;
    } else if #[cfg(target_arch = "x86_64")] {
        mod x86_64;
        pub use self::x86_64::*;
    } else if #[cfg(target_arch = "arm")] {
        mod arm;
        pub use self::arm::*;
    } else {
        pub use unsupported_target;
    }
}
