//! Time-related operations.

mod clock;
#[cfg(any(target_os = "android", target_os = "fuchsia", target_os = "linux"))]
#[cfg(feature = "time")]
mod timerfd;

// TODO: Convert WASI'S clock APIs to use handles rather than ambient clock
// identifiers, update `wasi-libc`, and then add support in `rustix`.
#[cfg(not(any(target_os = "redox", target_os = "wasi")))]
pub use clock::clock_getres;
#[cfg(not(target_os = "wasi"))]
pub use clock::{clock_gettime, clock_gettime_dynamic, ClockId, DynamicClockId};
pub use clock::{Nsecs, Secs, Timespec};
#[cfg(any(target_os = "android", target_os = "fuchsia", target_os = "linux"))]
#[cfg(feature = "time")]
pub use timerfd::{
    timerfd_create, timerfd_gettime, timerfd_settime, Itimerspec, TimerfdClockId, TimerfdFlags,
    TimerfdTimerFlags,
};
