//! Terminal I/O stream operations.

mod cf;
mod constants;
mod tc;
#[cfg(not(windows))]
mod tty;

pub use cf::{cfgetispeed, cfgetospeed, cfmakeraw, cfsetispeed, cfsetospeed, cfsetspeed};
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
)))]
pub use constants::B1000000;
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
)))]
pub use constants::B1152000;
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
)))]
pub use constants::B1500000;
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
)))]
pub use constants::B2000000;
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
)))]
pub use constants::B2500000;
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
)))]
pub use constants::B3000000;
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
)))]
pub use constants::B3500000;
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
)))]
pub use constants::B4000000;
#[cfg(not(any(target_os = "ios", target_os = "macos", target_os = "openbsd")))]
pub use constants::B460800;
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "illumos",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
)))]
pub use constants::B500000;
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "illumos",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
)))]
pub use constants::B576000;
#[cfg(not(any(target_os = "ios", target_os = "macos", target_os = "openbsd")))]
pub use constants::B921600;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use constants::BRKINT;
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
pub use constants::BS0;
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
pub use constants::BS1;
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
pub use constants::BSDLY;
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "redox",
)))]
pub use constants::CBAUD;
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
pub use constants::CBAUDEX;
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
pub use constants::CIBAUD;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use constants::CLOCAL;
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
pub use constants::CMSPAR;
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
pub use constants::CR0;
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
pub use constants::CR1;
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
pub use constants::CR2;
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
pub use constants::CR3;
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
pub use constants::CRDLY;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use constants::CREAD;
#[cfg(not(any(target_os = "ios", target_os = "macos", target_os = "redox")))]
pub use constants::CRTSCTS;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use constants::CS5;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use constants::CS6;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use constants::CS7;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use constants::CS8;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use constants::CSIZE;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use constants::CSTOPB;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use constants::ECHO;
#[cfg(not(any(target_os = "ios", target_os = "macos", target_os = "redox")))]
pub use constants::ECHOCTL;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use constants::ECHOE;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use constants::ECHOK;
#[cfg(not(any(target_os = "ios", target_os = "macos", target_os = "redox")))]
pub use constants::ECHOKE;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use constants::ECHONL;
#[cfg(not(any(target_os = "ios", target_os = "macos", target_os = "redox")))]
pub use constants::ECHOPRT;
#[cfg(not(any(
    target_os = "emscripten",
    target_os = "illumos",
    target_os = "ios",
    target_os = "macos",
    target_os = "redox",
)))]
pub use constants::EXTA;
#[cfg(not(any(
    target_os = "emscripten",
    target_os = "illumos",
    target_os = "ios",
    target_os = "macos",
    target_os = "redox",
)))]
pub use constants::EXTB;
#[cfg(not(any(target_os = "ios", target_os = "macos", target_os = "redox")))]
pub use constants::EXTPROC;
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
pub use constants::FF0;
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
pub use constants::FF1;
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
pub use constants::FFDLY;
#[cfg(not(any(target_os = "ios", target_os = "macos", target_os = "redox")))]
pub use constants::FLUSHO;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use constants::HUPCL;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use constants::ICRNL;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use constants::IEXTEN;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use constants::IGNBRK;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use constants::IGNCR;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use constants::IGNPAR;
#[cfg(not(any(target_os = "ios", target_os = "macos", target_os = "redox")))]
pub use constants::IMAXBEL;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use constants::INLCR;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use constants::INPCK;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use constants::ISIG;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use constants::ISTRIP;
#[cfg(any(
    linux_raw,
    all(
        libc,
        any(target_os = "haiku", target_os = "illumos", target_os = "solaris"),
    )
))]
pub use constants::IUCLC;
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
pub use constants::IUTF8;
#[cfg(not(any(target_os = "ios", target_os = "macos", target_os = "redox")))]
pub use constants::IXANY;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use constants::IXOFF;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use constants::IXON;
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
pub use constants::NL0;
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
pub use constants::NL1;
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
pub use constants::NLDLY;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use constants::NOFLSH;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use constants::OCRNL;
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
)))]
pub use constants::OFDEL;
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
)))]
pub use constants::OFILL;
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "redox",
)))]
pub use constants::OLCUC;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use constants::ONLCR;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use constants::ONLRET;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use constants::ONOCR;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use constants::OPOST;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use constants::PARENB;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use constants::PARMRK;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use constants::PARODD;
#[cfg(not(any(target_os = "ios", target_os = "macos", target_os = "redox")))]
pub use constants::PENDIN;
#[cfg(not(any(
    target_os = "fuchsia",
    target_os = "illumos",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "redox",
)))]
pub use constants::TAB0;
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
pub use constants::TAB1;
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
pub use constants::TAB2;
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
pub use constants::TAB3;
#[cfg(not(any(
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "illumos",
    target_os = "redox",
)))]
pub use constants::TABDLY;
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use constants::TOSTOP;
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "illumos",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
)))]
pub use constants::VSWTC;
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
pub use constants::VT0;
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
pub use constants::VT1;
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
pub use constants::VTDLY;
#[cfg(any(linux_raw, all(libc, any(target_arch = "s390x", target_os = "haiku"))))]
pub use constants::XCASE;
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
pub use constants::XTABS;
pub use constants::{
    speed_value, B0, B110, B115200, B1200, B134, B150, B1800, B19200, B200, B230400, B2400, B300,
    B38400, B4800, B50, B57600, B600, B75, B9600, ICANON, VDISCARD, VEOF, VEOL, VEOL2, VERASE,
    VINTR, VKILL, VLNEXT, VMIN, VQUIT, VREPRINT, VSTART, VSTOP, VSUSP, VTIME, VWERASE,
};
pub use tc::{
    tcdrain, tcflow, tcflush, tcgetattr, tcgetpgrp, tcgetsid, tcgetwinsize, tcsendbreak, tcsetattr,
    tcsetpgrp, tcsetwinsize, Action, OptionalActions, QueueSelector, Speed, Tcflag, Termios,
    Winsize,
};
#[cfg(not(windows))]
pub use tty::isatty;
#[cfg(not(any(target_os = "fuchsia", target_os = "wasi")))]
#[cfg(feature = "procfs")]
pub use tty::ttyname;
