//! Thread-associated operations.

#[cfg(not(target_os = "redox"))]
mod clock;
#[cfg(linux_raw)]
mod futex;
#[cfg(any(target_os = "android", target_os = "linux"))]
mod id;

#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "emscripten",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "openbsd",
    target_os = "redox",
    target_os = "wasi",
)))]
pub use clock::{clock_nanosleep_absolute, clock_nanosleep_relative, ClockId};
#[cfg(not(target_os = "redox"))]
pub use clock::{nanosleep, NanosleepRelativeResult, Timespec};
#[cfg(linux_raw)]
pub use futex::{futex, FutexFlags, FutexOperation};
#[cfg(any(target_os = "android", target_os = "linux"))]
pub use id::gettid;
