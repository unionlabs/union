use crate::backend::c;
use crate::fd::BorrowedFd;

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
