//! Adapt the Linux API to resemble a POSIX-style libc API.
//!
//! The linux_raw backend doesn't use actual libc; this just defines certain
//! types that are convenient to have defined.

#![allow(unused_imports)]
#![allow(non_camel_case_types)]

pub type size_t = usize;
pub(crate) use linux_raw_sys::ctypes::*;
pub(crate) use linux_raw_sys::errno::EINVAL;
// Import the kernel's `uid_t` and `gid_t` if they're 32-bit.
#[cfg(not(any(target_arch = "arm", target_arch = "sparc", target_arch = "x86")))]
pub(crate) use linux_raw_sys::general::{__kernel_gid_t as gid_t, __kernel_uid_t as uid_t};
pub(crate) use linux_raw_sys::general::{
    __kernel_pid_t as pid_t, __kernel_time64_t as time_t, __kernel_timespec as timespec, iovec,
    O_CLOEXEC, O_NOCTTY, O_NONBLOCK, O_RDWR,
};

#[cfg(feature = "event")]
#[cfg(test)]
pub(crate) use linux_raw_sys::general::epoll_event;

#[cfg(any(
    feature = "fs",
    all(
        not(feature = "use-libc-auxv"),
        not(target_vendor = "mustang"),
        any(
            feature = "param",
            feature = "runtime",
            feature = "time",
            target_arch = "x86",
        )
    )
))]
pub(crate) use linux_raw_sys::general::{
    AT_FDCWD, NFS_SUPER_MAGIC, O_LARGEFILE, PROC_SUPER_MAGIC, UTIME_NOW, UTIME_OMIT, XATTR_CREATE,
    XATTR_REPLACE,
};

#[cfg(feature = "io_uring")]
pub(crate) use linux_raw_sys::{general::open_how, io_uring::*};

#[cfg(feature = "net")]
pub(crate) use linux_raw_sys::{
    cmsg_macros::*,
    general::{O_CLOEXEC as SOCK_CLOEXEC, O_NONBLOCK as SOCK_NONBLOCK},
    if_ether::*,
    net::{
        AF_DECnet, __kernel_sa_family_t as sa_family_t,
        __kernel_sockaddr_storage as sockaddr_storage, cmsghdr, in6_addr, in_addr, ip_mreq,
        ipv6_mreq, linger, msghdr, sockaddr, sockaddr_in, sockaddr_in6, sockaddr_un, socklen_t,
        AF_APPLETALK, AF_ASH, AF_ATMPVC, AF_ATMSVC, AF_AX25, AF_BLUETOOTH, AF_BRIDGE, AF_CAN,
        AF_ECONET, AF_IEEE802154, AF_INET, AF_INET6, AF_IPX, AF_IRDA, AF_ISDN, AF_IUCV, AF_KEY,
        AF_LLC, AF_NETBEUI, AF_NETLINK, AF_NETROM, AF_PACKET, AF_PHONET, AF_PPPOX, AF_RDS, AF_ROSE,
        AF_RXRPC, AF_SECURITY, AF_SNA, AF_TIPC, AF_UNIX, AF_UNSPEC, AF_WANPIPE, AF_X25, IPPROTO_AH,
        IPPROTO_BEETPH, IPPROTO_COMP, IPPROTO_DCCP, IPPROTO_EGP, IPPROTO_ENCAP, IPPROTO_ESP,
        IPPROTO_ETHERNET, IPPROTO_FRAGMENT, IPPROTO_GRE, IPPROTO_ICMP, IPPROTO_ICMPV6, IPPROTO_IDP,
        IPPROTO_IGMP, IPPROTO_IP, IPPROTO_IPIP, IPPROTO_IPV6, IPPROTO_MH, IPPROTO_MPLS,
        IPPROTO_MPTCP, IPPROTO_MTP, IPPROTO_PIM, IPPROTO_PUP, IPPROTO_RAW, IPPROTO_ROUTING,
        IPPROTO_RSVP, IPPROTO_SCTP, IPPROTO_TCP, IPPROTO_TP, IPPROTO_UDP, IPPROTO_UDPLITE,
        IPV6_ADD_MEMBERSHIP, IPV6_DROP_MEMBERSHIP, IPV6_MULTICAST_HOPS, IPV6_MULTICAST_LOOP,
        IPV6_UNICAST_HOPS, IPV6_V6ONLY, IP_ADD_MEMBERSHIP, IP_DROP_MEMBERSHIP, IP_MULTICAST_LOOP,
        IP_MULTICAST_TTL, IP_TTL, MSG_CMSG_CLOEXEC, MSG_CONFIRM, MSG_DONTROUTE, MSG_DONTWAIT,
        MSG_EOR, MSG_ERRQUEUE, MSG_MORE, MSG_NOSIGNAL, MSG_OOB, MSG_PEEK, MSG_TRUNC, MSG_WAITALL,
        SCM_CREDENTIALS, SCM_RIGHTS, SHUT_RD, SHUT_RDWR, SHUT_WR, SOCK_DGRAM, SOCK_RAW, SOCK_RDM,
        SOCK_SEQPACKET, SOCK_STREAM, SOL_SOCKET, SO_BROADCAST, SO_ERROR, SO_KEEPALIVE, SO_LINGER,
        SO_PASSCRED, SO_RCVBUF, SO_RCVTIMEO_NEW, SO_RCVTIMEO_NEW as SO_RCVTIMEO, SO_RCVTIMEO_OLD,
        SO_REUSEADDR, SO_SNDBUF, SO_SNDTIMEO_NEW, SO_SNDTIMEO_NEW as SO_SNDTIMEO, SO_SNDTIMEO_OLD,
        SO_TYPE, TCP_NODELAY,
    },
    netlink::*,
};

#[cfg(any(feature = "process", feature = "runtime"))]
pub(crate) use linux_raw_sys::general::siginfo_t;

#[cfg(feature = "process")]
pub(crate) use linux_raw_sys::general::{
    CLD_CONTINUED, CLD_DUMPED, CLD_EXITED, CLD_KILLED, CLD_STOPPED, CLD_TRAPPED,
    O_NONBLOCK as PIDFD_NONBLOCK, P_ALL, P_PID, P_PIDFD,
};

#[cfg(feature = "termios")]
pub(crate) use linux_raw_sys::{
    general::{
        cc_t, speed_t, tcflag_t, termios, termios2, winsize, B0, B1000000, B110, B115200, B1152000,
        B1200, B134, B150, B1500000, B1800, B19200, B200, B2000000, B230400, B2400, B2500000, B300,
        B3000000, B3500000, B38400, B4000000, B460800, B4800, B50, B500000, B57600, B576000, B600,
        B75, B921600, B9600, BOTHER, BRKINT, BS0, BS1, BSDLY, CBAUD, CBAUDEX, CIBAUD, CLOCAL,
        CMSPAR, CR0, CR1, CR2, CR3, CRDLY, CREAD, CRTSCTS, CS5, CS6, CS7, CS8, CSIZE, CSTOPB, ECHO,
        ECHOCTL, ECHOE, ECHOK, ECHOKE, ECHONL, ECHOPRT, EXTA, EXTB, EXTPROC, FF0, FF1, FFDLY,
        FLUSHO, HUPCL, IBSHIFT, ICANON, ICRNL, IEXTEN, IGNBRK, IGNCR, IGNPAR, IMAXBEL, INLCR,
        INPCK, ISIG, ISTRIP, IUCLC, IUTF8, IXANY, IXOFF, IXON, NCCS, NL0, NL1, NLDLY, NOFLSH,
        OCRNL, OFDEL, OFILL, OLCUC, ONLCR, ONLRET, ONOCR, OPOST, PARENB, PARMRK, PARODD, PENDIN,
        TAB0, TAB1, TAB2, TAB3, TABDLY, TCIFLUSH, TCIOFF, TCIOFLUSH, TCION, TCOFLUSH, TCOOFF,
        TCOON, TCSADRAIN, TCSAFLUSH, TCSANOW, TOSTOP, VDISCARD, VEOF, VEOL, VEOL2, VERASE, VINTR,
        VKILL, VLNEXT, VMIN, VQUIT, VREPRINT, VSTART, VSTOP, VSUSP, VSWTC, VT0, VT1, VTDLY, VTIME,
        VWERASE, XCASE, XTABS,
    },
    ioctl::{TCGETS2, TCSETS2, TCSETSF2, TCSETSW2},
};

// On MIPS, `TCSANOW` et al have `TCSETS` added to them, so we need it to
// subtract it out.
#[cfg(all(feature = "termios", any(target_arch = "mips", target_arch = "mips64")))]
pub(crate) use linux_raw_sys::ioctl::TCSETS;

// Define our own `uid_t` and `gid_t` if the kernel's versions are not 32-bit.
#[cfg(any(target_arch = "arm", target_arch = "sparc", target_arch = "x86"))]
pub(crate) type uid_t = u32;
#[cfg(any(target_arch = "arm", target_arch = "sparc", target_arch = "x86"))]
pub(crate) type gid_t = u32;

// Bindgen infers `u32` for many of these macro types which meant to be
// used with `c_int` in the C APIs, so cast them to `c_int`.

// Convert the signal constants from `u32` to `c_int`.
pub(crate) const SIGHUP: c_int = linux_raw_sys::general::SIGHUP as _;
pub(crate) const SIGINT: c_int = linux_raw_sys::general::SIGINT as _;
pub(crate) const SIGQUIT: c_int = linux_raw_sys::general::SIGQUIT as _;
pub(crate) const SIGILL: c_int = linux_raw_sys::general::SIGILL as _;
pub(crate) const SIGTRAP: c_int = linux_raw_sys::general::SIGTRAP as _;
pub(crate) const SIGABRT: c_int = linux_raw_sys::general::SIGABRT as _;
pub(crate) const SIGBUS: c_int = linux_raw_sys::general::SIGBUS as _;
pub(crate) const SIGFPE: c_int = linux_raw_sys::general::SIGFPE as _;
pub(crate) const SIGKILL: c_int = linux_raw_sys::general::SIGKILL as _;
pub(crate) const SIGUSR1: c_int = linux_raw_sys::general::SIGUSR1 as _;
pub(crate) const SIGSEGV: c_int = linux_raw_sys::general::SIGSEGV as _;
pub(crate) const SIGUSR2: c_int = linux_raw_sys::general::SIGUSR2 as _;
pub(crate) const SIGPIPE: c_int = linux_raw_sys::general::SIGPIPE as _;
pub(crate) const SIGALRM: c_int = linux_raw_sys::general::SIGALRM as _;
pub(crate) const SIGTERM: c_int = linux_raw_sys::general::SIGTERM as _;
#[cfg(not(any(
    target_arch = "mips",
    target_arch = "mips64",
    target_arch = "sparc",
    target_arch = "sparc64"
)))]
pub(crate) const SIGSTKFLT: c_int = linux_raw_sys::general::SIGSTKFLT as _;
pub(crate) const SIGCHLD: c_int = linux_raw_sys::general::SIGCHLD as _;
pub(crate) const SIGCONT: c_int = linux_raw_sys::general::SIGCONT as _;
pub(crate) const SIGSTOP: c_int = linux_raw_sys::general::SIGSTOP as _;
pub(crate) const SIGTSTP: c_int = linux_raw_sys::general::SIGTSTP as _;
pub(crate) const SIGTTIN: c_int = linux_raw_sys::general::SIGTTIN as _;
pub(crate) const SIGTTOU: c_int = linux_raw_sys::general::SIGTTOU as _;
pub(crate) const SIGURG: c_int = linux_raw_sys::general::SIGURG as _;
pub(crate) const SIGXCPU: c_int = linux_raw_sys::general::SIGXCPU as _;
pub(crate) const SIGXFSZ: c_int = linux_raw_sys::general::SIGXFSZ as _;
pub(crate) const SIGVTALRM: c_int = linux_raw_sys::general::SIGVTALRM as _;
pub(crate) const SIGPROF: c_int = linux_raw_sys::general::SIGPROF as _;
pub(crate) const SIGWINCH: c_int = linux_raw_sys::general::SIGWINCH as _;
pub(crate) const SIGIO: c_int = linux_raw_sys::general::SIGIO as _;
pub(crate) const SIGPWR: c_int = linux_raw_sys::general::SIGPWR as _;
pub(crate) const SIGSYS: c_int = linux_raw_sys::general::SIGSYS as _;
#[cfg(any(
    target_arch = "mips",
    target_arch = "mips64",
    target_arch = "sparc",
    target_arch = "sparc64"
))]
pub(crate) const SIGEMT: c_int = linux_raw_sys::general::SIGEMT as _;

#[cfg(feature = "stdio")]
pub(crate) const STDIN_FILENO: c_int = linux_raw_sys::general::STDIN_FILENO as _;
#[cfg(feature = "stdio")]
pub(crate) const STDOUT_FILENO: c_int = linux_raw_sys::general::STDOUT_FILENO as _;
#[cfg(feature = "stdio")]
pub(crate) const STDERR_FILENO: c_int = linux_raw_sys::general::STDERR_FILENO as _;

pub(crate) const PIPE_BUF: usize = linux_raw_sys::general::PIPE_BUF as _;

pub(crate) const CLOCK_MONOTONIC: c_int = linux_raw_sys::general::CLOCK_MONOTONIC as _;
pub(crate) const CLOCK_REALTIME: c_int = linux_raw_sys::general::CLOCK_REALTIME as _;
pub(crate) const CLOCK_MONOTONIC_RAW: c_int = linux_raw_sys::general::CLOCK_MONOTONIC_RAW as _;
pub(crate) const CLOCK_MONOTONIC_COARSE: c_int =
    linux_raw_sys::general::CLOCK_MONOTONIC_COARSE as _;
pub(crate) const CLOCK_REALTIME_COARSE: c_int = linux_raw_sys::general::CLOCK_REALTIME_COARSE as _;
pub(crate) const CLOCK_THREAD_CPUTIME_ID: c_int =
    linux_raw_sys::general::CLOCK_THREAD_CPUTIME_ID as _;
pub(crate) const CLOCK_PROCESS_CPUTIME_ID: c_int =
    linux_raw_sys::general::CLOCK_PROCESS_CPUTIME_ID as _;
