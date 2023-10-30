//! Configure the process resource limits.
use cfg_if::cfg_if;

use crate::errno::Errno;
use crate::Result;
pub use libc::rlim_t;
use std::mem;

cfg_if! {
    if #[cfg(all(target_os = "linux", target_env = "gnu"))]{
        use libc::{__rlimit_resource_t, rlimit, RLIM_INFINITY};
    }else if #[cfg(any(
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "macos",
        target_os = "ios",
        target_os = "android",
        target_os = "dragonfly",
        all(target_os = "linux", not(target_env = "gnu"))
    ))]{
        use libc::{c_int, rlimit, RLIM_INFINITY};
    }
}

libc_enum! {
    /// The Resource enum is platform dependent. Check different platform
    /// manuals for more details. Some platform links has been provided for
    /// earier reference (non-exhaustive).
    ///
    /// * [Linux](https://man7.org/linux/man-pages/man2/getrlimit.2.html)
    /// * [FreeBSD](https://www.freebsd.org/cgi/man.cgi?query=setrlimit)

    // linux-gnu uses u_int as resource enum, which is implemented in libc as
    // well.
    //
    // https://gcc.gnu.org/legacy-ml/gcc/2015-08/msg00441.html
    // https://github.com/rust-lang/libc/blob/master/src/unix/linux_like/linux/gnu/mod.rs
    #[cfg_attr(all(target_os = "linux", target_env = "gnu"), repr(u32))]
    #[cfg_attr(any(
            target_os = "freebsd",
            target_os = "openbsd",
            target_os = "netbsd",
            target_os = "macos",
            target_os = "ios",
            target_os = "android",
            target_os = "dragonfly",
            all(target_os = "linux", not(target_env = "gnu"))
        ), repr(i32))]
    #[non_exhaustive]
    pub enum Resource {
        #[cfg(not(any(
                    target_os = "freebsd",
                    target_os = "netbsd",
                    target_os = "openbsd"
        )))]
        /// The maximum amount (in bytes) of virtual memory the process is
        /// allowed to map.
        RLIMIT_AS,
        /// The largest size (in bytes) core(5) file that may be created.
        RLIMIT_CORE,
        /// The maximum amount of cpu time (in seconds) to be used by each
        /// process.
        RLIMIT_CPU,
        /// The maximum size (in bytes) of the data segment for a process
        RLIMIT_DATA,
        /// The largest size (in bytes) file that may be created.
        RLIMIT_FSIZE,
        /// The maximum number of open files for this process.
        RLIMIT_NOFILE,
        /// The maximum size (in bytes) of the stack segment for a process.
        RLIMIT_STACK,

        #[cfg(target_os = "freebsd")]
        /// The maximum number of kqueues this user id is allowed to create.
        RLIMIT_KQUEUES,

        #[cfg(any(target_os = "android", target_os = "linux"))]
        /// A limit on the combined number of flock locks and fcntl leases that
        /// this process may establish.
        RLIMIT_LOCKS,

        #[cfg(any(target_os = "android", target_os = "freebsd", target_os = "openbsd", target_os = "linux"))]
        /// The maximum size (in bytes) which a process may lock into memory
        /// using the mlock(2) system call.
        RLIMIT_MEMLOCK,

        #[cfg(any(target_os = "android", target_os = "linux"))]
        /// A limit on the number of bytes that can be allocated for POSIX
        /// message queues  for  the  real  user  ID  of  the  calling process.
        RLIMIT_MSGQUEUE,

        #[cfg(any(target_os = "android", target_os = "linux"))]
        /// A ceiling to which the process's nice value can be raised using
        /// setpriority or nice.
        RLIMIT_NICE,

        #[cfg(any(target_os = "android", target_os = "freebsd", target_os = "openbsd", target_os = "linux"))]
        /// The maximum number of simultaneous processes for this user id.
        RLIMIT_NPROC,

        #[cfg(target_os = "freebsd")]
        /// The maximum number of pseudo-terminals this user id is allowed to
        /// create.
        RLIMIT_NPTS,

        #[cfg(any(target_os = "android", target_os = "freebsd", target_os = "openbsd", target_os = "linux"))]
        /// When there is memory pressure and swap is available, prioritize
        /// eviction of a process' resident pages beyond this amount (in bytes).
        RLIMIT_RSS,

        #[cfg(any(target_os = "android", target_os = "linux"))]
        /// A ceiling on the real-time priority that may be set for this process
        /// using sched_setscheduler and  sched_set‐ param.
        RLIMIT_RTPRIO,

        #[cfg(any(target_os = "linux"))]
        /// A limit (in microseconds) on the amount of CPU time that a process
        /// scheduled under a real-time scheduling policy may con‐ sume without
        /// making a blocking system call.
        RLIMIT_RTTIME,

        #[cfg(any(target_os = "android", target_os = "linux"))]
        /// A limit on the number of signals that may be queued for the real
        /// user ID of the  calling  process.
        RLIMIT_SIGPENDING,

        #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
        /// The maximum size (in bytes) of socket buffer usage for this user.
        RLIMIT_SBSIZE,

        #[cfg(target_os = "freebsd")]
        /// The maximum size (in bytes) of the swap space that may be reserved
        /// or used by all of this user id's processes.
        RLIMIT_SWAP,

        #[cfg(target_os = "freebsd")]
        /// An alias for RLIMIT_AS.
        RLIMIT_VMEM,
    }
}

/// Get the current processes resource limits
///
/// A value of `None` indicates the value equals to `RLIM_INFINITY` which means
/// there is no limit.
///
/// # Parameters
///
/// * `resource`: The [`Resource`] that we want to get the limits of.
///
/// # Examples
///
/// ```
/// # use nix::sys::resource::{getrlimit, Resource};
///
/// let (soft_limit, hard_limit) = getrlimit(Resource::RLIMIT_NOFILE).unwrap();
/// println!("current soft_limit: {:?}", soft_limit);
/// println!("current hard_limit: {:?}", hard_limit);
/// ```
///
/// # References
///
/// [getrlimit(2)](https://pubs.opengroup.org/onlinepubs/9699919799/functions/getrlimit.html#tag_16_215)
///
/// [`Resource`]: enum.Resource.html
pub fn getrlimit(resource: Resource) -> Result<(Option<rlim_t>, Option<rlim_t>)> {
    let mut old_rlim = mem::MaybeUninit::<rlimit>::uninit();

    cfg_if! {
        if #[cfg(all(target_os = "linux", target_env = "gnu"))]{
            let res = unsafe { libc::getrlimit(resource as __rlimit_resource_t, old_rlim.as_mut_ptr()) };
        }else{
            let res = unsafe { libc::getrlimit(resource as c_int, old_rlim.as_mut_ptr()) };
        }
    }

    Errno::result(res).map(|_| {
        let rlimit { rlim_cur, rlim_max } = unsafe { old_rlim.assume_init() };
        (Some(rlim_cur), Some(rlim_max))
    })
}

/// Set the current processes resource limits
///
/// # Parameters
///
/// * `resource`: The [`Resource`] that we want to set the limits of.
/// * `soft_limit`: The value that the kernel enforces for the corresponding
///   resource. Note: `None` input will be replaced by constant `RLIM_INFINITY`.
/// * `hard_limit`: The ceiling for the soft limit. Must be lower or equal to
///   the current hard limit for non-root users. Note: `None` input will be
///   replaced by constant `RLIM_INFINITY`.
///
/// > Note: for some os (linux_gnu), setting hard_limit to `RLIM_INFINITY` can
/// > results `EPERM` Error. So you will need to set the number explicitly.
///
/// # Examples
///
/// ```
/// # use nix::sys::resource::{setrlimit, Resource};
///
/// let soft_limit = Some(512);
/// let hard_limit = Some(1024);
/// setrlimit(Resource::RLIMIT_NOFILE, soft_limit, hard_limit).unwrap();
/// ```
///
/// # References
///
/// [setrlimit(2)](https://pubs.opengroup.org/onlinepubs/9699919799/functions/getrlimit.html#tag_16_215)
///
/// [`Resource`]: enum.Resource.html
///
/// Note: `setrlimit` provides a safe wrapper to libc's `setrlimit`.
pub fn setrlimit(
    resource: Resource,
    soft_limit: Option<rlim_t>,
    hard_limit: Option<rlim_t>,
) -> Result<()> {
    let new_rlim = rlimit {
        rlim_cur: soft_limit.unwrap_or(RLIM_INFINITY),
        rlim_max: hard_limit.unwrap_or(RLIM_INFINITY),
    };
    cfg_if! {
        if #[cfg(all(target_os = "linux", target_env = "gnu"))]{
            let res = unsafe { libc::setrlimit(resource as __rlimit_resource_t, &new_rlim as *const rlimit) };
        }else{
            let res = unsafe { libc::setrlimit(resource as c_int, &new_rlim as *const rlimit) };
        }
    }

    Errno::result(res).map(drop)
}
