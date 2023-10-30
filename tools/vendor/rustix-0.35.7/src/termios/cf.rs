use crate::termios::{Speed, Termios};
use crate::{backend, io};

/// `cfgetospeed(termios)`
#[inline]
#[must_use]
pub fn cfgetospeed(termios: &Termios) -> Speed {
    backend::termios::syscalls::cfgetospeed(termios)
}

/// `cfgetispeed(termios)`
#[inline]
#[must_use]
pub fn cfgetispeed(termios: &Termios) -> Speed {
    backend::termios::syscalls::cfgetispeed(termios)
}

/// `cfmakeraw(termios)`
#[inline]
pub fn cfmakeraw(termios: &mut Termios) {
    backend::termios::syscalls::cfmakeraw(termios)
}

/// `cfsetospeed(termios, speed)`
#[inline]
pub fn cfsetospeed(termios: &mut Termios, speed: Speed) -> io::Result<()> {
    backend::termios::syscalls::cfsetospeed(termios, speed)
}

/// `cfsetispeed(termios, speed)`
#[inline]
pub fn cfsetispeed(termios: &mut Termios, speed: Speed) -> io::Result<()> {
    backend::termios::syscalls::cfsetispeed(termios, speed)
}

/// `cfsetspeed(termios, speed)`
#[inline]
pub fn cfsetspeed(termios: &mut Termios, speed: Speed) -> io::Result<()> {
    backend::termios::syscalls::cfsetspeed(termios, speed)
}
