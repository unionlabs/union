//! libc syscalls supporting `rustix::process`.

use super::types::RawUname;
use crate::backend::c;
#[cfg(not(target_os = "wasi"))]
use crate::backend::conv::ret_infallible;
#[cfg(linux_kernel)]
use crate::system::Sysinfo;
use core::mem::MaybeUninit;
#[cfg(not(any(target_os = "emscripten", target_os = "redox", target_os = "wasi")))]
use {crate::backend::conv::ret, crate::io};

#[cfg(not(target_os = "wasi"))]
#[inline]
pub(crate) fn uname() -> RawUname {
    let mut uname = MaybeUninit::<RawUname>::uninit();
    unsafe {
        ret_infallible(c::uname(uname.as_mut_ptr()));
        uname.assume_init()
    }
}

#[cfg(linux_kernel)]
pub(crate) fn sysinfo() -> Sysinfo {
    let mut info = MaybeUninit::<Sysinfo>::uninit();
    unsafe {
        ret_infallible(c::sysinfo(info.as_mut_ptr()));
        info.assume_init()
    }
}

#[cfg(not(any(target_os = "emscripten", target_os = "redox", target_os = "wasi")))]
pub(crate) fn sethostname(name: &[u8]) -> io::Result<()> {
    unsafe {
        ret(c::sethostname(
            name.as_ptr().cast(),
            name.len().try_into().map_err(|_| io::Errno::INVAL)?,
        ))
    }
}
