use crate::backend::c;
use bitflags::bitflags;

bitflags! {
    /// `MSG_* flags for use with [`send`], [`send_to`], and related functions.
    ///
    /// [`send`]: crate::net::send
    /// [`sendto`]: crate::net::sendto
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
    pub struct SendFlags: u32 {
        /// `MSG_CONFIRM`
        #[cfg(not(any(
            bsd,
            solarish,
            windows,
            target_os = "haiku",
        )))]
        const CONFIRM = bitcast!(c::MSG_CONFIRM);
        /// `MSG_DONTROUTE`
        const DONTROUTE = bitcast!(c::MSG_DONTROUTE);
        /// `MSG_DONTWAIT`
        #[cfg(not(windows))]
        const DONTWAIT = bitcast!(c::MSG_DONTWAIT);
        /// `MSG_EOR`
        #[cfg(not(windows))]
        const EOT = bitcast!(c::MSG_EOR);
        /// `MSG_MORE`
        #[cfg(not(any(
            bsd,
            solarish,
            windows,
            target_os = "haiku",
        )))]
        const MORE = bitcast!(c::MSG_MORE);
        #[cfg(not(any(apple, windows)))]
        /// `MSG_NOSIGNAL`
        const NOSIGNAL = bitcast!(c::MSG_NOSIGNAL);
        /// `MSG_OOB`
        const OOB = bitcast!(c::MSG_OOB);
    }
}

bitflags! {
    /// `MSG_* flags for use with [`recv`], [`recvfrom`], and related functions.
    ///
    /// [`recv`]: crate::net::recv
    /// [`recvfrom`]: crate::net::recvfrom
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
    pub struct RecvFlags: u32 {
        #[cfg(not(any(apple, solarish, windows, target_os = "haiku")))]
        /// `MSG_CMSG_CLOEXEC`
        const CMSG_CLOEXEC = bitcast!(c::MSG_CMSG_CLOEXEC);
        /// `MSG_DONTWAIT`
        #[cfg(not(windows))]
        const DONTWAIT = bitcast!(c::MSG_DONTWAIT);
        /// `MSG_ERRQUEUE`
        #[cfg(not(any(
            bsd,
            solarish,
            windows,
            target_os = "haiku",
        )))]
        const ERRQUEUE = bitcast!(c::MSG_ERRQUEUE);
        /// `MSG_OOB`
        const OOB = bitcast!(c::MSG_OOB);
        /// `MSG_PEEK`
        const PEEK = bitcast!(c::MSG_PEEK);
        /// `MSG_TRUNC`
        const TRUNC = bitcast!(c::MSG_TRUNC);
        /// `MSG_WAITALL`
        const WAITALL = bitcast!(c::MSG_WAITALL);
    }
}
