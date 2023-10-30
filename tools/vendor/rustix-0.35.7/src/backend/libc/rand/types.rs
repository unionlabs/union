#[cfg(target_os = "linux")]
use super::super::c;
#[cfg(target_os = "linux")]
use bitflags::bitflags;

#[cfg(target_os = "linux")]
bitflags! {
    /// `GRND_*` flags for use with [`getrandom`].
    ///
    /// [`getrandom`]: crate::rand::getrandom
    pub struct GetRandomFlags: u32 {
        /// `GRND_RANDOM`
        const RANDOM = c::GRND_RANDOM;
        /// `GRND_NONBLOCK`
        const NONBLOCK = c::GRND_NONBLOCK;
        /// `GRND_INSECURE`
        const INSECURE = c::GRND_INSECURE;
    }
}
