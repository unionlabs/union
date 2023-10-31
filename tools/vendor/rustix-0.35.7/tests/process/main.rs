//! Tests for [`rustix::process`].

#![cfg(feature = "process")]
#![cfg(not(windows))]
#![cfg_attr(target_os = "wasi", feature(wasi_ext))]
#![cfg_attr(io_lifetimes_use_std, feature(io_safety))]
#![cfg_attr(core_c_str, feature(core_c_str))]

#[cfg(not(target_os = "wasi"))]
#[macro_use]
mod weak;

mod cpu_set;
#[cfg(not(target_os = "wasi"))] // WASI doesn't have get[gpu]id.
mod id;
#[cfg(any(target_os = "android", target_os = "linux"))]
mod membarrier;
#[cfg(not(any(target_os = "fuchsia", target_os = "wasi")))] // WASI doesn't have [gs]etpriority.
mod priority;
#[cfg(not(any(target_os = "fuchsia", target_os = "redox", target_os = "wasi")))]
mod rlimit;
mod sched_yield;
#[cfg(not(target_os = "wasi"))] // WASI doesn't have uname.
mod uname;
#[cfg(not(target_os = "wasi"))] // WASI doesn't have waitpid.
mod wait;
#[cfg(not(any(target_os = "redox", target_os = "wasi")))]
mod working_directory;
