//! Feature tests for OS functionality
pub use self::os::*;

#[cfg(any(target_os = "linux", target_os = "android"))]
mod os {
    use crate::sys::utsname::uname;

    // Features:
    // * atomic cloexec on socket: 2.6.27
    // * pipe2: 2.6.27
    // * accept4: 2.6.28

    static VERS_UNKNOWN: usize = 1;
    static VERS_2_6_18:  usize = 2;
    static VERS_2_6_27:  usize = 3;
    static VERS_2_6_28:  usize = 4;
    static VERS_3:       usize = 5;

    #[inline]
    fn digit(dst: &mut usize, b: u8) {
        *dst *= 10;
        *dst += (b - b'0') as usize;
    }

    fn parse_kernel_version() -> usize {
        let u = uname();

        let mut curr:  usize = 0;
        let mut major: usize = 0;
        let mut minor: usize = 0;
        let mut patch: usize = 0;

        for b in u.release().bytes() {
            if curr >= 3 {
                break;
            }

            match b {
                b'.' | b'-' => {
                    curr += 1;
                }
                b'0'..=b'9' => {
                    match curr {
                        0 => digit(&mut major, b),
                        1 => digit(&mut minor, b),
                        _ => digit(&mut patch, b),
                    }
                }
                _ => break,
            }
        }

        if major >= 3 {
            VERS_3
        } else if major >= 2 {
            if minor >= 7 {
                VERS_UNKNOWN
            } else if minor >= 6 {
                if patch >= 28 {
                    VERS_2_6_28
                } else if patch >= 27 {
                    VERS_2_6_27
                } else {
                    VERS_2_6_18
                }
            } else {
                VERS_UNKNOWN
            }
        } else {
            VERS_UNKNOWN
        }
    }

    fn kernel_version() -> usize {
        static mut KERNEL_VERS: usize = 0;

        unsafe {
            if KERNEL_VERS == 0 {
                KERNEL_VERS = parse_kernel_version();
            }

            KERNEL_VERS
        }
    }

    /// Check if the OS supports atomic close-on-exec for sockets
    pub fn socket_atomic_cloexec() -> bool {
        kernel_version() >= VERS_2_6_27
    }

    #[test]
    pub fn test_parsing_kernel_version() {
        assert!(kernel_version() > 0);
    }
}

#[cfg(any(
        target_os = "dragonfly",    // Since ???
        target_os = "freebsd",      // Since 10.0
        target_os = "illumos",      // Since ???
        target_os = "netbsd",       // Since 6.0
        target_os = "openbsd",      // Since 5.7
        target_os = "redox",        // Since 1-july-2020
))]
mod os {
    /// Check if the OS supports atomic close-on-exec for sockets
    pub const fn socket_atomic_cloexec() -> bool {
        true
    }
}

#[cfg(any(target_os = "macos",
          target_os = "ios",
          target_os = "fuchsia",
          target_os = "solaris"))]
mod os {
    /// Check if the OS supports atomic close-on-exec for sockets
    pub const fn socket_atomic_cloexec() -> bool {
        false
    }
}
