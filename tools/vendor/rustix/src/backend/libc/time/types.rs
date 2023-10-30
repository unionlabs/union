#[cfg(any(linux_kernel, target_os = "fuchsia"))]
use crate::backend::c;
#[cfg(all(
    any(target_arch = "arm", target_arch = "mips", target_arch = "x86"),
    target_env = "gnu",
))]
use crate::timespec::LibcTimespec;
#[cfg(any(linux_kernel, target_os = "fuchsia"))]
#[cfg(all(
    any(target_arch = "arm", target_arch = "mips", target_arch = "x86"),
    target_env = "gnu",
))]
use crate::timespec::Timespec;
#[cfg(any(linux_kernel, target_os = "fuchsia"))]
use bitflags::bitflags;

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
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
    pub struct TimerfdFlags: u32 {
        /// `TFD_NONBLOCK`
        const NONBLOCK = bitcast!(c::TFD_NONBLOCK);

        /// `TFD_CLOEXEC`
        const CLOEXEC = bitcast!(c::TFD_CLOEXEC);
    }
}

#[cfg(any(linux_kernel, target_os = "fuchsia"))]
bitflags! {
    /// `TFD_TIMER_*` flags for use with [`timerfd_settime`].
    ///
    /// [`timerfd_settime`]: crate::time::timerfd_settime
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
    pub struct TimerfdTimerFlags: u32 {
        /// `TFD_TIMER_ABSTIME`
        const ABSTIME = bitcast!(c::TFD_TIMER_ABSTIME);

        /// `TFD_TIMER_CANCEL_ON_SET`
        #[cfg(linux_kernel)]
        const CANCEL_ON_SET = bitcast!(c::TFD_TIMER_CANCEL_ON_SET);
    }
}

/// `CLOCK_*` constants for use with [`timerfd_create`].
///
/// [`timerfd_create`]: crate::time::timerfd_create
#[cfg(any(linux_kernel, target_os = "fuchsia"))]
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
    Realtime = bitcast!(c::CLOCK_REALTIME),

    /// `CLOCK_MONOTONIC`—A clock that tells an abstract time.
    ///
    /// Unlike `Realtime`, this clock is not based on a fixed known epoch, so
    /// individual times aren't meaningful. However, since it isn't settable,
    /// it is reliable for measuring durations.
    ///
    /// This clock does not advance while the system is suspended; see
    /// `Boottime` for a clock that does.
    Monotonic = bitcast!(c::CLOCK_MONOTONIC),

    /// `CLOCK_BOOTTIME`—Like `Monotonic`, but advances while suspended.
    ///
    /// This clock is similar to `Monotonic`, but does advance while the system
    /// is suspended.
    Boottime = bitcast!(c::CLOCK_BOOTTIME),

    /// `CLOCK_REALTIME_ALARM`—Like `Realtime`, but wakes a suspended system.
    ///
    /// This clock is like `Realtime`, but can wake up a suspended system.
    ///
    /// Use of this clock requires the `CAP_WAKE_ALARM` Linux capability.
    RealtimeAlarm = bitcast!(c::CLOCK_REALTIME_ALARM),

    /// `CLOCK_BOOTTIME_ALARM`—Like `Boottime`, but wakes a suspended system.
    ///
    /// This clock is like `Boottime`, but can wake up a suspended system.
    ///
    /// Use of this clock requires the `CAP_WAKE_ALARM` Linux capability.
    BoottimeAlarm = bitcast!(c::CLOCK_BOOTTIME_ALARM),
}

#[cfg(any(linux_kernel, target_os = "fuchsia"))]
#[test]
fn test_types() {
    use core::mem::size_of;
    assert_eq!(size_of::<TimerfdFlags>(), size_of::<c::c_int>());
    assert_eq!(size_of::<TimerfdTimerFlags>(), size_of::<c::c_int>());
}
