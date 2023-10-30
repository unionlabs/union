/// Posix error category, suitable for all environments.
///
/// In presence of OS, it means it identifies POSIX error codes.
pub struct PosixCategory;

#[cfg(target_os = "unknown")]
#[inline(always)]
pub const fn is_would_block(_: i32) -> bool {
    false
}

#[cfg(not(target_os = "unknown"))]
#[inline]
pub const fn is_would_block(code: i32) -> bool {
    code == libc::EWOULDBLOCK || code == libc::EAGAIN
}

pub const fn get_unimplemented_error() -> i32 {
    #[cfg(any(windows, unix, target_os = "wasi"))]
    {
        libc::ENOSYS
    }
    #[cfg(not(any(windows, unix, target_os = "wasi")))]
    {
        0
    }

}

pub fn get_last_error() -> i32 {
    #[cfg(not(any(target_os = "wasi", target_os = "cloudabi", target_os = "unknown")))]
    {
        extern {
            #[cfg(not(target_os = "dragonfly"))]
            #[cfg_attr(any(target_os = "macos",
                           target_os = "ios",
                           target_os = "freebsd"),
                       link_name = "__error")]
            #[cfg_attr(any(target_os = "openbsd",
                           target_os = "netbsd",
                           target_os = "bitrig",
                           target_os = "android"),
                       link_name = "__errno")]
            #[cfg_attr(target_os = "solaris",
                       link_name = "___errno")]
            #[cfg_attr(target_os = "linux",
                       link_name = "__errno_location")]
            #[cfg_attr(target_os = "windows",
                       link_name = "_errno")]
            fn errno_location() -> *mut libc::c_int;
        }

        return unsafe {
            *(errno_location())
        }
    }

    #[cfg(any(target_os = "cloudabi", target_os = "wasi"))]
    {
        extern {
            #[thread_local]
            static errno: i32;
        }

        return errno;
    }

    #[cfg(target_os = "vxworks")]
    {
        extern "C" {
            pub fn errnoGet() -> libc::c_int;
        }

        return unsafe {
            errnoGet();
        }
    }

    #[cfg(target_os = "unknown")]
    {
        return 0;
    }
}

pub fn write_error(code: i32, res: &mut crate::Str) {
    #[cfg(any(windows, all(unix, not(target_env = "gnu"))))]
    extern "C" {
        ///Only GNU impl is thread unsafe
        fn strerror(code: i32) -> *const i8;
        fn strlen(text: *const i8) -> usize;
    }

    #[cfg(all(unix, target_env = "gnu"))]
    extern "C" {
        fn strerror_l(code: i32, locale: *mut i8) -> *const i8;
        fn strlen(text: *const i8) -> usize;
    }

    #[cfg(all(unix, target_env = "gnu"))]
    #[inline]
    unsafe fn strerror(code: i32) -> *const i8 {
        strerror_l(code, core::ptr::null_mut())
    }

    #[cfg(any(windows, unix))]
    {
        let err = unsafe {
            strerror(code)
        };

        if !err.is_null() {
            let err_slice = unsafe {
                core::slice::from_raw_parts(err as *const u8, strlen(err))
            };

            match core::str::from_utf8(err_slice) {
                Ok(msg) => res.push_str(msg),
                Err(_) => res.push_str(crate::FAIL_FORMAT),
            };

            return;
        }
    }

    match code {
        0 => res.push_str("operation successful"),
        _ => res.push_str(crate::UNKNOWN_ERROR),
    };
}

pub fn to_error(code: i32) -> crate::Str {
    let mut res = crate::Str::new();
    write_error(code, &mut res);
    res
}

impl crate::Category for PosixCategory {
    const NAME: &'static str = "Posix error";

    #[inline(always)]
    fn message<'a>(code: i32) -> crate::Str {
        to_error(code)
    }
}
