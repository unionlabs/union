// The functions replacing the C macros use the same names as in libc.
#![allow(non_snake_case)]

pub(crate) use linux_raw_sys::general::{WCONTINUED, WNOHANG, WUNTRACED};

#[inline]
pub(crate) fn WIFSTOPPED(status: u32) -> bool {
    (status & 0xff) == 0x7f
}

#[inline]
pub(crate) fn WSTOPSIG(status: u32) -> u32 {
    (status >> 8) & 0xff
}

#[inline]
pub(crate) fn WIFCONTINUED(status: u32) -> bool {
    status == 0xffff
}

#[inline]
pub(crate) fn WIFSIGNALED(status: u32) -> bool {
    ((status & 0x7f) + 1) as i8 >= 2
}

#[inline]
pub(crate) fn WTERMSIG(status: u32) -> u32 {
    status & 0x7f
}

#[inline]
pub(crate) fn WIFEXITED(status: u32) -> bool {
    (status & 0x7f) == 0
}

#[inline]
pub(crate) fn WEXITSTATUS(status: u32) -> u32 {
    (status >> 8) & 0xff
}
