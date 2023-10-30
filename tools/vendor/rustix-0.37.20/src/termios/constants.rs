use crate::backend;

pub use backend::termios::types::*;

/// Translate from a `Speed` code to a speed value `u32`.
///
/// ```
/// let speed = rustix::termios::speed_value(rustix::termios::B57600);
/// assert_eq!(speed, Some(57600));
/// ```
pub fn speed_value(speed: backend::termios::types::Speed) -> Option<u32> {
    match speed {
        backend::termios::types::B0 => Some(0),
        backend::termios::types::B50 => Some(50),
        backend::termios::types::B75 => Some(75),
        backend::termios::types::B110 => Some(110),
        backend::termios::types::B134 => Some(134),
        backend::termios::types::B150 => Some(150),
        backend::termios::types::B200 => Some(200),
        backend::termios::types::B300 => Some(300),
        backend::termios::types::B600 => Some(600),
        backend::termios::types::B1200 => Some(1200),
        backend::termios::types::B1800 => Some(1800),
        backend::termios::types::B2400 => Some(2400),
        backend::termios::types::B4800 => Some(4800),
        backend::termios::types::B9600 => Some(9600),
        backend::termios::types::B19200 => Some(19200),
        backend::termios::types::B38400 => Some(38400),
        #[cfg(not(target_os = "aix"))]
        backend::termios::types::B57600 => Some(57600),
        #[cfg(not(target_os = "aix"))]
        backend::termios::types::B115200 => Some(115_200),
        #[cfg(not(target_os = "aix"))]
        backend::termios::types::B230400 => Some(230_400),
        #[cfg(not(any(
            apple,
            target_os = "aix",
            target_os = "dragonfly",
            target_os = "haiku",
            target_os = "openbsd"
        )))]
        backend::termios::types::B460800 => Some(460_800),
        #[cfg(not(any(bsd, solarish, target_os = "aix", target_os = "haiku")))]
        backend::termios::types::B500000 => Some(500_000),
        #[cfg(not(any(bsd, solarish, target_os = "aix", target_os = "haiku")))]
        backend::termios::types::B576000 => Some(576_000),
        #[cfg(not(any(
            apple,
            target_os = "aix",
            target_os = "dragonfly",
            target_os = "haiku",
            target_os = "openbsd"
        )))]
        backend::termios::types::B921600 => Some(921_600),
        #[cfg(not(any(bsd, target_os = "aix", target_os = "haiku", target_os = "solaris")))]
        backend::termios::types::B1000000 => Some(1_000_000),
        #[cfg(not(any(bsd, target_os = "aix", target_os = "haiku", target_os = "solaris")))]
        backend::termios::types::B1152000 => Some(1_152_000),
        #[cfg(not(any(bsd, target_os = "aix", target_os = "haiku", target_os = "solaris")))]
        backend::termios::types::B1500000 => Some(1_500_000),
        #[cfg(not(any(bsd, target_os = "aix", target_os = "haiku", target_os = "solaris")))]
        backend::termios::types::B2000000 => Some(2_000_000),
        #[cfg(not(any(
            target_arch = "sparc",
            target_arch = "sparc64",
            bsd,
            target_os = "aix",
            target_os = "haiku",
            target_os = "solaris",
        )))]
        backend::termios::types::B2500000 => Some(2_500_000),
        #[cfg(not(any(
            target_arch = "sparc",
            target_arch = "sparc64",
            bsd,
            target_os = "aix",
            target_os = "haiku",
            target_os = "solaris",
        )))]
        backend::termios::types::B3000000 => Some(3_000_000),
        #[cfg(not(any(
            target_arch = "sparc",
            target_arch = "sparc64",
            bsd,
            target_os = "aix",
            target_os = "haiku",
            target_os = "solaris",
        )))]
        backend::termios::types::B3500000 => Some(3_500_000),
        #[cfg(not(any(
            target_arch = "sparc",
            target_arch = "sparc64",
            bsd,
            target_os = "aix",
            target_os = "haiku",
            target_os = "solaris",
        )))]
        backend::termios::types::B4000000 => Some(4_000_000),
        _ => None,
    }
}
