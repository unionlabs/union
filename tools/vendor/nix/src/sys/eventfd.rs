use std::os::unix::io::RawFd;
use crate::Result;
use crate::errno::Errno;

libc_bitflags! {
    pub struct EfdFlags: libc::c_int {
        EFD_CLOEXEC; // Since Linux 2.6.27
        EFD_NONBLOCK; // Since Linux 2.6.27
        EFD_SEMAPHORE; // Since Linux 2.6.30
    }
}

pub fn eventfd(initval: libc::c_uint, flags: EfdFlags) -> Result<RawFd> {
    let res = unsafe { libc::eventfd(initval, flags.bits()) };

    Errno::result(res).map(|r| r as RawFd)
}
