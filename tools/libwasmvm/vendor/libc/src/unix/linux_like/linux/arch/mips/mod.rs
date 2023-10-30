s! {
    pub struct termios2 {
        pub c_iflag: ::tcflag_t,
        pub c_oflag: ::tcflag_t,
        pub c_cflag: ::tcflag_t,
        pub c_lflag: ::tcflag_t,
        pub c_line: ::cc_t,
        pub c_cc: [::cc_t; 23],
        pub c_ispeed: ::speed_t,
        pub c_ospeed: ::speed_t,
    }
}

// arch/mips/include/uapi/asm/socket.h
pub const SOL_SOCKET: ::c_int = 0xffff;

// Defined in unix/linux_like/mod.rs
// pub const SO_DEBUG: ::c_int = 0x0001;
pub const SO_REUSEADDR: ::c_int = 0x0004;
pub const SO_KEEPALIVE: ::c_int = 0x0008;
pub const SO_DONTROUTE: ::c_int = 0x0010;
pub const SO_BROADCAST: ::c_int = 0x0020;
pub const SO_LINGER: ::c_int = 0x0080;
pub const SO_OOBINLINE: ::c_int = 0x0100;
pub const SO_REUSEPORT: ::c_int = 0x0200;
pub const SO_TYPE: ::c_int = 0x1008;
// pub const SO_STYLE: ::c_int = SO_TYPE;
pub const SO_ERROR: ::c_int = 0x1007;
pub const SO_SNDBUF: ::c_int = 0x1001;
pub const SO_RCVBUF: ::c_int = 0x1002;
pub const SO_SNDLOWAT: ::c_int = 0x1003;
pub const SO_RCVLOWAT: ::c_int = 0x1004;
// NOTE: These definitions are now being renamed with _OLD postfix,
// but CI haven't support them yet.
// Some related consts could be found in b32.rs and b64.rs
pub const SO_SNDTIMEO: ::c_int = 0x1005;
pub const SO_RCVTIMEO: ::c_int = 0x1006;
// pub const SO_SNDTIMEO_OLD: ::c_int = 0x1005;
// pub const SO_RCVTIMEO_OLD: ::c_int = 0x1006;
pub const SO_ACCEPTCONN: ::c_int = 0x1009;
pub const SO_PROTOCOL: ::c_int = 0x1028;
pub const SO_DOMAIN: ::c_int = 0x1029;

pub const SO_NO_CHECK: ::c_int = 11;
pub const SO_PRIORITY: ::c_int = 12;
pub const SO_BSDCOMPAT: ::c_int = 14;
pub const SO_PASSCRED: ::c_int = 17;
pub const SO_PEERCRED: ::c_int = 18;
pub const SO_SECURITY_AUTHENTICATION: ::c_int = 22;
pub const SO_SECURITY_ENCRYPTION_TRANSPORT: ::c_int = 23;
pub const SO_SECURITY_ENCRYPTION_NETWORK: ::c_int = 24;
pub const SO_BINDTODEVICE: ::c_int = 25;
pub const SO_ATTACH_FILTER: ::c_int = 26;
pub const SO_DETACH_FILTER: ::c_int = 27;
pub const SO_GET_FILTER: ::c_int = SO_ATTACH_FILTER;
pub const SO_PEERNAME: ::c_int = 28;
pub const SO_PEERSEC: ::c_int = 30;
pub const SO_SNDBUFFORCE: ::c_int = 31;
pub const SO_RCVBUFFORCE: ::c_int = 33;
pub const SO_PASSSEC: ::c_int = 34;
pub const SO_MARK: ::c_int = 36;
pub const SO_RXQ_OVFL: ::c_int = 40;
pub const SO_WIFI_STATUS: ::c_int = 41;
pub const SCM_WIFI_STATUS: ::c_int = SO_WIFI_STATUS;
pub const SO_PEEK_OFF: ::c_int = 42;
pub const SO_NOFCS: ::c_int = 43;
pub const SO_LOCK_FILTER: ::c_int = 44;
pub const SO_SELECT_ERR_QUEUE: ::c_int = 45;
pub const SO_BUSY_POLL: ::c_int = 46;
pub const SO_MAX_PACING_RATE: ::c_int = 47;
pub const SO_BPF_EXTENSIONS: ::c_int = 48;
pub const SO_INCOMING_CPU: ::c_int = 49;
pub const SO_ATTACH_BPF: ::c_int = 50;
pub const SO_DETACH_BPF: ::c_int = SO_DETACH_FILTER;
pub const SO_ATTACH_REUSEPORT_CBPF: ::c_int = 51;
pub const SO_ATTACH_REUSEPORT_EBPF: ::c_int = 52;
pub const SO_CNX_ADVICE: ::c_int = 53;
pub const SCM_TIMESTAMPING_OPT_STATS: ::c_int = 54;
pub const SO_MEMINFO: ::c_int = 55;
pub const SO_INCOMING_NAPI_ID: ::c_int = 56;
pub const SO_COOKIE: ::c_int = 57;
pub const SCM_TIMESTAMPING_PKTINFO: ::c_int = 58;
pub const SO_PEERGROUPS: ::c_int = 59;
pub const SO_ZEROCOPY: ::c_int = 60;
pub const SO_TXTIME: ::c_int = 61;
pub const SCM_TXTIME: ::c_int = SO_TXTIME;
pub const SO_BINDTOIFINDEX: ::c_int = 62;
// NOTE: These definitions are now being renamed with _OLD postfix,
// but CI haven't support them yet.
// Some related consts could be found in b32.rs and b64.rs
pub const SO_TIMESTAMP: ::c_int = 29;
pub const SO_TIMESTAMPNS: ::c_int = 35;
pub const SO_TIMESTAMPING: ::c_int = 37;
// pub const SO_TIMESTAMP_OLD: ::c_int = 29;
// pub const SO_TIMESTAMPNS_OLD: ::c_int = 35;
// pub const SO_TIMESTAMPING_OLD: ::c_int = 37;
// pub const SO_TIMESTAMP_NEW: ::c_int = 63;
// pub const SO_TIMESTAMPNS_NEW: ::c_int = 64;
// pub const SO_TIMESTAMPING_NEW: ::c_int = 65;
// pub const SO_RCVTIMEO_NEW: ::c_int = 66;
// pub const SO_SNDTIMEO_NEW: ::c_int = 67;
// pub const SO_DETACH_REUSEPORT_BPF: ::c_int = 68;
// pub const SO_PREFER_BUSY_POLL: ::c_int = 69;
// pub const SO_BUSY_POLL_BUDGET: ::c_int = 70;

pub const FICLONE: ::c_ulong = 0x80049409;
pub const FICLONERANGE: ::c_ulong = 0x8020940D;

// Defined in unix/linux_like/mod.rs
// pub const SCM_TIMESTAMP: ::c_int = SO_TIMESTAMP;
pub const SCM_TIMESTAMPNS: ::c_int = SO_TIMESTAMPNS;
pub const SCM_TIMESTAMPING: ::c_int = SO_TIMESTAMPING;

// Ioctl Constants

pub const TCGETS: ::Ioctl = 0x540d;
pub const TCSETS: ::Ioctl = 0x540e;
pub const TCSETSW: ::Ioctl = 0x540f;
pub const TCSETSF: ::Ioctl = 0x5410;
pub const TCGETA: ::Ioctl = 0x5401;
pub const TCSETA: ::Ioctl = 0x5402;
pub const TCSETAW: ::Ioctl = 0x5403;
pub const TCSETAF: ::Ioctl = 0x5404;
pub const TCSBRK: ::Ioctl = 0x5405;
pub const TCXONC: ::Ioctl = 0x5406;
pub const TCFLSH: ::Ioctl = 0x5407;
pub const TIOCEXCL: ::Ioctl = 0x740d;
pub const TIOCNXCL: ::Ioctl = 0x740e;
pub const TIOCSCTTY: ::Ioctl = 0x5480;
pub const TIOCGPGRP: ::Ioctl = 0x40047477;
pub const TIOCSPGRP: ::Ioctl = 0x80047476;
pub const TIOCOUTQ: ::Ioctl = 0x7472;
pub const TIOCSTI: ::Ioctl = 0x5472;
pub const TIOCGWINSZ: ::Ioctl = 0x40087468;
pub const TIOCSWINSZ: ::Ioctl = 0x80087467;
pub const TIOCMGET: ::Ioctl = 0x741d;
pub const TIOCMBIS: ::Ioctl = 0x741b;
pub const TIOCMBIC: ::Ioctl = 0x741c;
pub const TIOCMSET: ::Ioctl = 0x741a;
pub const TIOCGSOFTCAR: ::Ioctl = 0x5481;
pub const TIOCSSOFTCAR: ::Ioctl = 0x5482;
pub const FIONREAD: ::Ioctl = 0x467f;
pub const TIOCINQ: ::Ioctl = FIONREAD;
pub const TIOCLINUX: ::Ioctl = 0x5483;
pub const TIOCCONS: ::Ioctl = 0x80047478;
pub const TIOCGSERIAL: ::Ioctl = 0x5484;
pub const TIOCSSERIAL: ::Ioctl = 0x5485;
pub const TIOCPKT: ::Ioctl = 0x5470;
pub const FIONBIO: ::Ioctl = 0x667e;
pub const TIOCNOTTY: ::Ioctl = 0x5471;
pub const TIOCSETD: ::Ioctl = 0x7401;
pub const TIOCGETD: ::Ioctl = 0x7400;
pub const TCSBRKP: ::Ioctl = 0x5486;
pub const TIOCSBRK: ::Ioctl = 0x5427;
pub const TIOCCBRK: ::Ioctl = 0x5428;
pub const TIOCGSID: ::Ioctl = 0x7416;
pub const TCGETS2: ::Ioctl = 0x4030542a;
pub const TCSETS2: ::Ioctl = 0x8030542b;
pub const TCSETSW2: ::Ioctl = 0x8030542c;
pub const TCSETSF2: ::Ioctl = 0x8030542d;
pub const TIOCGPTN: ::Ioctl = 0x40045430;
pub const TIOCSPTLCK: ::Ioctl = 0x80045431;
pub const TIOCGDEV: ::Ioctl = 0x40045432;
pub const TIOCSIG: ::Ioctl = 0x80045436;
pub const TIOCVHANGUP: ::Ioctl = 0x5437;
pub const TIOCGPKT: ::Ioctl = 0x40045438;
pub const TIOCGPTLCK: ::Ioctl = 0x40045439;
pub const TIOCGEXCL: ::Ioctl = 0x40045440;
pub const TIOCGPTPEER: ::Ioctl = 0x20005441;
//pub const TIOCGISO7816: ::Ioctl = 0x40285442;
//pub const TIOCSISO7816: ::Ioctl = 0xc0285443;
pub const FIONCLEX: ::Ioctl = 0x6602;
pub const FIOCLEX: ::Ioctl = 0x6601;
pub const FIOASYNC: ::Ioctl = 0x667d;
pub const TIOCSERCONFIG: ::Ioctl = 0x5488;
pub const TIOCSERGWILD: ::Ioctl = 0x5489;
pub const TIOCSERSWILD: ::Ioctl = 0x548a;
pub const TIOCGLCKTRMIOS: ::Ioctl = 0x548b;
pub const TIOCSLCKTRMIOS: ::Ioctl = 0x548c;
pub const TIOCSERGSTRUCT: ::Ioctl = 0x548d;
pub const TIOCSERGETLSR: ::Ioctl = 0x548e;
pub const TIOCSERGETMULTI: ::Ioctl = 0x548f;
pub const TIOCSERSETMULTI: ::Ioctl = 0x5490;
pub const TIOCMIWAIT: ::Ioctl = 0x5491;
pub const TIOCGICOUNT: ::Ioctl = 0x5492;
pub const FIOQSIZE: ::Ioctl = 0x667f;
pub const TIOCSLTC: ::Ioctl = 0x7475;
pub const TIOCGETP: ::Ioctl = 0x7408;
pub const TIOCSETP: ::Ioctl = 0x7409;
pub const TIOCSETN: ::Ioctl = 0x740a;
pub const BLKIOMIN: ::Ioctl = 0x20001278;
pub const BLKIOOPT: ::Ioctl = 0x20001279;
pub const BLKSSZGET: ::Ioctl = 0x20001268;
pub const BLKPBSZGET: ::Ioctl = 0x2000127B;

cfg_if! {
    if #[cfg(target_env = "musl")] {
        pub const TIOCGRS485: ::Ioctl = 0x4020542e;
        pub const TIOCSRS485: ::Ioctl = 0xc020542f;
    }
}

pub const TIOCM_LE: ::c_int = 0x001;
pub const TIOCM_DTR: ::c_int = 0x002;
pub const TIOCM_RTS: ::c_int = 0x004;
pub const TIOCM_ST: ::c_int = 0x010;
pub const TIOCM_SR: ::c_int = 0x020;
pub const TIOCM_CTS: ::c_int = 0x040;
pub const TIOCM_CAR: ::c_int = 0x100;
pub const TIOCM_CD: ::c_int = TIOCM_CAR;
pub const TIOCM_RNG: ::c_int = 0x200;
pub const TIOCM_RI: ::c_int = TIOCM_RNG;
pub const TIOCM_DSR: ::c_int = 0x400;

pub const BOTHER: ::speed_t = 0o010000;
pub const IBSHIFT: ::tcflag_t = 16;

// RLIMIT Constants

cfg_if! {
    if #[cfg(any(target_env = "gnu",
                 target_env = "uclibc"))] {

        pub const RLIMIT_CPU: ::__rlimit_resource_t = 0;
        pub const RLIMIT_FSIZE: ::__rlimit_resource_t = 1;
        pub const RLIMIT_DATA: ::__rlimit_resource_t = 2;
        pub const RLIMIT_STACK: ::__rlimit_resource_t = 3;
        pub const RLIMIT_CORE: ::__rlimit_resource_t = 4;
        pub const RLIMIT_NOFILE: ::__rlimit_resource_t = 5;
        pub const RLIMIT_AS: ::__rlimit_resource_t = 6;
        pub const RLIMIT_RSS: ::__rlimit_resource_t = 7;
        pub const RLIMIT_NPROC: ::__rlimit_resource_t = 8;
        pub const RLIMIT_MEMLOCK: ::__rlimit_resource_t = 9;
        pub const RLIMIT_LOCKS: ::__rlimit_resource_t = 10;
        pub const RLIMIT_SIGPENDING: ::__rlimit_resource_t = 11;
        pub const RLIMIT_MSGQUEUE: ::__rlimit_resource_t = 12;
        pub const RLIMIT_NICE: ::__rlimit_resource_t = 13;
        pub const RLIMIT_RTPRIO: ::__rlimit_resource_t = 14;
        pub const RLIMIT_RTTIME: ::__rlimit_resource_t = 15;
        pub const RLIMIT_NLIMITS: ::__rlimit_resource_t = RLIM_NLIMITS;

    } else if #[cfg(target_env = "musl")] {

        pub const RLIMIT_CPU: ::c_int = 0;
        pub const RLIMIT_FSIZE: ::c_int = 1;
        pub const RLIMIT_DATA: ::c_int = 2;
        pub const RLIMIT_STACK: ::c_int = 3;
        pub const RLIMIT_CORE: ::c_int = 4;
        pub const RLIMIT_NOFILE: ::c_int = 5;
        pub const RLIMIT_AS: ::c_int = 6;
        pub const RLIMIT_RSS: ::c_int = 7;
        pub const RLIMIT_NPROC: ::c_int = 8;
        pub const RLIMIT_MEMLOCK: ::c_int = 9;
        pub const RLIMIT_LOCKS: ::c_int = 10;
        pub const RLIMIT_SIGPENDING: ::c_int = 11;
        pub const RLIMIT_MSGQUEUE: ::c_int = 12;
        pub const RLIMIT_NICE: ::c_int = 13;
        pub const RLIMIT_RTPRIO: ::c_int = 14;
        pub const RLIMIT_RTTIME: ::c_int = 15;
        pub const RLIM_NLIMITS: ::c_int = 15;
        pub const RLIMIT_NLIMITS: ::c_int = RLIM_NLIMITS;
        pub const RLIM_INFINITY: ::rlim_t = !0;
    }
}

cfg_if! {
    if #[cfg(target_env = "gnu")] {
        pub const RLIM_NLIMITS: ::__rlimit_resource_t = 16;
    } else if #[cfg(target_env = "uclibc")] {
        pub const RLIM_NLIMITS: ::__rlimit_resource_t = 15;
    }
}

cfg_if! {
    if #[cfg(target_arch = "mips64",
         any(target_env = "gnu",
             target_env = "uclibc"))] {
        pub const RLIM_INFINITY: ::rlim_t = !0;
    }
}

cfg_if! {
    if #[cfg(target_arch = "mips",
         any(target_env = "gnu",
             target_env = "uclibc"))] {
        pub const RLIM_INFINITY: ::rlim_t = 0x7fffffff;
    }
}
