use super::super::c;

/// `TCSA*` values for use with [`tcsetattr`].
///
/// [`tcsetattr`]: crate::termios::tcsetattr
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[repr(i32)]
pub enum OptionalActions {
    /// `TCSANOW`—Make the change immediately.
    #[doc(alias = "TCSANOW")]
    Now = c::TCSANOW,

    /// `TCSADRAIN`—Make the change after all output has been transmitted.
    #[doc(alias = "TCSADRAIN")]
    Drain = c::TCSADRAIN,

    /// `TCSAFLUSH`—Discard any pending input and then make the change
    /// after all output has been transmitted.
    #[doc(alias = "TCSAFLUSH")]
    Flush = c::TCSAFLUSH,
}

/// `TC*` values for use with [`tcflush`].
///
/// [`tcflush`]: crate::termios::tcflush
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[repr(i32)]
pub enum QueueSelector {
    /// `TCIFLUSH`—Flush data received but not read.
    #[doc(alias = "TCIFLUSH")]
    IFlush = c::TCIFLUSH,

    /// `TCOFLUSH`—Flush data written but not transmitted.
    #[doc(alias = "TCOFLUSH")]
    OFlush = c::TCOFLUSH,

    /// `TCIOFLUSH`—`IFlush` and `OFlush` combined.
    #[doc(alias = "TCIOFLUSH")]
    IOFlush = c::TCIOFLUSH,
}

/// `TC*` values for use with [`tcflow`].
///
/// [`tcflow`]: crate::termios::tcflow
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[repr(i32)]
pub enum Action {
    /// `TCOOFF`—Suspend output.
    #[doc(alias = "TCOOFF")]
    OOff = c::TCOOFF,

    /// `TCOON`—Restart suspended output.
    #[doc(alias = "TCOON")]
    OOn = c::TCOON,

    /// `TCIOFF`—Transmits a STOP byte.
    #[doc(alias = "TCIOFF")]
    IOff = c::TCIOFF,

    /// `TCION`—Transmits a START byte.
    #[doc(alias = "TCION")]
    IOn = c::TCION,
}

/// `struct termios` for use with [`tcgetattr`] and [`tcsetattr`].
///
/// [`tcgetattr`]: crate::termios::tcgetattr
/// [`tcsetattr`]: crate::termios::tcsetattr
#[doc(alias = "termios")]
pub type Termios = c::termios;

/// `struct termios2` for use with [`tcgetattr2`] and [`tcsetattr2`].
///
/// [`tcgetattr2`]: crate::termios::tcgetattr2
/// [`tcsetattr2`]: crate::termios::tcsetattr2
#[cfg(all(
    linux_kernel,
    any(
        target_arch = "x86",
        target_arch = "x86_64",
        target_arch = "x32",
        target_arch = "riscv64",
        target_arch = "aarch64",
        target_arch = "arm",
        target_arch = "mips",
        target_arch = "mips64",
    )
))]
#[doc(alias = "termios2")]
pub type Termios2 = c::termios2;

/// `struct winsize` for use with [`tcgetwinsize`].
///
/// [`tcgetwinsize`]: crate::termios::tcgetwinsize
#[doc(alias = "winsize")]
pub type Winsize = c::winsize;

/// `tcflag_t`—A type for the flags fields of [`Termios`].
#[doc(alias = "tcflag_t")]
pub type Tcflag = c::tcflag_t;

/// `speed_t`—A return type for [`cfsetspeed`] and similar.
///
/// [`cfsetspeed`]: crate::termios::cfsetspeed
#[doc(alias = "speed_t")]
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
#[cfg(not(any(bsd, solarish, target_os = "aix", target_os = "haiku")))]
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
#[cfg(not(target_os = "haiku"))]
pub const VREPRINT: usize = c::VREPRINT as usize;

/// `VDISCARD`
#[cfg(not(any(target_os = "aix", target_os = "haiku")))]
pub const VDISCARD: usize = c::VDISCARD as usize;

/// `VWERASE`
#[cfg(not(any(target_os = "aix", target_os = "haiku")))]
pub const VWERASE: usize = c::VWERASE as usize;

/// `VLNEXT`
#[cfg(not(target_os = "haiku"))]
pub const VLNEXT: usize = c::VLNEXT as usize;

/// `VEOL2`
pub const VEOL2: usize = c::VEOL2 as usize;

/// `IGNBRK`
pub const IGNBRK: Tcflag = c::IGNBRK;

/// `BRKINT`
pub const BRKINT: Tcflag = c::BRKINT;

/// `IGNPAR`
pub const IGNPAR: Tcflag = c::IGNPAR;

/// `PARMRK`
pub const PARMRK: Tcflag = c::PARMRK;

/// `INPCK`
pub const INPCK: Tcflag = c::INPCK;

/// `ISTRIP`
pub const ISTRIP: Tcflag = c::ISTRIP;

/// `INLCR`
pub const INLCR: Tcflag = c::INLCR;

/// `IGNCR`
pub const IGNCR: Tcflag = c::IGNCR;

/// `ICRNL`
pub const ICRNL: Tcflag = c::ICRNL;

/// `IUCLC`
#[cfg(any(solarish, target_os = "haiku"))]
pub const IUCLC: Tcflag = c::IUCLC;

/// `IXON`
pub const IXON: Tcflag = c::IXON;

/// `IXANY`
#[cfg(not(target_os = "redox"))]
pub const IXANY: Tcflag = c::IXANY;

/// `IXOFF`
pub const IXOFF: Tcflag = c::IXOFF;

/// `IMAXBEL`
#[cfg(not(any(target_os = "haiku", target_os = "redox")))]
pub const IMAXBEL: Tcflag = c::IMAXBEL;

/// `IUTF8`
#[cfg(not(any(
    freebsdlike,
    netbsdlike,
    solarish,
    target_os = "aix",
    target_os = "emscripten",
    target_os = "haiku",
    target_os = "redox",
)))]
pub const IUTF8: Tcflag = c::IUTF8;

/// `OPOST`
pub const OPOST: Tcflag = c::OPOST;

/// `OLCUC`
#[cfg(not(any(
    apple,
    freebsdlike,
    target_os = "aix",
    target_os = "netbsd",
    target_os = "redox",
)))]
pub const OLCUC: Tcflag = c::OLCUC;

/// `ONLCR`
pub const ONLCR: Tcflag = c::ONLCR;

/// `OCRNL`
pub const OCRNL: Tcflag = c::OCRNL;

/// `ONOCR`
pub const ONOCR: Tcflag = c::ONOCR;

/// `ONLRET`
pub const ONLRET: Tcflag = c::ONLRET;

/// `OFILL`
#[cfg(not(bsd))]
pub const OFILL: Tcflag = c::OFILL;

/// `OFDEL`
#[cfg(not(bsd))]
pub const OFDEL: Tcflag = c::OFDEL;

/// `NLDLY`
#[cfg(not(any(bsd, solarish, target_os = "redox")))]
pub const NLDLY: Tcflag = c::NLDLY;

/// `NL0`
#[cfg(not(any(bsd, solarish, target_os = "fuchsia", target_os = "redox")))]
pub const NL0: Tcflag = c::NL0;

/// `NL1`
#[cfg(not(any(bsd, solarish, target_os = "fuchsia", target_os = "redox")))]
pub const NL1: Tcflag = c::NL1;

/// `CRDLY`
#[cfg(not(any(bsd, solarish, target_os = "redox")))]
pub const CRDLY: Tcflag = c::CRDLY;

/// `CR0`
#[cfg(not(any(bsd, solarish, target_os = "fuchsia", target_os = "redox")))]
pub const CR0: Tcflag = c::CR0;

/// `CR1`
#[cfg(not(any(
    target_env = "musl",
    bsd,
    solarish,
    target_os = "emscripten",
    target_os = "fuchsia",
    target_os = "redox",
)))]
pub const CR1: Tcflag = c::CR1;

/// `CR2`
#[cfg(not(any(
    target_env = "musl",
    bsd,
    solarish,
    target_os = "emscripten",
    target_os = "fuchsia",
    target_os = "redox",
)))]
pub const CR2: Tcflag = c::CR2;

/// `CR3`
#[cfg(not(any(
    target_env = "musl",
    bsd,
    solarish,
    target_os = "emscripten",
    target_os = "fuchsia",
    target_os = "redox",
)))]
pub const CR3: Tcflag = c::CR3;

/// `TABDLY`
#[cfg(not(any(netbsdlike, solarish, target_os = "dragonfly", target_os = "redox")))]
pub const TABDLY: Tcflag = c::TABDLY;

/// `TAB0`
#[cfg(not(any(
    netbsdlike,
    solarish,
    target_os = "dragonfly",
    target_os = "fuchsia",
    target_os = "redox",
)))]
pub const TAB0: Tcflag = c::TAB0;

/// `TAB1`
#[cfg(not(any(
    target_env = "musl",
    bsd,
    solarish,
    target_os = "emscripten",
    target_os = "fuchsia",
    target_os = "redox",
)))]
pub const TAB1: Tcflag = c::TAB1;

/// `TAB2`
#[cfg(not(any(
    target_env = "musl",
    bsd,
    solarish,
    target_os = "emscripten",
    target_os = "fuchsia",
    target_os = "redox",
)))]
pub const TAB2: Tcflag = c::TAB2;

/// `TAB3`
#[cfg(not(any(
    target_env = "musl",
    bsd,
    solarish,
    target_os = "emscripten",
    target_os = "fuchsia",
    target_os = "redox",
)))]
pub const TAB3: Tcflag = c::TAB3;

/// `BSDLY`
#[cfg(not(any(bsd, solarish, target_os = "redox")))]
pub const BSDLY: Tcflag = c::BSDLY;

/// `BS0`
#[cfg(not(any(bsd, solarish, target_os = "fuchsia", target_os = "redox")))]
pub const BS0: Tcflag = c::BS0;

/// `BS1`
#[cfg(not(any(
    target_env = "musl",
    bsd,
    solarish,
    target_os = "emscripten",
    target_os = "fuchsia",
    target_os = "redox",
)))]
pub const BS1: Tcflag = c::BS1;

/// `FFDLY`
#[cfg(not(any(target_env = "musl", bsd, solarish, target_os = "redox")))]
pub const FFDLY: Tcflag = c::FFDLY;

/// `FF0`
#[cfg(not(any(bsd, solarish, target_os = "fuchsia", target_os = "redox")))]
pub const FF0: Tcflag = c::FF0;

/// `FF1`
#[cfg(not(any(
    target_env = "musl",
    bsd,
    solarish,
    target_os = "emscripten",
    target_os = "fuchsia",
    target_os = "redox",
)))]
pub const FF1: Tcflag = c::FF1;

/// `VTDLY`
#[cfg(not(any(target_env = "musl", bsd, solarish, target_os = "redox")))]
pub const VTDLY: Tcflag = c::VTDLY;

/// `VT0`
#[cfg(not(any(bsd, solarish, target_os = "fuchsia", target_os = "redox")))]
pub const VT0: Tcflag = c::VT0;

/// `VT1`
#[cfg(not(any(
    target_env = "musl",
    bsd,
    solarish,
    target_os = "emscripten",
    target_os = "fuchsia",
    target_os = "redox",
)))]
pub const VT1: Tcflag = c::VT1;

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
#[cfg(not(target_os = "aix"))]
pub const B57600: Speed = c::B57600;

/// `B115200`
#[cfg(not(target_os = "aix"))]
pub const B115200: Speed = c::B115200;

/// `B230400`
#[cfg(not(target_os = "aix"))]
pub const B230400: Speed = c::B230400;

/// `B460800`
#[cfg(not(any(
    apple,
    target_os = "aix",
    target_os = "dragonfly",
    target_os = "haiku",
    target_os = "openbsd"
)))]
pub const B460800: Speed = c::B460800;

/// `B500000`
#[cfg(not(any(bsd, solarish, target_os = "aix", target_os = "haiku")))]
pub const B500000: Speed = c::B500000;

/// `B576000`
#[cfg(not(any(bsd, solarish, target_os = "aix", target_os = "haiku")))]
pub const B576000: Speed = c::B576000;

/// `B921600`
#[cfg(not(any(
    apple,
    target_os = "aix",
    target_os = "dragonfly",
    target_os = "haiku",
    target_os = "openbsd"
)))]
pub const B921600: Speed = c::B921600;

/// `B1000000`
#[cfg(not(any(bsd, target_os = "aix", target_os = "haiku", target_os = "solaris")))]
pub const B1000000: Speed = c::B1000000;

/// `B1152000`
#[cfg(not(any(bsd, target_os = "aix", target_os = "haiku", target_os = "solaris")))]
pub const B1152000: Speed = c::B1152000;

/// `B1500000`
#[cfg(not(any(bsd, target_os = "aix", target_os = "haiku", target_os = "solaris")))]
pub const B1500000: Speed = c::B1500000;

/// `B2000000`
#[cfg(not(any(bsd, target_os = "aix", target_os = "haiku", target_os = "solaris")))]
pub const B2000000: Speed = c::B2000000;

/// `B2500000`
#[cfg(not(any(
    target_arch = "sparc",
    target_arch = "sparc64",
    bsd,
    target_os = "aix",
    target_os = "haiku",
    target_os = "solaris",
)))]
pub const B2500000: Speed = c::B2500000;

/// `B3000000`
#[cfg(not(any(
    target_arch = "sparc",
    target_arch = "sparc64",
    bsd,
    target_os = "aix",
    target_os = "haiku",
    target_os = "solaris",
)))]
pub const B3000000: Speed = c::B3000000;

/// `B3500000`
#[cfg(not(any(
    target_arch = "sparc",
    target_arch = "sparc64",
    bsd,
    target_os = "aix",
    target_os = "haiku",
    target_os = "solaris",
)))]
pub const B3500000: Speed = c::B3500000;

/// `B4000000`
#[cfg(not(any(
    target_arch = "sparc",
    target_arch = "sparc64",
    bsd,
    target_os = "aix",
    target_os = "haiku",
    target_os = "solaris",
)))]
pub const B4000000: Speed = c::B4000000;

/// `BOTHER`
#[cfg(linux_kernel)]
pub const BOTHER: Speed = c::BOTHER;

/// `CSIZE`
pub const CSIZE: Tcflag = c::CSIZE;

/// `CS5`
pub const CS5: Tcflag = c::CS5;

/// `CS6`
pub const CS6: Tcflag = c::CS6;

/// `CS7`
pub const CS7: Tcflag = c::CS7;

/// `CS8`
pub const CS8: Tcflag = c::CS8;

/// `CSTOPB`
pub const CSTOPB: Tcflag = c::CSTOPB;

/// `CREAD`
pub const CREAD: Tcflag = c::CREAD;

/// `PARENB`
pub const PARENB: Tcflag = c::PARENB;

/// `PARODD`
pub const PARODD: Tcflag = c::PARODD;

/// `HUPCL`
pub const HUPCL: Tcflag = c::HUPCL;

/// `CLOCAL`
pub const CLOCAL: Tcflag = c::CLOCAL;

/// `ISIG`
pub const ISIG: Tcflag = c::ISIG;

/// `ICANON`—A flag for the `c_lflag` field of [`Termios`] indicating
/// canonical mode.
pub const ICANON: Tcflag = c::ICANON;

/// `ECHO`
pub const ECHO: Tcflag = c::ECHO;

/// `ECHOE`
pub const ECHOE: Tcflag = c::ECHOE;

/// `ECHOK`
pub const ECHOK: Tcflag = c::ECHOK;

/// `ECHONL`
pub const ECHONL: Tcflag = c::ECHONL;

/// `NOFLSH`
pub const NOFLSH: Tcflag = c::NOFLSH;

/// `TOSTOP`
pub const TOSTOP: Tcflag = c::TOSTOP;

/// `IEXTEN`
pub const IEXTEN: Tcflag = c::IEXTEN;

/// `EXTA`
#[cfg(not(any(
    solarish,
    target_os = "emscripten",
    target_os = "haiku",
    target_os = "redox",
)))]
pub const EXTA: Speed = c::EXTA;

/// `EXTB`
#[cfg(not(any(
    solarish,
    target_os = "emscripten",
    target_os = "haiku",
    target_os = "redox",
)))]
pub const EXTB: Speed = c::EXTB;

/// `CBAUD`
#[cfg(not(any(bsd, target_os = "haiku", target_os = "redox")))]
pub const CBAUD: Tcflag = c::CBAUD;

/// `CBAUDEX`
#[cfg(not(any(
    bsd,
    solarish,
    target_os = "aix",
    target_os = "haiku",
    target_os = "redox",
)))]
pub const CBAUDEX: Tcflag = c::CBAUDEX;

/// `CIBAUD`
#[cfg(not(any(
    target_arch = "powerpc",
    target_arch = "powerpc64",
    bsd,
    target_os = "emscripten",
    target_os = "haiku",
    target_os = "redox",
)))]
pub const CIBAUD: Tcflag = c::CIBAUD;

/// `CIBAUD`
// glibc on powerpc lacks a definition for `CIBAUD`, even though the Linux
// headers and Musl on powerpc both have one. So define it manually.
#[cfg(any(target_arch = "powerpc", target_arch = "powerpc64"))]
pub const CIBAUD: Tcflag = 0o77600000;

/// `CMSPAR`
#[cfg(not(any(
    bsd,
    solarish,
    target_os = "aix",
    target_os = "emscripten",
    target_os = "haiku",
    target_os = "redox",
)))]
pub const CMSPAR: Tcflag = c::CMSPAR;

/// `CRTSCTS`
#[cfg(not(any(target_os = "aix", target_os = "redox")))]
pub const CRTSCTS: Tcflag = c::CRTSCTS;

/// `XCASE`
#[cfg(any(target_arch = "s390x", target_os = "haiku"))]
pub const XCASE: Tcflag = c::XCASE;

/// `ECHOCTL`
#[cfg(not(any(target_os = "redox")))]
pub const ECHOCTL: Tcflag = c::ECHOCTL;

/// `ECHOPRT`
#[cfg(not(any(target_os = "redox")))]
pub const ECHOPRT: Tcflag = c::ECHOPRT;

/// `ECHOKE`
#[cfg(not(any(target_os = "redox")))]
pub const ECHOKE: Tcflag = c::ECHOKE;

/// `FLUSHO`
#[cfg(not(any(target_os = "redox")))]
pub const FLUSHO: Tcflag = c::FLUSHO;

/// `PENDIN`
#[cfg(not(any(target_os = "redox")))]
pub const PENDIN: Tcflag = c::PENDIN;

/// `EXTPROC`
#[cfg(not(any(target_os = "aix", target_os = "haiku", target_os = "redox")))]
pub const EXTPROC: Tcflag = c::EXTPROC;

/// `XTABS`
#[cfg(not(any(
    bsd,
    solarish,
    target_os = "aix",
    target_os = "haiku",
    target_os = "redox",
)))]
pub const XTABS: Tcflag = c::XTABS;
