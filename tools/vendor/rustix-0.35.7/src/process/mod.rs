//! Process-associated operations.

#[cfg(not(target_os = "wasi"))]
mod chdir;
mod exit;
#[cfg(not(target_os = "wasi"))] // WASI doesn't have get[gpu]id.
mod id;
#[cfg(not(target_os = "wasi"))]
mod kill;
#[cfg(any(target_os = "android", target_os = "linux"))]
mod membarrier;
#[cfg(not(any(target_os = "fuchsia", target_os = "wasi")))] // WASI doesn't have [gs]etpriority.
mod priority;
#[cfg(not(any(target_os = "fuchsia", target_os = "redox", target_os = "wasi")))]
mod rlimit;
#[cfg(any(
    target_os = "android",
    target_os = "dragonfly",
    target_os = "fuchsia",
    target_os = "linux",
))]
mod sched;
mod sched_yield;
#[cfg(not(target_os = "wasi"))] // WASI doesn't have uname.
mod uname;
#[cfg(not(target_os = "wasi"))]
mod wait;

#[cfg(not(target_os = "wasi"))]
pub use chdir::chdir;
#[cfg(not(any(target_os = "wasi", target_os = "fuchsia")))]
pub use chdir::fchdir;
#[cfg(not(target_os = "wasi"))]
pub use chdir::getcwd;
#[cfg(not(target_os = "wasi"))]
pub use exit::EXIT_SIGNALED_SIGABRT;
pub use exit::{EXIT_FAILURE, EXIT_SUCCESS};
#[cfg(any(target_os = "android", target_os = "linux"))]
pub use id::Cpuid;
#[cfg(not(target_os = "wasi"))]
pub use id::{
    getegid, geteuid, getgid, getpid, getppid, getuid, setsid, Gid, Pid, RawGid, RawNonZeroPid,
    RawPid, RawUid, Uid,
};
#[cfg(not(target_os = "wasi"))]
pub use kill::{kill_current_process_group, kill_process, kill_process_group, Signal};
#[cfg(any(target_os = "android", target_os = "linux"))]
pub use membarrier::{
    membarrier, membarrier_cpu, membarrier_query, MembarrierCommand, MembarrierQuery,
};
#[cfg(not(any(target_os = "fuchsia", target_os = "wasi")))]
pub use priority::nice;
#[cfg(not(any(target_os = "fuchsia", target_os = "redox", target_os = "wasi")))]
pub use priority::{
    getpriority_pgrp, getpriority_process, getpriority_user, setpriority_pgrp, setpriority_process,
    setpriority_user,
};
#[cfg(any(target_os = "android", target_os = "linux"))]
pub use rlimit::prlimit;
#[cfg(not(any(target_os = "fuchsia", target_os = "redox", target_os = "wasi")))]
pub use rlimit::{getrlimit, setrlimit, Resource, Rlimit};
#[cfg(any(
    target_os = "android",
    target_os = "dragonfly",
    target_os = "fuchsia",
    target_os = "linux",
))]
pub use sched::{sched_getaffinity, sched_setaffinity, CpuSet};
pub use sched_yield::sched_yield;
#[cfg(not(target_os = "wasi"))]
pub use uname::{uname, Uname};
#[cfg(not(target_os = "wasi"))]
pub use wait::{wait, waitpid, WaitOptions, WaitStatus};

#[cfg(not(target_os = "wasi"))]
pub(crate) use id::translate_fchown_args;
