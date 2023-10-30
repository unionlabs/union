#[cfg(any(
    target_os = "android",
    target_os = "dragonfly",
    target_os = "fuchsia",
    target_os = "linux",
))]
pub(crate) mod cpu_set;
#[cfg(not(windows))]
pub(crate) mod syscalls;
pub(crate) mod types;
#[cfg(not(target_os = "wasi"))]
pub(crate) mod wait;
