use super::super::c;

/// `TCSA*` values for use with [`tcsetattr`].
///
/// [`tcsetattr`]: crate::termios::tcsetattr
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[repr(u32)]
pub enum OptionalActions {
    /// `TCSANOW`—Make the change immediately.
    #[doc(alias = "TCSANOW")]
    Now = linux_raw_sys::general::TCSANOW,

    /// `TCSADRAIN`—Make the change after all output has been transmitted.
    #[doc(alias = "TCSADRAIN")]
    Drain = linux_raw_sys::general::TCSADRAIN,

    /// `TCSAFLUSH`—Discard any pending input and then make the change
    /// after all output has been transmitted.
    #[doc(alias = "TCSAFLUSH")]
    Flush = linux_raw_sys::general::TCSAFLUSH,
}

/// `TC*` values for use with [`tcflush`].
///
/// [`tcflush`]: crate::termios::tcflush
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[repr(u32)]
pub enum QueueSelector {
    /// `TCIFLUSH`—Flush data received but not read.
    #[doc(alias = "TCIFLUSH")]
    IFlush = linux_raw_sys::general::TCIFLUSH,

    /// `TCOFLUSH`—Flush data written but not transmitted.
    #[doc(alias = "TCOFLUSH")]
    OFlush = linux_raw_sys::general::TCOFLUSH,

    /// `TCIOFLUSH`—`IFlush` and `OFlush` combined.
    #[doc(alias = "TCIOFLUSH")]
    IOFlush = linux_raw_sys::general::TCIOFLUSH,
}

/// `TC*` values for use with [`tcflow`].
///
/// [`tcflow`]: crate::termios::tcflow
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[repr(u32)]
pub enum Action {
    /// `TCOOFF`—Suspend output.
    #[doc(alias = "TCOOFF")]
    OOff = linux_raw_sys::general::TCOOFF,

    /// `TCOON`—Restart suspended output.
    #[doc(alias = "TCOON")]
    OOn = linux_raw_sys::general::TCOON,

    /// `TCIOFF`—Transmits a STOP byte.
    #[doc(alias = "TCIOFF")]
    IOff = linux_raw_sys::general::TCIOFF,

    /// `TCION`—Transmits a START byte.
    #[doc(alias = "TCION")]
    IOn = linux_raw_sys::general::TCION,
}

/// `struct termios` for use with [`tcgetattr`] and [`tcsetattr`].
///
/// [`tcgetattr`]: crate::termios::tcgetattr
/// [`tcsetattr`]: crate::termios::tcsetattr
#[doc(alias = "termios")]
pub type Termios = linux_raw_sys::general::termios;

/// `struct termios2` for use with [`tcgetattr2`] and [`tcsetattr2`].
///
/// [`tcgetattr2`]: crate::termios::tcgetattr2
/// [`tcsetattr2`]: crate::termios::tcsetattr2
#[cfg(any(
    target_arch = "x86",
    target_arch = "x86_64",
    target_arch = "x32",
    target_arch = "riscv64",
    target_arch = "aarch64",
    target_arch = "arm",
    target_arch = "mips",
    target_arch = "mips64",
))]
#[doc(alias = "termios2")]
pub type Termios2 = linux_raw_sys::general::termios2;

/// `struct winsize` for use with [`tcgetwinsize`].
///
/// [`tcgetwinsize`]: crate::termios::tcgetwinsize
#[doc(alias = "winsize")]
pub type Winsize = linux_raw_sys::general::winsize;

/// `tcflag_t`—A type for the flags fields of [`Termios`].
#[doc(alias = "tcflag_t")]
pub type Tcflag = linux_raw_sys::general::tcflag_t;

/// `speed_t`—A return type for [`cfsetspeed`] and similar.
///
/// [`cfsetspeed`]: crate::termios::cfsetspeed
#[doc(alias = "speed_t")]
pub type Speed = linux_raw_sys::general::speed_t;

/// `VINTR`
pub const VINTR: usize = linux_raw_sys::general::VINTR as usize;

/// `VQUIT`
pub const VQUIT: usize = linux_raw_sys::general::VQUIT as usize;

/// `VERASE`
pub const VERASE: usize = linux_raw_sys::general::VERASE as usize;

/// `VKILL`
pub const VKILL: usize = linux_raw_sys::general::VKILL as usize;

/// `VEOF`
pub const VEOF: usize = linux_raw_sys::general::VEOF as usize;

/// `VTIME`
pub const VTIME: usize = linux_raw_sys::general::VTIME as usize;

/// `VMIN`
pub const VMIN: usize = linux_raw_sys::general::VMIN as usize;

/// `VSWTC`
pub const VSWTC: usize = linux_raw_sys::general::VSWTC as usize;

/// `VSTART`
pub const VSTART: usize = linux_raw_sys::general::VSTART as usize;

/// `VSTOP`
pub const VSTOP: usize = linux_raw_sys::general::VSTOP as usize;

/// `VSUSP`
pub const VSUSP: usize = linux_raw_sys::general::VSUSP as usize;

/// `VEOL`
pub const VEOL: usize = linux_raw_sys::general::VEOL as usize;

/// `VREPRINT`
pub const VREPRINT: usize = linux_raw_sys::general::VREPRINT as usize;

/// `VDISCARD`
pub const VDISCARD: usize = linux_raw_sys::general::VDISCARD as usize;

/// `VWERASE`
pub const VWERASE: usize = linux_raw_sys::general::VWERASE as usize;

/// `VLNEXT`
pub const VLNEXT: usize = linux_raw_sys::general::VLNEXT as usize;

/// `VEOL2`
pub const VEOL2: usize = linux_raw_sys::general::VEOL2 as usize;

/// `IGNBRK`
pub const IGNBRK: c::c_uint = linux_raw_sys::general::IGNBRK;

/// `BRKINT`
pub const BRKINT: c::c_uint = linux_raw_sys::general::BRKINT;

/// `IGNPAR`
pub const IGNPAR: c::c_uint = linux_raw_sys::general::IGNPAR;

/// `PARMRK`
pub const PARMRK: c::c_uint = linux_raw_sys::general::PARMRK;

/// `INPCK`
pub const INPCK: c::c_uint = linux_raw_sys::general::INPCK;

/// `ISTRIP`
pub const ISTRIP: c::c_uint = linux_raw_sys::general::ISTRIP;

/// `INLCR`
pub const INLCR: c::c_uint = linux_raw_sys::general::INLCR;

/// `IGNCR`
pub const IGNCR: c::c_uint = linux_raw_sys::general::IGNCR;

/// `ICRNL`
pub const ICRNL: c::c_uint = linux_raw_sys::general::ICRNL;

/// `IUCLC`
pub const IUCLC: c::c_uint = linux_raw_sys::general::IUCLC;

/// `IXON`
pub const IXON: c::c_uint = linux_raw_sys::general::IXON;

/// `IXANY`
pub const IXANY: c::c_uint = linux_raw_sys::general::IXANY;

/// `IXOFF`
pub const IXOFF: c::c_uint = linux_raw_sys::general::IXOFF;

/// `IMAXBEL`
pub const IMAXBEL: c::c_uint = linux_raw_sys::general::IMAXBEL;

/// `IUTF8`
pub const IUTF8: c::c_uint = linux_raw_sys::general::IUTF8;

/// `OPOST`
pub const OPOST: c::c_uint = linux_raw_sys::general::OPOST;

/// `OLCUC`
pub const OLCUC: c::c_uint = linux_raw_sys::general::OLCUC;

/// `ONLCR`
pub const ONLCR: c::c_uint = linux_raw_sys::general::ONLCR;

/// `OCRNL`
pub const OCRNL: c::c_uint = linux_raw_sys::general::OCRNL;

/// `ONOCR`
pub const ONOCR: c::c_uint = linux_raw_sys::general::ONOCR;

/// `ONLRET`
pub const ONLRET: c::c_uint = linux_raw_sys::general::ONLRET;

/// `OFILL`
pub const OFILL: c::c_uint = linux_raw_sys::general::OFILL;

/// `OFDEL`
pub const OFDEL: c::c_uint = linux_raw_sys::general::OFDEL;

/// `NLDLY`
pub const NLDLY: c::c_uint = linux_raw_sys::general::NLDLY;

/// `NL0`
pub const NL0: c::c_uint = linux_raw_sys::general::NL0;

/// `NL1`
pub const NL1: c::c_uint = linux_raw_sys::general::NL1;

/// `CRDLY`
pub const CRDLY: c::c_uint = linux_raw_sys::general::CRDLY;

/// `CR0`
pub const CR0: c::c_uint = linux_raw_sys::general::CR0;

/// `CR1`
pub const CR1: c::c_uint = linux_raw_sys::general::CR1;

/// `CR2`
pub const CR2: c::c_uint = linux_raw_sys::general::CR2;

/// `CR3`
pub const CR3: c::c_uint = linux_raw_sys::general::CR3;

/// `TABDLY`
pub const TABDLY: c::c_uint = linux_raw_sys::general::TABDLY;

/// `TAB0`
pub const TAB0: c::c_uint = linux_raw_sys::general::TAB0;

/// `TAB1`
pub const TAB1: c::c_uint = linux_raw_sys::general::TAB1;

/// `TAB2`
pub const TAB2: c::c_uint = linux_raw_sys::general::TAB2;

/// `TAB3`
pub const TAB3: c::c_uint = linux_raw_sys::general::TAB3;

/// `BSDLY`
pub const BSDLY: c::c_uint = linux_raw_sys::general::BSDLY;

/// `BS0`
pub const BS0: c::c_uint = linux_raw_sys::general::BS0;

/// `BS1`
pub const BS1: c::c_uint = linux_raw_sys::general::BS1;

/// `FFDLY`
pub const FFDLY: c::c_uint = linux_raw_sys::general::FFDLY;

/// `FF0`
pub const FF0: c::c_uint = linux_raw_sys::general::FF0;

/// `FF1`
pub const FF1: c::c_uint = linux_raw_sys::general::FF1;

/// `VTDLY`
pub const VTDLY: c::c_uint = linux_raw_sys::general::VTDLY;

/// `VT0`
pub const VT0: c::c_uint = linux_raw_sys::general::VT0;

/// `VT1`
pub const VT1: c::c_uint = linux_raw_sys::general::VT1;

/// `B0`
pub const B0: Speed = linux_raw_sys::general::B0;

/// `B50`
pub const B50: Speed = linux_raw_sys::general::B50;

/// `B75`
pub const B75: Speed = linux_raw_sys::general::B75;

/// `B110`
pub const B110: Speed = linux_raw_sys::general::B110;

/// `B134`
pub const B134: Speed = linux_raw_sys::general::B134;

/// `B150`
pub const B150: Speed = linux_raw_sys::general::B150;

/// `B200`
pub const B200: Speed = linux_raw_sys::general::B200;

/// `B300`
pub const B300: Speed = linux_raw_sys::general::B300;

/// `B600`
pub const B600: Speed = linux_raw_sys::general::B600;

/// `B1200`
pub const B1200: Speed = linux_raw_sys::general::B1200;

/// `B1800`
pub const B1800: Speed = linux_raw_sys::general::B1800;

/// `B2400`
pub const B2400: Speed = linux_raw_sys::general::B2400;

/// `B4800`
pub const B4800: Speed = linux_raw_sys::general::B4800;

/// `B9600`
pub const B9600: Speed = linux_raw_sys::general::B9600;

/// `B19200`
pub const B19200: Speed = linux_raw_sys::general::B19200;

/// `B38400`
pub const B38400: Speed = linux_raw_sys::general::B38400;

/// `B57600`
pub const B57600: Speed = linux_raw_sys::general::B57600;

/// `B115200`
pub const B115200: Speed = linux_raw_sys::general::B115200;

/// `B230400`
pub const B230400: Speed = linux_raw_sys::general::B230400;

/// `B460800`
pub const B460800: Speed = linux_raw_sys::general::B460800;

/// `B500000`
pub const B500000: Speed = linux_raw_sys::general::B500000;

/// `B576000`
pub const B576000: Speed = linux_raw_sys::general::B576000;

/// `B921600`
pub const B921600: Speed = linux_raw_sys::general::B921600;

/// `B1000000`
pub const B1000000: Speed = linux_raw_sys::general::B1000000;

/// `B1152000`
pub const B1152000: Speed = linux_raw_sys::general::B1152000;

/// `B1500000`
pub const B1500000: Speed = linux_raw_sys::general::B1500000;

/// `B2000000`
pub const B2000000: Speed = linux_raw_sys::general::B2000000;

/// `B2500000`
#[cfg(not(any(target_arch = "sparc", target_arch = "sparc64")))]
pub const B2500000: Speed = linux_raw_sys::general::B2500000;

/// `B3000000`
#[cfg(not(any(target_arch = "sparc", target_arch = "sparc64")))]
pub const B3000000: Speed = linux_raw_sys::general::B3000000;

/// `B3500000`
#[cfg(not(any(target_arch = "sparc", target_arch = "sparc64")))]
pub const B3500000: Speed = linux_raw_sys::general::B3500000;

/// `B4000000`
#[cfg(not(any(target_arch = "sparc", target_arch = "sparc64")))]
pub const B4000000: Speed = linux_raw_sys::general::B4000000;

/// `BOTHER`
pub const BOTHER: c::c_uint = linux_raw_sys::general::BOTHER;

/// `CSIZE`
pub const CSIZE: c::c_uint = linux_raw_sys::general::CSIZE;

/// `CS5`
pub const CS5: c::c_uint = linux_raw_sys::general::CS5;

/// `CS6`
pub const CS6: c::c_uint = linux_raw_sys::general::CS6;

/// `CS7`
pub const CS7: c::c_uint = linux_raw_sys::general::CS7;

/// `CS8`
pub const CS8: c::c_uint = linux_raw_sys::general::CS8;

/// `CSTOPB`
pub const CSTOPB: c::c_uint = linux_raw_sys::general::CSTOPB;

/// `CREAD`
pub const CREAD: c::c_uint = linux_raw_sys::general::CREAD;

/// `PARENB`
pub const PARENB: c::c_uint = linux_raw_sys::general::PARENB;

/// `PARODD`
pub const PARODD: c::c_uint = linux_raw_sys::general::PARODD;

/// `HUPCL`
pub const HUPCL: c::c_uint = linux_raw_sys::general::HUPCL;

/// `CLOCAL`
pub const CLOCAL: c::c_uint = linux_raw_sys::general::CLOCAL;

/// `ISIG`
pub const ISIG: c::c_uint = linux_raw_sys::general::ISIG;

/// `ICANON`—A flag for the `c_lflag` field of [`Termios`] indicating
/// canonical mode.
pub const ICANON: Tcflag = linux_raw_sys::general::ICANON;

/// `ECHO`
pub const ECHO: c::c_uint = linux_raw_sys::general::ECHO;

/// `ECHOE`
pub const ECHOE: c::c_uint = linux_raw_sys::general::ECHOE;

/// `ECHOK`
pub const ECHOK: c::c_uint = linux_raw_sys::general::ECHOK;

/// `ECHONL`
pub const ECHONL: c::c_uint = linux_raw_sys::general::ECHONL;

/// `NOFLSH`
pub const NOFLSH: c::c_uint = linux_raw_sys::general::NOFLSH;

/// `TOSTOP`
pub const TOSTOP: c::c_uint = linux_raw_sys::general::TOSTOP;

/// `IEXTEN`
pub const IEXTEN: c::c_uint = linux_raw_sys::general::IEXTEN;

/// `EXTA`
pub const EXTA: c::c_uint = linux_raw_sys::general::EXTA;

/// `EXTB`
pub const EXTB: c::c_uint = linux_raw_sys::general::EXTB;

/// `CBAUD`
pub const CBAUD: c::c_uint = linux_raw_sys::general::CBAUD;

/// `CBAUDEX`
pub const CBAUDEX: c::c_uint = linux_raw_sys::general::CBAUDEX;

/// `CIBAUD`
pub const CIBAUD: c::c_uint = linux_raw_sys::general::CIBAUD;

/// `CMSPAR`
pub const CMSPAR: c::c_uint = linux_raw_sys::general::CMSPAR;

/// `CRTSCTS`
pub const CRTSCTS: c::c_uint = linux_raw_sys::general::CRTSCTS;

/// `XCASE`
pub const XCASE: c::c_uint = linux_raw_sys::general::XCASE;

/// `ECHOCTL`
pub const ECHOCTL: c::c_uint = linux_raw_sys::general::ECHOCTL;

/// `ECHOPRT`
pub const ECHOPRT: c::c_uint = linux_raw_sys::general::ECHOPRT;

/// `ECHOKE`
pub const ECHOKE: c::c_uint = linux_raw_sys::general::ECHOKE;

/// `FLUSHO`
pub const FLUSHO: c::c_uint = linux_raw_sys::general::FLUSHO;

/// `PENDIN`
pub const PENDIN: c::c_uint = linux_raw_sys::general::PENDIN;

/// `EXTPROC`
pub const EXTPROC: c::c_uint = linux_raw_sys::general::EXTPROC;

/// `XTABS`
pub const XTABS: c::c_uint = linux_raw_sys::general::XTABS;
