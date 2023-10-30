use crate::backend;
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
)))]
pub use backend::termios::types::B1000000;
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
)))]
pub use backend::termios::types::B1152000;
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
)))]
pub use backend::termios::types::B1500000;
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
)))]
pub use backend::termios::types::B2000000;
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
)))]
pub use backend::termios::types::B2500000;
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
)))]
pub use backend::termios::types::B3000000;
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
)))]
pub use backend::termios::types::B3500000;
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
)))]
pub use backend::termios::types::B4000000;
#[cfg(not(any(target_os = "ios", target_os = "macos", target_os = "openbsd")))]
pub use backend::termios::types::B460800;
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "illumos",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
)))]
pub use backend::termios::types::B500000;
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "illumos",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
)))]
pub use backend::termios::types::B576000;
#[cfg(not(any(target_os = "ios", target_os = "macos", target_os = "openbsd")))]
pub use backend::termios::types::B921600;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use backend::termios::types::BRKINT;
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "fuchsia",
    target_os = "illumos",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "redox",
)))]
pub use backend::termios::types::BS0;
#[cfg(not(any(
    all(libc, target_env = "musl"),
    target_os = "dragonfly",
    target_os = "emscripten",
    target_os = "freebsd",
    target_os = "fuchsia",
    target_os = "illumos",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "redox",
)))]
pub use backend::termios::types::BS1;
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "illumos",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "redox",
)))]
pub use backend::termios::types::BSDLY;
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "redox",
)))]
pub use backend::termios::types::CBAUD;
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "illumos",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "redox",
)))]
pub use backend::termios::types::CBAUDEX;
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "emscripten",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "redox",
)))]
pub use backend::termios::types::CIBAUD;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use backend::termios::types::CLOCAL;
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "emscripten",
    target_os = "freebsd",
    target_os = "illumos",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "redox",
)))]
pub use backend::termios::types::CMSPAR;
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "fuchsia",
    target_os = "illumos",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "redox",
)))]
pub use backend::termios::types::CR0;
#[cfg(not(any(
    all(libc, target_env = "musl"),
    target_os = "dragonfly",
    target_os = "emscripten",
    target_os = "freebsd",
    target_os = "fuchsia",
    target_os = "illumos",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "redox",
)))]
pub use backend::termios::types::CR1;
#[cfg(not(any(
    all(libc, target_env = "musl"),
    target_os = "dragonfly",
    target_os = "emscripten",
    target_os = "freebsd",
    target_os = "fuchsia",
    target_os = "illumos",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "redox",
)))]
pub use backend::termios::types::CR2;
#[cfg(not(any(
    all(libc, target_env = "musl"),
    target_os = "dragonfly",
    target_os = "emscripten",
    target_os = "freebsd",
    target_os = "fuchsia",
    target_os = "illumos",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "redox",
)))]
pub use backend::termios::types::CR3;
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "illumos",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "redox",
)))]
pub use backend::termios::types::CRDLY;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use backend::termios::types::CREAD;
#[cfg(not(any(target_os = "ios", target_os = "macos", target_os = "redox")))]
pub use backend::termios::types::CRTSCTS;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use backend::termios::types::CS5;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use backend::termios::types::CS6;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use backend::termios::types::CS7;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use backend::termios::types::CS8;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use backend::termios::types::CSIZE;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use backend::termios::types::CSTOPB;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use backend::termios::types::ECHO;
#[cfg(not(any(target_os = "ios", target_os = "macos", target_os = "redox")))]
pub use backend::termios::types::ECHOCTL;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use backend::termios::types::ECHOE;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use backend::termios::types::ECHOK;
#[cfg(not(any(target_os = "ios", target_os = "macos", target_os = "redox")))]
pub use backend::termios::types::ECHOKE;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use backend::termios::types::ECHONL;
#[cfg(not(any(target_os = "ios", target_os = "macos", target_os = "redox")))]
pub use backend::termios::types::ECHOPRT;
#[cfg(not(any(
    target_os = "emscripten",
    target_os = "illumos",
    target_os = "ios",
    target_os = "macos",
    target_os = "redox",
)))]
pub use backend::termios::types::EXTA;
#[cfg(not(any(
    target_os = "emscripten",
    target_os = "illumos",
    target_os = "ios",
    target_os = "macos",
    target_os = "redox",
)))]
pub use backend::termios::types::EXTB;
#[cfg(not(any(target_os = "ios", target_os = "macos", target_os = "redox")))]
pub use backend::termios::types::EXTPROC;
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "fuchsia",
    target_os = "illumos",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "redox",
)))]
pub use backend::termios::types::FF0;
#[cfg(not(any(
    all(libc, target_env = "musl"),
    target_os = "dragonfly",
    target_os = "emscripten",
    target_os = "freebsd",
    target_os = "fuchsia",
    target_os = "illumos",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "redox",
)))]
pub use backend::termios::types::FF1;
#[cfg(not(any(
    all(libc, target_env = "musl"),
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "illumos",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "redox",
)))]
pub use backend::termios::types::FFDLY;
#[cfg(not(any(target_os = "ios", target_os = "macos", target_os = "redox")))]
pub use backend::termios::types::FLUSHO;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use backend::termios::types::HUPCL;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use backend::termios::types::ICRNL;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use backend::termios::types::IEXTEN;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use backend::termios::types::IGNBRK;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use backend::termios::types::IGNCR;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use backend::termios::types::IGNPAR;
#[cfg(not(any(target_os = "ios", target_os = "macos", target_os = "redox")))]
pub use backend::termios::types::IMAXBEL;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use backend::termios::types::INLCR;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use backend::termios::types::INPCK;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use backend::termios::types::ISIG;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use backend::termios::types::ISTRIP;
#[cfg(any(
    linux_raw,
    all(
        libc,
        any(target_os = "haiku", target_os = "illumos", target_os = "solaris"),
    )
))]
pub use backend::termios::types::IUCLC;
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "emscripten",
    target_os = "freebsd",
    target_os = "illumos",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "redox",
)))]
pub use backend::termios::types::IUTF8;
#[cfg(not(any(target_os = "ios", target_os = "macos", target_os = "redox")))]
pub use backend::termios::types::IXANY;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use backend::termios::types::IXOFF;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use backend::termios::types::IXON;
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "fuchsia",
    target_os = "illumos",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "redox",
)))]
pub use backend::termios::types::NL0;
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "fuchsia",
    target_os = "illumos",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "redox",
)))]
pub use backend::termios::types::NL1;
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "illumos",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "redox",
)))]
pub use backend::termios::types::NLDLY;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use backend::termios::types::NOFLSH;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use backend::termios::types::OCRNL;
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
)))]
pub use backend::termios::types::OFDEL;
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
)))]
pub use backend::termios::types::OFILL;
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "redox",
)))]
pub use backend::termios::types::OLCUC;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use backend::termios::types::ONLCR;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use backend::termios::types::ONLRET;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use backend::termios::types::ONOCR;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use backend::termios::types::OPOST;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use backend::termios::types::PARENB;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use backend::termios::types::PARMRK;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use backend::termios::types::PARODD;
#[cfg(not(any(target_os = "ios", target_os = "macos", target_os = "redox")))]
pub use backend::termios::types::PENDIN;
#[cfg(not(any(
    target_os = "fuchsia",
    target_os = "illumos",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "redox",
)))]
pub use backend::termios::types::TAB0;
#[cfg(not(any(
    all(libc, target_env = "musl"),
    target_os = "dragonfly",
    target_os = "emscripten",
    target_os = "freebsd",
    target_os = "fuchsia",
    target_os = "illumos",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "redox",
)))]
pub use backend::termios::types::TAB1;
#[cfg(not(any(
    all(libc, target_env = "musl"),
    target_os = "dragonfly",
    target_os = "emscripten",
    target_os = "freebsd",
    target_os = "fuchsia",
    target_os = "illumos",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "redox",
)))]
pub use backend::termios::types::TAB2;
#[cfg(not(any(
    all(libc, target_env = "musl"),
    target_os = "emscripten",
    target_os = "fuchsia",
    target_os = "illumos",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "redox",
)))]
pub use backend::termios::types::TAB3;
#[cfg(not(any(
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "illumos",
    target_os = "redox",
)))]
pub use backend::termios::types::TABDLY;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use backend::termios::types::TOSTOP;
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "illumos",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
)))]
pub use backend::termios::types::VSWTC;
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "fuchsia",
    target_os = "illumos",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "redox",
)))]
pub use backend::termios::types::VT0;
#[cfg(not(any(
    all(libc, target_env = "musl"),
    target_os = "dragonfly",
    target_os = "emscripten",
    target_os = "freebsd",
    target_os = "fuchsia",
    target_os = "illumos",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "redox",
)))]
pub use backend::termios::types::VT1;
#[cfg(not(any(
    all(libc, target_env = "musl"),
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "illumos",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "redox",
)))]
pub use backend::termios::types::VTDLY;
#[cfg(any(linux_raw, all(libc, any(target_arch = "s390x", target_os = "haiku"))))]
pub use backend::termios::types::XCASE;
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "illumos",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "redox",
)))]
pub use backend::termios::types::XTABS;
pub use backend::termios::types::{
    B0, B110, B115200, B1200, B134, B150, B1800, B19200, B200, B230400, B2400, B300, B38400, B4800,
    B50, B57600, B600, B75, B9600, ICANON, VDISCARD, VEOF, VEOL, VEOL2, VERASE, VINTR, VKILL,
    VLNEXT, VMIN, VQUIT, VREPRINT, VSTART, VSTOP, VSUSP, VTIME, VWERASE,
};

/// Translate from a `Speed` code to a speed value `u32`.
///
/// ```rust
/// let speed = rustix::termios::speed_value(rustix::termios::B57600);
/// assert_eq!(speed, Some(57600));
/// ```
pub fn speed_value(speed: backend::termios::types::Speed) -> Option<u32> {
    match speed {
        backend::termios::types::B0 => Some(0),
        backend::termios::types::B50 => Some(50),
        backend::termios::types::B75 => Some(75),
        backend::termios::types::B110 => Some(110),
        backend::termios::types::B134 => Some(134),
        backend::termios::types::B150 => Some(150),
        backend::termios::types::B200 => Some(200),
        backend::termios::types::B300 => Some(300),
        backend::termios::types::B600 => Some(600),
        backend::termios::types::B1200 => Some(1200),
        backend::termios::types::B1800 => Some(1800),
        backend::termios::types::B2400 => Some(2400),
        backend::termios::types::B4800 => Some(4800),
        backend::termios::types::B9600 => Some(9600),
        backend::termios::types::B19200 => Some(19200),
        backend::termios::types::B38400 => Some(38400),
        backend::termios::types::B57600 => Some(57600),
        backend::termios::types::B115200 => Some(115_200),
        backend::termios::types::B230400 => Some(230_400),
        #[cfg(not(any(target_os = "ios", target_os = "macos", target_os = "openbsd")))]
        backend::termios::types::B460800 => Some(460_800),
        #[cfg(not(any(
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "illumos",
            target_os = "ios",
            target_os = "macos",
            target_os = "netbsd",
            target_os = "openbsd",
        )))]
        backend::termios::types::B500000 => Some(500_000),
        #[cfg(not(any(
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "illumos",
            target_os = "ios",
            target_os = "macos",
            target_os = "netbsd",
            target_os = "openbsd",
        )))]
        backend::termios::types::B576000 => Some(576_000),
        #[cfg(not(any(target_os = "ios", target_os = "macos", target_os = "openbsd")))]
        backend::termios::types::B921600 => Some(921_600),
        #[cfg(not(any(
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "ios",
            target_os = "macos",
            target_os = "netbsd",
            target_os = "openbsd",
        )))]
        backend::termios::types::B1000000 => Some(1_000_000),
        #[cfg(not(any(
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "ios",
            target_os = "macos",
            target_os = "netbsd",
            target_os = "openbsd",
        )))]
        backend::termios::types::B1152000 => Some(1_152_000),
        #[cfg(not(any(
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "ios",
            target_os = "macos",
            target_os = "netbsd",
            target_os = "openbsd",
        )))]
        backend::termios::types::B1500000 => Some(1_500_000),
        #[cfg(not(any(
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "ios",
            target_os = "macos",
            target_os = "netbsd",
            target_os = "openbsd",
        )))]
        backend::termios::types::B2000000 => Some(2_000_000),
        #[cfg(not(any(
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "ios",
            target_os = "macos",
            target_os = "netbsd",
            target_os = "openbsd",
        )))]
        backend::termios::types::B2500000 => Some(2_500_000),
        #[cfg(not(any(
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "ios",
            target_os = "macos",
            target_os = "netbsd",
            target_os = "openbsd",
        )))]
        backend::termios::types::B3000000 => Some(3_000_000),
        #[cfg(not(any(
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "ios",
            target_os = "macos",
            target_os = "netbsd",
            target_os = "openbsd",
        )))]
        backend::termios::types::B3500000 => Some(3_500_000),
        #[cfg(not(any(
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "ios",
            target_os = "macos",
            target_os = "netbsd",
            target_os = "openbsd",
        )))]
        backend::termios::types::B4000000 => Some(4_000_000),
        _ => None,
    }
}
