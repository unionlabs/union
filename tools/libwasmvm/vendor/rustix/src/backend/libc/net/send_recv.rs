use super::super::c;
use bitflags::bitflags;

bitflags! {
    /// `MSG_* flags for use with [`send`], [`send_to`], and related functions.
    ///
    /// [`send`]: crate::net::send
    /// [`sendto`]: crate::net::sendto
    pub struct SendFlags: i32 {
        /// `MSG_CONFIRM`
        #[cfg(not(any(
            bsd,
            solarish,
            windows,
            target_os = "haiku",
        )))]
        const CONFIRM = c::MSG_CONFIRM;
        /// `MSG_DONTROUTE`
        const DONTROUTE = c::MSG_DONTROUTE;
        /// `MSG_DONTWAIT`
        #[cfg(not(windows))]
        const DONTWAIT = c::MSG_DONTWAIT;
        /// `MSG_EOR`
        #[cfg(not(windows))]
        const EOT = c::MSG_EOR;
        /// `MSG_MORE`
        #[cfg(not(any(
            bsd,
            solarish,
            windows,
            target_os = "haiku",
        )))]
        const MORE = c::MSG_MORE;
        #[cfg(not(any(apple, windows)))]
        /// `MSG_NOSIGNAL`
        const NOSIGNAL = c::MSG_NOSIGNAL;
        /// `MSG_OOB`
        const OOB = c::MSG_OOB;
    }
}

bitflags! {
    /// `MSG_* flags for use with [`recv`], [`recvfrom`], and related functions.
    ///
    /// [`recv`]: crate::net::recv
    /// [`recvfrom`]: crate::net::recvfrom
    pub struct RecvFlags: i32 {
        #[cfg(not(any(apple, solarish, windows, target_os = "haiku")))]
        /// `MSG_CMSG_CLOEXEC`
        const CMSG_CLOEXEC = c::MSG_CMSG_CLOEXEC;
        /// `MSG_DONTWAIT`
        #[cfg(not(windows))]
        const DONTWAIT = c::MSG_DONTWAIT;
        /// `MSG_ERRQUEUE`
        #[cfg(not(any(
            bsd,
            solarish,
            windows,
            target_os = "haiku",
        )))]
        const ERRQUEUE = c::MSG_ERRQUEUE;
        /// `MSG_OOB`
        const OOB = c::MSG_OOB;
        /// `MSG_PEEK`
        const PEEK = c::MSG_PEEK;
        /// `MSG_TRUNC`
        const TRUNC = c::MSG_TRUNC as c::c_int;
        /// `MSG_WAITALL`
        const WAITALL = c::MSG_WAITALL;
    }
}
