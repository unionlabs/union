use super::super::c;

/// `TCSA*` values for use with [`tcsetattr`].
///
/// [`tcsetattr`]: crate::termios::tcsetattr
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[repr(i32)]
pub enum OptionalActions {
    /// `TCSANOW`—Make the change immediately.
    Now = c::TCSANOW,

    /// `TCSADRAIN`—Make the change after all output has been transmitted.
    Drain = c::TCSADRAIN,

    /// `TCSAFLUSH`—Discard any pending input and then make the change
    /// after all output has been transmitted.
    Flush = c::TCSAFLUSH,
}

/// `TC*` values for use with [`tcflush`].
///
/// [`tcflush`]: crate::termios::tcflush
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[repr(i32)]
pub enum QueueSelector {
    /// `TCIFLUSH`—Flush data received but not read.
    IFlush = c::TCIFLUSH,

    /// `TCOFLUSH`—Flush data written but not transmitted.
    OFlush = c::TCOFLUSH,

    /// `TCIOFLUSH`—`IFlush` and `OFlush` combined.
    IOFlush = c::TCIOFLUSH,
}

/// `TC*` values for use with [`tcflow`].
///
/// [`tcflow`]: crate::termios::tcflow
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[repr(i32)]
pub enum Action {
    /// `TCOOFF`—Suspend output.
    OOff = c::TCOOFF,

    /// `TCOON`—Restart suspended output.
    OOn = c::TCOON,

    /// `TCIOFF`—Transmits a STOP byte.
    IOff = c::TCIOFF,

    /// `TCION`—Transmits a START byte.
    IOn = c::TCION,
}

/// `struct termios` for use with [`tcgetattr`].
///
/// [`tcgetattr`]: crate::termios::tcgetattr
pub type Termios = c::termios;

/// `struct winsize` for use with [`tcgetwinsize`].
///
/// [`tcgetwinsize`]: crate::termios::tcgetwinsize
pub type Winsize = c::winsize;

/// `tcflag_t`—A type for the flags fields of [`Termios`].
pub type Tcflag = c::tcflag_t;

/// `speed_t`—A return type for [`cfsetspeed`] and similar.
///
/// [`cfsetspeed`]: crate::termios::cfsetspeed
pub type Speed = c::speed_t;

/// `VINTR`
pub const VINTR: usize = c::VINTR as usize;

/// `VQUIT`
pub const VQUIT: usize = c::VQUIT as usize;

/// `VERASE`
pub const VERASE: usize = c::VERASE as usize;

/// `VKILL`
pub const VKILL: usize = c::VKILL as usize;

/// `VEOF`
pub const VEOF: usize = c::VEOF as usize;

/// `VTIME`
pub const VTIME: usize = c::VTIME as usize;

/// `VMIN`
pub const VMIN: usize = c::VMIN as usize;

/// `VSWTC`
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "illumos",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
)))]
pub const VSWTC: usize = c::VSWTC as usize;

/// `VSTART`
pub const VSTART: usize = c::VSTART as usize;

/// `VSTOP`
pub const VSTOP: usize = c::VSTOP as usize;

/// `VSUSP`
pub const VSUSP: usize = c::VSUSP as usize;

/// `VEOL`
pub const VEOL: usize = c::VEOL as usize;

/// `VREPRINT`
pub const VREPRINT: usize = c::VREPRINT as usize;

/// `VDISCARD`
pub const VDISCARD: usize = c::VDISCARD as usize;

/// `VWERASE`
pub const VWERASE: usize = c::VWERASE as usize;

/// `VLNEXT`
pub const VLNEXT: usize = c::VLNEXT as usize;

/// `VEOL2`
pub const VEOL2: usize = c::VEOL2 as usize;

/// `IGNBRK`
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub const IGNBRK: c::c_uint = c::IGNBRK;

/// `BRKINT`
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub const BRKINT: c::c_uint = c::BRKINT;

/// `IGNPAR`
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub const IGNPAR: c::c_uint = c::IGNPAR;

/// `PARMRK`
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub const PARMRK: c::c_uint = c::PARMRK;

/// `INPCK`
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub const INPCK: c::c_uint = c::INPCK;

/// `ISTRIP`
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub const ISTRIP: c::c_uint = c::ISTRIP;

/// `INLCR`
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub const INLCR: c::c_uint = c::INLCR;

/// `IGNCR`
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub const IGNCR: c::c_uint = c::IGNCR;

/// `ICRNL`
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub const ICRNL: c::c_uint = c::ICRNL;

/// `IUCLC`
#[cfg(any(target_os = "haiku", target_os = "illumos", target_os = "solaris"))]
pub const IUCLC: c::c_uint = c::IUCLC;

/// `IXON`
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub const IXON: c::c_uint = c::IXON;

/// `IXANY`
#[cfg(not(any(target_os = "ios", target_os = "macos", target_os = "redox")))]
pub const IXANY: c::c_uint = c::IXANY;

/// `IXOFF`
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub const IXOFF: c::c_uint = c::IXOFF;

/// `IMAXBEL`
#[cfg(not(any(target_os = "ios", target_os = "macos", target_os = "redox")))]
pub const IMAXBEL: c::c_uint = c::IMAXBEL;

/// `IUTF8`
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
pub const IUTF8: c::c_uint = c::IUTF8;

/// `OPOST`
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub const OPOST: c::c_uint = c::OPOST;

/// `OLCUC`
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "redox",
)))]
pub const OLCUC: c::c_uint = c::OLCUC;

/// `ONLCR`
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub const ONLCR: c::c_uint = c::ONLCR;

/// `OCRNL`
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub const OCRNL: c::c_uint = c::OCRNL;

/// `ONOCR`
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub const ONOCR: c::c_uint = c::ONOCR;

/// `ONLRET`
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub const ONLRET: c::c_uint = c::ONLRET;

/// `OFILL`
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
)))]
pub const OFILL: c::c_uint = c::OFILL;

/// `OFDEL`
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
)))]
pub const OFDEL: c::c_uint = c::OFDEL;

/// `NLDLY`
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
pub const NLDLY: c::c_uint = c::NLDLY;

/// `NL0`
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
pub const NL0: c::c_uint = c::NL0;

/// `NL1`
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
pub const NL1: c::c_uint = c::NL1;

/// `CRDLY`
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
pub const CRDLY: c::c_uint = c::CRDLY;

/// `CR0`
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
pub const CR0: c::c_uint = c::CR0;

/// `CR1`
#[cfg(not(any(
    target_env = "musl",
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
pub const CR1: c::c_uint = c::CR1;

/// `CR2`
#[cfg(not(any(
    target_env = "musl",
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
pub const CR2: c::c_uint = c::CR2;

/// `CR3`
#[cfg(not(any(
    target_env = "musl",
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
pub const CR3: c::c_uint = c::CR3;

/// `TABDLY`
#[cfg(not(any(
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "illumos",
    target_os = "redox",
)))]
pub const TABDLY: c::c_uint = c::TABDLY;

/// `TAB0`
#[cfg(not(any(
    target_os = "fuchsia",
    target_os = "illumos",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "redox",
)))]
pub const TAB0: c::c_uint = c::TAB0;

/// `TAB1`
#[cfg(not(any(
    target_env = "musl",
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
pub const TAB1: c::c_uint = c::TAB1;

/// `TAB2`
#[cfg(not(any(
    target_env = "musl",
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
pub const TAB2: c::c_uint = c::TAB2;

/// `TAB3`
#[cfg(not(any(
    target_env = "musl",
    target_os = "emscripten",
    target_os = "fuchsia",
    target_os = "illumos",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "redox",
)))]
pub const TAB3: c::c_uint = c::TAB3;

/// `BSDLY`
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
pub const BSDLY: c::c_uint = c::BSDLY;

/// `BS0`
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
pub const BS0: c::c_uint = c::BS0;

/// `BS1`
#[cfg(not(any(
    target_env = "musl",
    target_os = "emscripten",
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
pub const BS1: c::c_uint = c::BS1;

/// `FFDLY`
#[cfg(not(any(
    target_env = "musl",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "illumos",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "redox",
)))]
pub const FFDLY: c::c_uint = c::FFDLY;

/// `FF0`
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
pub const FF0: c::c_uint = c::FF0;

/// `FF1`
#[cfg(not(any(
    target_env = "musl",
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
pub const FF1: c::c_uint = c::FF1;

/// `VTDLY`
#[cfg(not(any(
    target_env = "musl",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "illumos",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "redox",
)))]
pub const VTDLY: c::c_uint = c::VTDLY;

/// `VT0`
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
pub const VT0: c::c_uint = c::VT0;

/// `VT1`
#[cfg(not(any(
    target_env = "musl",
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
pub const VT1: c::c_uint = c::VT1;

/// `B0`
pub const B0: Speed = c::B0;

/// `B50`
pub const B50: Speed = c::B50;

/// `B75`
pub const B75: Speed = c::B75;

/// `B110`
pub const B110: Speed = c::B110;

/// `B134`
pub const B134: Speed = c::B134;

/// `B150`
pub const B150: Speed = c::B150;

/// `B200`
pub const B200: Speed = c::B200;

/// `B300`
pub const B300: Speed = c::B300;

/// `B600`
pub const B600: Speed = c::B600;

/// `B1200`
pub const B1200: Speed = c::B1200;

/// `B1800`
pub const B1800: Speed = c::B1800;

/// `B2400`
pub const B2400: Speed = c::B2400;

/// `B4800`
pub const B4800: Speed = c::B4800;

/// `B9600`
pub const B9600: Speed = c::B9600;

/// `B19200`
pub const B19200: Speed = c::B19200;

/// `B38400`
pub const B38400: Speed = c::B38400;

/// `B57600`
pub const B57600: Speed = c::B57600;

/// `B115200`
pub const B115200: Speed = c::B115200;

/// `B230400`
pub const B230400: Speed = c::B230400;

/// `B460800`
#[cfg(not(any(target_os = "ios", target_os = "macos", target_os = "openbsd")))]
pub const B460800: Speed = c::B460800;

/// `B500000`
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "illumos",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
)))]
pub const B500000: Speed = c::B500000;

/// `B576000`
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "illumos",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
)))]
pub const B576000: Speed = c::B576000;

/// `B921600`
#[cfg(not(any(target_os = "ios", target_os = "macos", target_os = "openbsd")))]
pub const B921600: Speed = c::B921600;

/// `B1000000`
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
)))]
pub const B1000000: Speed = c::B1000000;

/// `B1152000`
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
)))]
pub const B1152000: Speed = c::B1152000;

/// `B1500000`
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
)))]
pub const B1500000: Speed = c::B1500000;

/// `B2000000`
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
)))]
pub const B2000000: Speed = c::B2000000;

/// `B2500000`
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
)))]
pub const B2500000: Speed = c::B2500000;

/// `B3000000`
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
)))]
pub const B3000000: Speed = c::B3000000;

/// `B3500000`
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
)))]
pub const B3500000: Speed = c::B3500000;

/// `B4000000`
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
)))]
pub const B4000000: Speed = c::B4000000;

/// `CSIZE`
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub const CSIZE: c::c_uint = c::CSIZE;

/// `CS5`
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub const CS5: c::c_uint = c::CS5;

/// `CS6`
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub const CS6: c::c_uint = c::CS6;

/// `CS7`
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub const CS7: c::c_uint = c::CS7;

/// `CS8`
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub const CS8: c::c_uint = c::CS8;

/// `CSTOPB`
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub const CSTOPB: c::c_uint = c::CSTOPB;

/// `CREAD`
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub const CREAD: c::c_uint = c::CREAD;

/// `PARENB`
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub const PARENB: c::c_uint = c::PARENB;

/// `PARODD`
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub const PARODD: c::c_uint = c::PARODD;

/// `HUPCL`
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub const HUPCL: c::c_uint = c::HUPCL;

/// `CLOCAL`
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub const CLOCAL: c::c_uint = c::CLOCAL;

/// `ISIG`
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub const ISIG: c::c_uint = c::ISIG;

/// `ICANON`—A flag for the `c_lflag` field of [`Termios`] indicating
/// canonical mode.
pub const ICANON: Tcflag = c::ICANON;

/// `ECHO`
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub const ECHO: c::c_uint = c::ECHO;

/// `ECHOE`
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub const ECHOE: c::c_uint = c::ECHOE;

/// `ECHOK`
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub const ECHOK: c::c_uint = c::ECHOK;

/// `ECHONL`
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub const ECHONL: c::c_uint = c::ECHONL;

/// `NOFLSH`
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub const NOFLSH: c::c_uint = c::NOFLSH;

/// `TOSTOP`
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub const TOSTOP: c::c_uint = c::TOSTOP;

/// `IEXTEN`
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub const IEXTEN: c::c_uint = c::IEXTEN;

/// `EXTA`
#[cfg(not(any(
    target_os = "emscripten",
    target_os = "illumos",
    target_os = "ios",
    target_os = "macos",
    target_os = "redox",
)))]
pub const EXTA: c::c_uint = c::EXTA;

/// `EXTB`
#[cfg(not(any(
    target_os = "emscripten",
    target_os = "illumos",
    target_os = "ios",
    target_os = "macos",
    target_os = "redox",
)))]
pub const EXTB: c::c_uint = c::EXTB;

/// `CBAUD`
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "redox",
)))]
pub const CBAUD: c::c_uint = c::CBAUD;

/// `CBAUDEX`
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
pub const CBAUDEX: c::c_uint = c::CBAUDEX;

/// `CIBAUD`
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "emscripten",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "redox",
    target_arch = "powerpc",
    target_arch = "powerpc64",
)))]
pub const CIBAUD: c::tcflag_t = c::CIBAUD;

/// `CIBAUD`
// TODO: Upstream this.
#[cfg(any(target_arch = "powerpc", target_arch = "powerpc64"))]
pub const CIBAUD: c::tcflag_t = 0o77600000;

/// `CMSPAR`
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
pub const CMSPAR: c::c_uint = c::CMSPAR;

/// `CRTSCTS`
#[cfg(not(any(target_os = "ios", target_os = "macos", target_os = "redox")))]
pub const CRTSCTS: c::c_uint = c::CRTSCTS;

/// `XCASE`
#[cfg(any(target_arch = "s390x", target_os = "haiku"))]
pub const XCASE: c::c_uint = c::XCASE;

/// `ECHOCTL`
#[cfg(not(any(target_os = "ios", target_os = "macos", target_os = "redox")))]
pub const ECHOCTL: c::c_uint = c::ECHOCTL;

/// `ECHOPRT`
#[cfg(not(any(target_os = "ios", target_os = "macos", target_os = "redox")))]
pub const ECHOPRT: c::c_uint = c::ECHOPRT;

/// `ECHOKE`
#[cfg(not(any(target_os = "ios", target_os = "macos", target_os = "redox")))]
pub const ECHOKE: c::c_uint = c::ECHOKE;

/// `FLUSHO`
#[cfg(not(any(target_os = "ios", target_os = "macos", target_os = "redox")))]
pub const FLUSHO: c::c_uint = c::FLUSHO;

/// `PENDIN`
#[cfg(not(any(target_os = "ios", target_os = "macos", target_os = "redox")))]
pub const PENDIN: c::c_uint = c::PENDIN;

/// `EXTPROC`
#[cfg(not(any(target_os = "ios", target_os = "macos", target_os = "redox")))]
pub const EXTPROC: c::c_uint = c::EXTPROC;

/// `XTABS`
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
pub const XTABS: c::c_uint = c::XTABS;
