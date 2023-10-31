use super::super::c;
use crate::fd::BorrowedFd;
use bitflags::bitflags;

/// `struct timespec`
pub type Timespec = linux_raw_sys::general::__kernel_timespec;

/// A type for the `tv_sec` field of [`Timespec`].
pub type Secs = linux_raw_sys::general::__kernel_time64_t;

/// A type for the `tv_nsec` field of [`Timespec`].
pub type Nsecs = i64;

/// `struct itimerspec` for use with [`timerfd_gettime`] and
/// [`timerfd_settime`].
///
/// [`timerfd_gettime`]: crate::time::timerfd_gettime
/// [`timerfd_settime`]: crate::time::timerfd_settime
pub type Itimerspec = linux_raw_sys::general::__kernel_itimerspec;

/// `CLOCK_*` constants for use with [`clock_gettime`].
///
/// These constants are always supported at runtime, so `clock_gettime` never
/// has to fail with `INVAL` due to an unsupported clock. See
/// [`DynamicClockId`] for a greater set of clocks, with the caveat that not
/// all of them are always supported.
///
/// [`clock_gettime`]: crate::time::clock_gettime
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[repr(u32)]
#[non_exhaustive]
pub enum ClockId {
    /// `CLOCK_REALTIME`
    Realtime = linux_raw_sys::general::CLOCK_REALTIME,

    /// `CLOCK_MONOTONIC`
    Monotonic = linux_raw_sys::general::CLOCK_MONOTONIC,

    /// `CLOCK_PROCESS_CPUTIME_ID`
    ProcessCPUTime = linux_raw_sys::general::CLOCK_PROCESS_CPUTIME_ID,

    /// `CLOCK_THREAD_CPUTIME_ID`
    ThreadCPUTime = linux_raw_sys::general::CLOCK_THREAD_CPUTIME_ID,

    /// `CLOCK_REALTIME_COARSE`
    RealtimeCoarse = linux_raw_sys::general::CLOCK_REALTIME_COARSE,

    /// `CLOCK_MONOTONIC_COARSE`
    MonotonicCoarse = linux_raw_sys::general::CLOCK_MONOTONIC_COARSE,

    /// `CLOCK_MONOTONIC_RAW`
    MonotonicRaw = linux_raw_sys::general::CLOCK_MONOTONIC_RAW,
}

/// `CLOCK_*` constants for use with [`clock_gettime_dynamic`].
///
/// These constants may be unsupported at runtime, depending on the OS version,
/// and `clock_gettime_dynamic` may fail with `INVAL`. See [`ClockId`] for
/// clocks which are always supported at runtime.
///
/// [`clock_gettime_dynamic`]: crate::time::clock_gettime_dynamic
#[derive(Debug, Copy, Clone)]
#[non_exhaustive]
pub enum DynamicClockId<'a> {
    /// `ClockId` values that are always supported at runtime.
    Known(ClockId),

    /// Linux dynamic clocks.
    Dynamic(BorrowedFd<'a>),

    /// `CLOCK_REALTIME_ALARM`, available on Linux >= 3.0
    RealtimeAlarm,

    /// `CLOCK_TAI`, available on Linux >= 3.10
    Tai,

    /// `CLOCK_BOOTTIME`, available on Linux >= 2.6.39
    Boottime,

    /// `CLOCK_BOOTTIME_ALARM`, available on Linux >= 2.6.39
    BoottimeAlarm,
}

bitflags! {
    /// `TFD_*` flags for use with [`timerfd_create`].
    ///
    /// [`timerfd_create`]: crate::time::timerfd_create
    pub struct TimerfdFlags: c::c_uint {
        /// `TFD_NONBLOCK`
        const NONBLOCK = linux_raw_sys::general::TFD_NONBLOCK;

        /// `TFD_CLOEXEC`
        const CLOEXEC = linux_raw_sys::general::TFD_CLOEXEC;
    }
}

bitflags! {
    /// `TFD_TIMER_*` flags for use with [`timerfd_settime`].
    ///
    /// [`timerfd_settime`]: crate::time::timerfd_settime
    pub struct TimerfdTimerFlags: c::c_uint {
        /// `TFD_TIMER_ABSTIME`
        const ABSTIME = linux_raw_sys::general::TFD_TIMER_ABSTIME;

        /// `TFD_TIMER_CANCEL_ON_SET`
        const CANCEL_ON_SET = linux_raw_sys::general::TFD_TIMER_CANCEL_ON_SET;
    }
}

/// `CLOCK_*` constants for use with [`timerfd_create`].
///
/// [`timerfd_create`]: crate::time::timerfd_create
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[repr(u32)]
#[non_exhaustive]
pub enum TimerfdClockId {
    /// `CLOCK_REALTIME`—A clock that tells the “real” time.
    ///
    /// This is a clock that tells the amount of time elapsed since the
    /// Unix epoch, 1970-01-01T00:00:00Z. The clock is externally settable, so
    /// it is not monotonic. Successive reads may see decreasing times, so it
    /// isn't reliable for measuring durations.
    Realtime = linux_raw_sys::general::CLOCK_REALTIME,

    /// `CLOCK_MONOTONIC`—A clock that tells an abstract time.
    ///
    /// Unlike `Realtime`, this clock is not based on a fixed known epoch, so
    /// individual times aren't meaningful. However, since it isn't settable,
    /// it is reliable for measuring durations.
    ///
    /// This clock does not advance while the system is suspended; see
    /// `Boottime` for a clock that does.
    Monotonic = linux_raw_sys::general::CLOCK_MONOTONIC,

    /// `CLOCK_BOOTTIME`—Like `Monotonic`, but advances while suspended.
    ///
    /// This clock is similar to `Monotonic`, but does advance while the system
    /// is suspended.
    Boottime = linux_raw_sys::general::CLOCK_BOOTTIME,

    /// `CLOCK_REALTIME_ALARM`—Like `Realtime`, but wakes a suspended system.
    ///
    /// This clock is like `Realtime`, but can wake up a suspended system.
    ///
    /// Use of this clock requires the `CAP_WAKE_ALARM` Linux capability.
    RealtimeAlarm = linux_raw_sys::general::CLOCK_REALTIME_ALARM,

    /// `CLOCK_BOOTTIME_ALARM`—Like `Boottime`, but wakes a suspended system.
    ///
    /// This clock is like `Boottime`, but can wake up a suspended system.
    ///
    /// Use of this clock requires the `CAP_WAKE_ALARM` Linux capability.
    BoottimeAlarm = linux_raw_sys::general::CLOCK_BOOTTIME_ALARM,
}
