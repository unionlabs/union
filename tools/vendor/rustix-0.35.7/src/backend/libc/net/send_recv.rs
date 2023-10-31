use super::super::c;
use bitflags::bitflags;

bitflags! {
    /// `MSG_*`
    pub struct SendFlags: i32 {
        /// `MSG_CONFIRM`
        #[cfg(not(any(
            windows,
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "illumos",
            target_os = "ios",
            target_os = "macos",
            target_os = "netbsd",
            target_os = "openbsd",
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
            windows,
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "illumos",
            target_os = "ios",
            target_os = "macos",
            target_os = "netbsd",
            target_os = "openbsd",
        )))]
        const MORE = c::MSG_MORE;
        #[cfg(not(any(windows, target_os = "ios", target_os = "macos")))]
        /// `MSG_NOSIGNAL`
        const NOSIGNAL = c::MSG_NOSIGNAL;
        /// `MSG_OOB`
        const OOB = c::MSG_OOB;
    }
}

bitflags! {
    /// `MSG_*`
    pub struct RecvFlags: i32 {
        #[cfg(not(any(windows, target_os = "illumos", target_os = "ios", target_os = "macos")))]
        /// `MSG_CMSG_CLOEXEC`
        const CMSG_CLOEXEC = c::MSG_CMSG_CLOEXEC;
        /// `MSG_DONTWAIT`
        #[cfg(not(windows))]
        const DONTWAIT = c::MSG_DONTWAIT;
        /// `MSG_ERRQUEUE`
        #[cfg(not(any(
            windows,
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "illumos",
            target_os = "ios",
            target_os = "macos",
            target_os = "netbsd",
            target_os = "openbsd",
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
