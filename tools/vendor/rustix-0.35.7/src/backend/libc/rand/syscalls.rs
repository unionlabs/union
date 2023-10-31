//! libc syscalls supporting `rustix::rand`.

#[cfg(target_os = "linux")]
use {super::super::c, super::super::conv::ret_ssize_t, crate::io, crate::rand::GetRandomFlags};

#[cfg(target_os = "linux")]
pub(crate) fn getrandom(buf: &mut [u8], flags: GetRandomFlags) -> io::Result<usize> {
    // `getrandom` wasn't supported in glibc until 2.25.
    weak_or_syscall! {
        fn getrandom(buf: *mut c::c_void, buflen: c::size_t, flags: c::c_uint) via SYS_getrandom -> c::ssize_t
    }

    let nread =
        unsafe { ret_ssize_t(getrandom(buf.as_mut_ptr().cast(), buf.len(), flags.bits()))? };
    Ok(nread as usize)
}
