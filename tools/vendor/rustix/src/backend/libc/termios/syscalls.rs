//! libc syscalls supporting `rustix::termios`.
//!
//! # Safety
//!
//! See the `rustix::backend::syscalls` module documentation for details.

use crate::backend::c;
use crate::backend::conv::{borrowed_fd, ret, ret_pid_t};
use crate::fd::BorrowedFd;
#[cfg(feature = "procfs")]
#[cfg(not(any(target_os = "fuchsia", target_os = "wasi")))]
use crate::ffi::CStr;
use core::mem::MaybeUninit;
#[cfg(not(target_os = "wasi"))]
use {
    crate::io,
    crate::pid::Pid,
    crate::termios::{Action, OptionalActions, QueueSelector, Termios, Winsize},
    crate::utils::as_mut_ptr,
};

#[cfg(not(target_os = "wasi"))]
pub(crate) fn tcgetattr(fd: BorrowedFd<'_>) -> io::Result<Termios> {
    // If we have `TCGETS2`, use it, so that we fill in the `c_ispeed` and
    // `c_ospeed` fields.
    #[cfg(linux_kernel)]
    unsafe {
        use crate::termios::{ControlModes, InputModes, LocalModes, OutputModes, SpecialCodes};
        use core::mem::zeroed;

        let mut termios2 = MaybeUninit::<c::termios2>::uninit();

        ret(c::ioctl(
            borrowed_fd(fd),
            c::TCGETS2.into(),
            termios2.as_mut_ptr(),
        ))?;

        let termios2 = termios2.assume_init();

        // Convert from the Linux `termios2` to our `Termios`.
        let mut result = Termios {
            input_modes: InputModes::from_bits_retain(termios2.c_iflag),
            output_modes: OutputModes::from_bits_retain(termios2.c_oflag),
            control_modes: ControlModes::from_bits_retain(termios2.c_cflag),
            local_modes: LocalModes::from_bits_retain(termios2.c_lflag),
            line_discipline: termios2.c_line,
            special_codes: SpecialCodes(zeroed()),
            input_speed: termios2.c_ispeed,
            output_speed: termios2.c_ospeed,
        };
        result.special_codes.0[..termios2.c_cc.len()].copy_from_slice(&termios2.c_cc);

        Ok(result)
    }

    #[cfg(not(linux_kernel))]
    unsafe {
        let mut result = MaybeUninit::<Termios>::uninit();

        ret(c::tcgetattr(borrowed_fd(fd), result.as_mut_ptr().cast()))?;

        Ok(result.assume_init())
    }
}

#[cfg(not(target_os = "wasi"))]
pub(crate) fn tcgetpgrp(fd: BorrowedFd<'_>) -> io::Result<Pid> {
    unsafe {
        let pid = ret_pid_t(c::tcgetpgrp(borrowed_fd(fd)))?;
        Ok(Pid::from_raw_unchecked(pid))
    }
}

#[cfg(not(target_os = "wasi"))]
pub(crate) fn tcsetpgrp(fd: BorrowedFd<'_>, pid: Pid) -> io::Result<()> {
    unsafe { ret(c::tcsetpgrp(borrowed_fd(fd), pid.as_raw_nonzero().get())) }
}

#[cfg(not(target_os = "wasi"))]
pub(crate) fn tcsetattr(
    fd: BorrowedFd,
    optional_actions: OptionalActions,
    termios: &Termios,
) -> io::Result<()> {
    // If we have `TCSETS2`, use it, so that we use the `c_ispeed` and
    // `c_ospeed` fields.
    #[cfg(linux_kernel)]
    unsafe {
        use crate::termios::speed;
        use core::mem::zeroed;
        use linux_raw_sys::general::{termios2, BOTHER, CBAUD, IBSHIFT};

        #[cfg(not(any(target_arch = "sparc", target_arch = "sparc64")))]
        use linux_raw_sys::ioctl::{TCSETS, TCSETS2};

        // linux-raw-sys' ioctl-generation script for sparc isn't working yet,
        // so as a temporary workaround, declare these manually.
        #[cfg(any(target_arch = "sparc", target_arch = "sparc64"))]
        const TCSETS: u32 = 0x80245409;
        #[cfg(any(target_arch = "sparc", target_arch = "sparc64"))]
        const TCSETS2: u32 = 0x802c540d;

        // Translate from `optional_actions` into an ioctl request code. On MIPS,
        // `optional_actions` already has `TCGETS` added to it.
        let request = TCSETS2
            + if cfg!(any(target_arch = "mips", target_arch = "mips64")) {
                optional_actions as u32 - TCSETS
            } else {
                optional_actions as u32
            };

        let input_speed = termios.input_speed();
        let output_speed = termios.output_speed();
        let mut termios2 = termios2 {
            c_iflag: termios.input_modes.bits(),
            c_oflag: termios.output_modes.bits(),
            c_cflag: termios.control_modes.bits(),
            c_lflag: termios.local_modes.bits(),
            c_line: termios.line_discipline,
            c_cc: zeroed(),
            c_ispeed: input_speed,
            c_ospeed: output_speed,
        };
        // Ensure that our input and output speeds are set, as `libc`
        // routines don't always support setting these separately.
        termios2.c_cflag &= !CBAUD;
        termios2.c_cflag |= speed::encode(output_speed).unwrap_or(BOTHER);
        termios2.c_cflag &= !(CBAUD << IBSHIFT);
        termios2.c_cflag |= speed::encode(input_speed).unwrap_or(BOTHER) << IBSHIFT;
        let nccs = termios2.c_cc.len();
        termios2
            .c_cc
            .copy_from_slice(&termios.special_codes.0[..nccs]);

        ret(c::ioctl(borrowed_fd(fd), request as _, &termios2))
    }

    #[cfg(not(linux_kernel))]
    unsafe {
        ret(c::tcsetattr(
            borrowed_fd(fd),
            optional_actions as _,
            crate::utils::as_ptr(termios).cast(),
        ))
    }
}

#[cfg(not(target_os = "wasi"))]
pub(crate) fn tcsendbreak(fd: BorrowedFd) -> io::Result<()> {
    unsafe { ret(c::tcsendbreak(borrowed_fd(fd), 0)) }
}

#[cfg(not(target_os = "wasi"))]
pub(crate) fn tcdrain(fd: BorrowedFd) -> io::Result<()> {
    unsafe { ret(c::tcdrain(borrowed_fd(fd))) }
}

#[cfg(not(target_os = "wasi"))]
pub(crate) fn tcflush(fd: BorrowedFd, queue_selector: QueueSelector) -> io::Result<()> {
    unsafe { ret(c::tcflush(borrowed_fd(fd), queue_selector as _)) }
}

#[cfg(not(target_os = "wasi"))]
pub(crate) fn tcflow(fd: BorrowedFd, action: Action) -> io::Result<()> {
    unsafe { ret(c::tcflow(borrowed_fd(fd), action as _)) }
}

#[cfg(not(target_os = "wasi"))]
pub(crate) fn tcgetsid(fd: BorrowedFd) -> io::Result<Pid> {
    unsafe {
        let pid = ret_pid_t(c::tcgetsid(borrowed_fd(fd)))?;
        Ok(Pid::from_raw_unchecked(pid))
    }
}

#[cfg(not(target_os = "wasi"))]
pub(crate) fn tcsetwinsize(fd: BorrowedFd, winsize: Winsize) -> io::Result<()> {
    unsafe { ret(c::ioctl(borrowed_fd(fd), c::TIOCSWINSZ, &winsize)) }
}

#[cfg(not(target_os = "wasi"))]
pub(crate) fn tcgetwinsize(fd: BorrowedFd) -> io::Result<Winsize> {
    unsafe {
        let mut buf = MaybeUninit::<Winsize>::uninit();
        ret(c::ioctl(
            borrowed_fd(fd),
            c::TIOCGWINSZ.into(),
            buf.as_mut_ptr(),
        ))?;
        Ok(buf.assume_init())
    }
}

#[cfg(not(any(target_os = "haiku", target_os = "redox", target_os = "wasi")))]
pub(crate) fn ioctl_tiocexcl(fd: BorrowedFd) -> io::Result<()> {
    unsafe { ret(c::ioctl(borrowed_fd(fd), c::TIOCEXCL as _)) }
}

#[cfg(not(any(target_os = "haiku", target_os = "redox", target_os = "wasi")))]
pub(crate) fn ioctl_tiocnxcl(fd: BorrowedFd) -> io::Result<()> {
    unsafe { ret(c::ioctl(borrowed_fd(fd), c::TIOCNXCL as _)) }
}

#[cfg(not(target_os = "wasi"))]
#[inline]
pub(crate) fn set_speed(termios: &mut Termios, arbitrary_speed: u32) -> io::Result<()> {
    #[cfg(bsd)]
    let encoded_speed = arbitrary_speed;

    #[cfg(not(bsd))]
    let encoded_speed = match crate::termios::speed::encode(arbitrary_speed) {
        Some(encoded_speed) => encoded_speed,
        #[cfg(linux_kernel)]
        None => c::BOTHER,
        #[cfg(not(linux_kernel))]
        None => return Err(io::Errno::INVAL),
    };

    #[cfg(not(linux_kernel))]
    unsafe {
        ret(c::cfsetspeed(
            as_mut_ptr(termios).cast(),
            encoded_speed.into(),
        ))
    }

    // Linux libc implementations don't support arbitrary speeds, so we encode
    // the speed manually.
    #[cfg(linux_kernel)]
    {
        use crate::termios::ControlModes;

        debug_assert_eq!(encoded_speed & !c::CBAUD, 0);

        termios.control_modes -= ControlModes::from_bits_retain(c::CBAUD | c::CIBAUD);
        termios.control_modes |=
            ControlModes::from_bits_retain(encoded_speed | (encoded_speed << c::IBSHIFT));

        termios.input_speed = arbitrary_speed;
        termios.output_speed = arbitrary_speed;

        Ok(())
    }
}

#[cfg(not(target_os = "wasi"))]
#[inline]
pub(crate) fn set_output_speed(termios: &mut Termios, arbitrary_speed: u32) -> io::Result<()> {
    #[cfg(bsd)]
    let encoded_speed = arbitrary_speed;

    #[cfg(not(bsd))]
    let encoded_speed = match crate::termios::speed::encode(arbitrary_speed) {
        Some(encoded_speed) => encoded_speed,
        #[cfg(linux_kernel)]
        None => c::BOTHER,
        #[cfg(not(linux_kernel))]
        None => return Err(io::Errno::INVAL),
    };

    #[cfg(not(linux_kernel))]
    unsafe {
        ret(c::cfsetospeed(
            as_mut_ptr(termios).cast(),
            encoded_speed.into(),
        ))
    }

    // Linux libc implementations don't support arbitrary speeds or setting the
    // input and output speeds separately, so we encode the speed manually.
    #[cfg(linux_kernel)]
    {
        use crate::termios::ControlModes;

        debug_assert_eq!(encoded_speed & !c::CBAUD, 0);

        termios.control_modes -= ControlModes::from_bits_retain(c::CBAUD);
        termios.control_modes |= ControlModes::from_bits_retain(encoded_speed);

        termios.output_speed = arbitrary_speed;

        Ok(())
    }
}

#[cfg(not(target_os = "wasi"))]
#[inline]
pub(crate) fn set_input_speed(termios: &mut Termios, arbitrary_speed: u32) -> io::Result<()> {
    #[cfg(bsd)]
    let encoded_speed = arbitrary_speed;

    #[cfg(not(bsd))]
    let encoded_speed = match crate::termios::speed::encode(arbitrary_speed) {
        Some(encoded_speed) => encoded_speed,
        #[cfg(linux_kernel)]
        None => c::BOTHER,
        #[cfg(not(linux_kernel))]
        None => return Err(io::Errno::INVAL),
    };

    #[cfg(not(linux_kernel))]
    unsafe {
        ret(c::cfsetispeed(
            as_mut_ptr(termios).cast(),
            encoded_speed.into(),
        ))
    }

    // Linux libc implementations don't support arbitrary speeds or setting the
    // input and output speeds separately, so we encode the speed manually.
    #[cfg(linux_kernel)]
    {
        use crate::termios::ControlModes;

        debug_assert_eq!(encoded_speed & !c::CBAUD, 0);

        termios.control_modes -= ControlModes::from_bits_retain(c::CIBAUD);
        termios.control_modes |= ControlModes::from_bits_retain(encoded_speed << c::IBSHIFT);

        termios.input_speed = arbitrary_speed;

        Ok(())
    }
}

#[cfg(not(target_os = "wasi"))]
#[inline]
pub(crate) fn cfmakeraw(termios: &mut Termios) {
    unsafe { c::cfmakeraw(as_mut_ptr(termios).cast()) }
}

pub(crate) fn isatty(fd: BorrowedFd<'_>) -> bool {
    // Use the return value of `isatty` alone. We don't check `errno` because
    // we return `bool` rather than `io::Result<bool>`, because we assume
    // `BorrrowedFd` protects us from `EBADF`, and any other reasonably
    // anticipated `errno` value would end up interpreted as “assume it's not a
    // terminal” anyway.
    unsafe { c::isatty(borrowed_fd(fd)) != 0 }
}

#[cfg(feature = "procfs")]
#[cfg(not(any(target_os = "fuchsia", target_os = "wasi")))]
pub(crate) fn ttyname(dirfd: BorrowedFd<'_>, buf: &mut [MaybeUninit<u8>]) -> io::Result<usize> {
    unsafe {
        // `ttyname_r` returns its error status rather than using `errno`.
        match c::ttyname_r(borrowed_fd(dirfd), buf.as_mut_ptr().cast(), buf.len()) {
            0 => Ok(CStr::from_ptr(buf.as_ptr().cast()).to_bytes().len()),
            err => Err(io::Errno::from_raw_os_error(err)),
        }
    }
}
