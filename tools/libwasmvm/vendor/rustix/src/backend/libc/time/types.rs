use super::super::c;
#[cfg(not(target_os = "wasi"))]
use crate::fd::BorrowedFd;
#[cfg(any(linux_kernel, target_os = "fuchsia"))]
use bitflags::bitflags;

/// `struct timespec`
#[cfg(not(all(
    any(target_arch = "arm", target_arch = "mips", target_arch = "x86"),
    target_env = "gnu",
)))]
pub type Timespec = c::timespec;

/// `struct timespec`
#[cfg(all(
    any(target_arch = "arm", target_arch = "mips", target_arch = "x86"),
    target_env = "gnu",
))]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Timespec {
    /// Seconds.
    pub tv_sec: Secs,

    /// Nanoseconds. Must be less than 1_000_000_000.
    pub tv_nsec: Nsecs,
}

/// A type for the `tv_sec` field of [`Timespec`].
#[cfg(not(all(
    any(target_arch = "arm", target_arch = "mips", target_arch = "x86"),
    target_env = "gnu",
)))]
#[allow(deprecated)]
pub type Secs = c::time_t;

/// A type for the `tv_sec` field of [`Timespec`].
#[cfg(all(
    any(target_arch = "arm", target_arch = "mips", target_arch = "x86"),
    target_env = "gnu",
))]
pub type Secs = i64;

/// A type for the `tv_nsec` field of [`Timespec`].
#[cfg(all(target_arch = "x86_64", target_pointer_width = "32"))]
pub type Nsecs = i64;

/// A type for the `tv_nsec` field of [`Timespec`].
#[cfg(not(all(target_arch = "x86_64", target_pointer_width = "32")))]
pub type Nsecs = c::c_long;

/// On most platforms, `LibcTimespec` is just `Timespec`.
#[cfg(not(all(
    any(target_arch = "arm", target_arch = "mips", target_arch = "x86"),
    target_env = "gnu",
)))]
pub(crate) type LibcTimespec = Timespec;

/// On 32-bit glibc platforms, `timespec` has anonymous padding fields, which
/// Rust doesn't support yet (see `unnamed_fields`), so we define our own
/// struct with explicit padding, with bidirectional `From` impls.
#[cfg(all(
    any(target_arch = "arm", target_arch = "mips", target_arch = "x86"),
    target_env = "gnu",
))]
#[repr(C)]
#[derive(Debug, Clone)]
pub(crate) struct LibcTimespec {
    pub(crate) tv_sec: Secs,

    #[cfg(target_endian = "big")]
    padding: core::mem::MaybeUninit<u32>,

    pub(crate) tv_nsec: Nsecs,

    #[cfg(target_endian = "little")]
    padding: core::mem::MaybeUninit<u32>,
}

#[cfg(all(
    any(target_arch = "arm", target_arch = "mips", target_arch = "x86"),
    target_env = "gnu",
))]
impl From<LibcTimespec> for Timespec {
    #[inline]
    fn from(t: LibcTimespec) -> Self {
        Self {
            tv_sec: t.tv_sec,
            tv_nsec: t.tv_nsec,
        }
    }
}

#[cfg(all(
    any(target_arch = "arm", target_arch = "mips", target_arch = "x86"),
    target_env = "gnu",
))]
impl From<Timespec> for LibcTimespec {
    #[inline]
    fn from(t: Timespec) -> Self {
        Self {
            tv_sec: t.tv_sec,
            tv_nsec: t.tv_nsec,
            padding: core::mem::MaybeUninit::uninit(),
        }
    }
}

/// `CLOCK_*` constants for use with [`clock_gettime`].
///
/// These constants are always supported at runtime, so `clock_gettime` never
/// has to fail with `INVAL` due to an unsupported clock. See
/// [`DynamicClockId`] for a greater set of clocks, with the caveat that not
/// all of them are always supported.
///
/// [`clock_gettime`]: crate::time::clock_gettime
#[cfg(not(any(apple, target_os = "wasi")))]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(not(target_os = "dragonfly"), repr(i32))]
#[cfg_attr(target_os = "dragonfly", repr(u64))]
#[non_exhaustive]
pub enum ClockId {
    /// `CLOCK_REALTIME`
    Realtime = c::CLOCK_REALTIME,

    /// `CLOCK_MONOTONIC`
    Monotonic = c::CLOCK_MONOTONIC,

    /// `CLOCK_UPTIME`
    #[cfg(any(freebsdlike, target_os = "openbsd"))]
    Uptime = c::CLOCK_UPTIME,

    /// `CLOCK_PROCESS_CPUTIME_ID`
    #[cfg(not(any(solarish, target_os = "netbsd", target_os = "redox")))]
    ProcessCPUTime = c::CLOCK_PROCESS_CPUTIME_ID,

    /// `CLOCK_THREAD_CPUTIME_ID`
    #[cfg(not(any(solarish, target_os = "netbsd", target_os = "redox")))]
    ThreadCPUTime = c::CLOCK_THREAD_CPUTIME_ID,

    /// `CLOCK_REALTIME_COARSE`
    #[cfg(any(linux_kernel, target_os = "freebsd"))]
    RealtimeCoarse = c::CLOCK_REALTIME_COARSE,

    /// `CLOCK_MONOTONIC_COARSE`
    #[cfg(any(linux_kernel, target_os = "freebsd"))]
    MonotonicCoarse = c::CLOCK_MONOTONIC_COARSE,

    /// `CLOCK_MONOTONIC_RAW`
    #[cfg(linux_kernel)]
    MonotonicRaw = c::CLOCK_MONOTONIC_RAW,
}

/// `CLOCK_*` constants for use with [`clock_gettime`].
///
/// These constants are always supported at runtime, so `clock_gettime` never
/// has to fail with `INVAL` due to an unsupported clock. See
/// [`DynamicClockId`] for a greater set of clocks, with the caveat that not
/// all of them are always supported.
#[cfg(apple)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[repr(u32)]
#[non_exhaustive]
pub enum ClockId {
    /// `CLOCK_REALTIME`
    Realtime = c::CLOCK_REALTIME,

    /// `CLOCK_MONOTONIC`
    Monotonic = c::CLOCK_MONOTONIC,

    /// `CLOCK_PROCESS_CPUTIME_ID`
    ProcessCPUTime = c::CLOCK_PROCESS_CPUTIME_ID,

    /// `CLOCK_THREAD_CPUTIME_ID`
    ThreadCPUTime = c::CLOCK_THREAD_CPUTIME_ID,
}

/// `CLOCK_*` constants for use with [`clock_gettime_dynamic`].
///
/// These constants may be unsupported at runtime, depending on the OS version,
/// and `clock_gettime_dynamic` may fail with `INVAL`. See [`ClockId`] for
/// clocks which are always supported at runtime.
///
/// [`clock_gettime_dynamic`]: crate::time::clock_gettime_dynamic
#[cfg(not(target_os = "wasi"))]
#[derive(Debug, Copy, Clone)]
#[non_exhaustive]
pub enum DynamicClockId<'a> {
    /// `ClockId` values that are always supported at runtime.
    Known(ClockId),

    /// Linux dynamic clocks.
    Dynamic(BorrowedFd<'a>),

    /// `CLOCK_REALTIME_ALARM`, available on Linux >= 3.0
    #[cfg(linux_kernel)]
    RealtimeAlarm,

    /// `CLOCK_TAI`, available on Linux >= 3.10
    #[cfg(linux_kernel)]
    Tai,

    /// `CLOCK_BOOTTIME`, available on Linux >= 2.6.39
    #[cfg(any(linux_kernel, target_os = "openbsd"))]
    Boottime,

    /// `CLOCK_BOOTTIME_ALARM`, available on Linux >= 2.6.39
    #[cfg(linux_kernel)]
    BoottimeAlarm,
}

/// `struct itimerspec` for use with [`timerfd_gettime`] and
/// [`timerfd_settime`].
///
/// [`timerfd_gettime`]: crate::time::timerfd_gettime
/// [`timerfd_settime`]: crate::time::timerfd_settime
#[cfg(any(linux_kernel, target_os = "fuchsia"))]
#[cfg(not(all(
    any(target_arch = "arm", target_arch = "mips", target_arch = "x86"),
    target_env = "gnu",
)))]
pub type Itimerspec = c::itimerspec;

/// `struct itimerspec` for use with [`timerfd_gettime`] and
/// [`timerfd_settime`].
///
/// [`timerfd_gettime`]: crate::time::timerfd_gettime
/// [`timerfd_settime`]: crate::time::timerfd_settime
#[cfg(any(linux_kernel, target_os = "fuchsia"))]
#[cfg(all(
    any(target_arch = "arm", target_arch = "mips", target_arch = "x86"),
    target_env = "gnu",
))]
#[allow(missing_docs)]
#[repr(C)]
#[derive(Debug, Clone)]
pub struct Itimerspec {
    pub it_interval: Timespec,
    pub it_value: Timespec,
}

/// On most platforms, `LibcItimerspec` is just `Itimerspec`.
#[cfg(any(linux_kernel, target_os = "fuchsia"))]
#[cfg(not(all(
    any(target_arch = "arm", target_arch = "mips", target_arch = "x86"),
    target_env = "gnu",
)))]
pub(crate) type LibcItimerspec = Itimerspec;

/// On 32-bit glibc platforms, `LibcTimespec` differs from `Timespec`, so we
/// define our own struct, with bidirectional `From` impls.
#[cfg(any(linux_kernel, target_os = "fuchsia"))]
#[cfg(all(
    any(target_arch = "arm", target_arch = "mips", target_arch = "x86"),
    target_env = "gnu",
))]
#[repr(C)]
#[derive(Debug, Clone)]
pub(crate) struct LibcItimerspec {
    pub it_interval: LibcTimespec,
    pub it_value: LibcTimespec,
}

#[cfg(any(linux_kernel, target_os = "fuchsia"))]
#[cfg(all(
    any(target_arch = "arm", target_arch = "mips", target_arch = "x86"),
    target_env = "gnu",
))]
impl From<LibcItimerspec> for Itimerspec {
    #[inline]
    fn from(t: LibcItimerspec) -> Self {
        Self {
            it_interval: t.it_interval.into(),
            it_value: t.it_value.into(),
        }
    }
}

#[cfg(any(linux_kernel, target_os = "fuchsia"))]
#[cfg(all(
    any(target_arch = "arm", target_arch = "mips", target_arch = "x86"),
    target_env = "gnu",
))]
impl From<Itimerspec> for LibcItimerspec {
    #[inline]
    fn from(t: Itimerspec) -> Self {
        Self {
            it_interval: t.it_interval.into(),
            it_value: t.it_value.into(),
        }
    }
}

#[cfg(any(linux_kernel, target_os = "fuchsia"))]
bitflags! {
    /// `TFD_*` flags for use with [`timerfd_create`].
    ///
    /// [`timerfd_create`]: crate::time::timerfd_create
    pub struct TimerfdFlags: c::c_int {
        /// `TFD_NONBLOCK`
        const NONBLOCK = c::TFD_NONBLOCK;

        /// `TFD_CLOEXEC`
        const CLOEXEC = c::TFD_CLOEXEC;
    }
}

#[cfg(any(linux_kernel, target_os = "fuchsia"))]
bitflags! {
    /// `TFD_TIMER_*` flags for use with [`timerfd_settime`].
    ///
    /// [`timerfd_settime`]: crate::time::timerfd_settime
    pub struct TimerfdTimerFlags: c::c_int {
        /// `TFD_TIMER_ABSTIME`
        const ABSTIME = c::TFD_TIMER_ABSTIME;

        /// `TFD_TIMER_CANCEL_ON_SET`
        #[cfg(linux_kernel)]
        const CANCEL_ON_SET = c::TFD_TIMER_CANCEL_ON_SET;
    }
}

/// `CLOCK_*` constants for use with [`timerfd_create`].
///
/// [`timerfd_create`]: crate::time::timerfd_create
#[cfg(any(linux_kernel, target_os = "fuchsia"))]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[repr(i32)]
#[non_exhaustive]
pub enum TimerfdClockId {
    /// `CLOCK_REALTIME`—A clock that tells the “real” time.
    ///
    /// This is a clock that tells the amount of time elapsed since the
    /// Unix epoch, 1970-01-01T00:00:00Z. The clock is externally settable, so
    /// it is not monotonic. Successive reads may see decreasing times, so it
    /// isn't reliable for measuring durations.
    Realtime = c::CLOCK_REALTIME,

    /// `CLOCK_MONOTONIC`—A clock that tells an abstract time.
    ///
    /// Unlike `Realtime`, this clock is not based on a fixed known epoch, so
    /// individual times aren't meaningful. However, since it isn't settable,
    /// it is reliable for measuring durations.
    ///
    /// This clock does not advance while the system is suspended; see
    /// `Boottime` for a clock that does.
    Monotonic = c::CLOCK_MONOTONIC,

    /// `CLOCK_BOOTTIME`—Like `Monotonic`, but advances while suspended.
    ///
    /// This clock is similar to `Monotonic`, but does advance while the system
    /// is suspended.
    Boottime = c::CLOCK_BOOTTIME,

    /// `CLOCK_REALTIME_ALARM`—Like `Realtime`, but wakes a suspended system.
    ///
    /// This clock is like `Realtime`, but can wake up a suspended system.
    ///
    /// Use of this clock requires the `CAP_WAKE_ALARM` Linux capability.
    RealtimeAlarm = c::CLOCK_REALTIME_ALARM,

    /// `CLOCK_BOOTTIME_ALARM`—Like `Boottime`, but wakes a suspended system.
    ///
    /// This clock is like `Boottime`, but can wake up a suspended system.
    ///
    /// Use of this clock requires the `CAP_WAKE_ALARM` Linux capability.
    BoottimeAlarm = c::CLOCK_BOOTTIME_ALARM,
}
