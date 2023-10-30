//! Low-level implementation details for libc-like runtime libraries such as
//! [origin].
//!
//! These functions are for implementing thread-local storage (TLS), managing
//! threads, loaded libraries, and other process-wide resources. Most of
//! `rustix` doesn't care about what other libraries are linked into the
//! program or what they're doing, but the features in this module generally
//! can only be used by one entity within a process.
//!
//! The API for these functions is not stable, and this module is
//! `doc(hidden)`.
//!
//! [origin]: https://github.com/sunfishcode/mustang/tree/main/origin
//!
//! # Safety
//!
//! This module is intended to be used for implementing a runtime library such
//! as libc. Use of these features for any other purpose is likely to create
//! serious problems.
#![allow(unsafe_code)]

use crate::backend;
#[cfg(linux_raw)]
use crate::ffi::CStr;
#[cfg(linux_raw)]
use crate::fs::AtFlags;
#[cfg(linux_raw)]
use crate::io;
#[cfg(linux_raw)]
use crate::process::Pid;
#[cfg(linux_raw)]
use backend::fd::AsFd;
#[cfg(linux_raw)]
use core::ffi::c_void;

#[cfg(linux_raw)]
#[cfg(target_arch = "x86")]
#[inline]
pub unsafe fn set_thread_area(u_info: &mut UserDesc) -> io::Result<()> {
    backend::runtime::syscalls::tls::set_thread_area(u_info)
}

#[cfg(linux_raw)]
#[cfg(target_arch = "arm")]
#[inline]
pub unsafe fn arm_set_tls(data: *mut c_void) -> io::Result<()> {
    backend::runtime::syscalls::tls::arm_set_tls(data)
}

#[cfg(linux_raw)]
#[cfg(target_arch = "x86_64")]
#[inline]
pub unsafe fn set_fs(data: *mut c_void) {
    backend::runtime::syscalls::tls::set_fs(data)
}

#[cfg(linux_raw)]
#[inline]
pub unsafe fn set_tid_address(data: *mut c_void) -> Pid {
    backend::runtime::syscalls::tls::set_tid_address(data)
}

/// `prctl(PR_SET_NAME, name)`
///
/// # References
///  - [Linux]: https://man7.org/linux/man-pages/man2/prctl.2.html
///
/// # Safety
///
/// This is a very low-level feature for implementing threading libraries.
/// See the references links above.
///
/// [Linux]: https://man7.org/linux/man-pages/man2/prctl.2.html
#[cfg(linux_raw)]
#[inline]
pub unsafe fn set_thread_name(name: &CStr) -> io::Result<()> {
    backend::runtime::syscalls::tls::set_thread_name(name)
}

#[cfg(linux_raw)]
#[cfg(target_arch = "x86")]
pub use backend::runtime::tls::UserDesc;

/// `syscall(SYS_exit, status)`—Exit the current thread.
///
/// # Safety
///
/// This is a very low-level feature for implementing threading libraries.
#[cfg(linux_raw)]
#[inline]
pub unsafe fn exit_thread(status: i32) -> ! {
    backend::runtime::syscalls::tls::exit_thread(status)
}

/// Exit all the threads in the current process' thread group.
///
/// This is equivalent to `_exit` and `_Exit` in libc.
///
/// This does not all any `__cxa_atexit`, `atexit`, or any other destructors.
/// Most programs should use [`std::process::exit`] instead of calling this
/// directly.
///
/// # References
///  - [POSIX `_Exit`]
///  - [Linux `exit_group`]
///  - [Linux `_Exit`]
///
/// [POSIX `_Exit`]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/_Exit.html
/// [Linux `exit_group`]: https://man7.org/linux/man-pages/man2/exit_group.2.html
/// [Linux `_Exit`]: https://man7.org/linux/man-pages/man2/exit.2.html
#[doc(alias = "_exit")]
#[doc(alias = "_Exit")]
#[inline]
pub fn exit_group(status: i32) -> ! {
    backend::process::syscalls::exit_group(status)
}

/// Return fields from the main executable segment headers ("phdrs") relevant
/// to initializing TLS provided to the program at startup.
#[cfg(linux_raw)]
#[inline]
pub fn startup_tls_info() -> StartupTlsInfo {
    backend::runtime::tls::startup_tls_info()
}

/// `(getauxval(AT_PHDR), getauxval(AT_PHNUM))`—Returns the address and
/// number of ELF segment headers for the main executable.
///
/// # References
///  - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man3/getauxval.3.html
#[cfg(linux_raw)]
#[cfg(any(target_os = "android", target_os = "linux"))]
#[inline]
pub fn exe_phdrs() -> (*const c_void, usize) {
    backend::param::auxv::exe_phdrs()
}

#[cfg(linux_raw)]
pub use backend::runtime::tls::StartupTlsInfo;

/// `fork()`—Creates a new process by duplicating the calling process.
///
/// On success, the pid of the child process is returned in the parent, and
/// `None` is returned in the child.
///
/// Unlike its POSIX and libc counterparts, this `fork` does not invoke any
/// handlers (such as those registered with `pthread_atfork`).
///
/// The program environment in the child after a `fork` and before an `execve`
/// is very special. All code that executes in this environment must avoid:
///
///  - Acquiring any other locks that are held in other threads on the parent
///    at the time of the `fork`, as the child only contains one thread, and
///    attempting to acquire such locks will deadlock (though this is [not
///    considered unsafe]).
///
///  - Performing any dynamic allocation using the global allocator, since
///    global allocators may use locks to ensure thread safety, and their locks
///    may not be released in the child process, so attempts to allocate may
///    deadlock (as described in the previous point).
///
///  - Accessing any external state which the parent assumes it has exclusive
///    access to, such as a file protected by a file lock, as this could
///    corrupt the external state.
///
///  - Accessing any random-number-generator state inherited from the parent,
///    as the parent may have the same state and generate the same random
///    numbers, which may violate security invariants.
///
///  - Accessing any thread runtime state, since this function does not update
///    the thread id in the thread runtime, so thread runtime functions could
///    cause undefined behavior.
///
///  - Accessing any memory shared with the parent, such as a [`MAP_SHARED`]
///    mapping, even with anonymous or [`memfd_create`] mappings, as this could
///    cause undefined behavior.
///
///  - Calling any C function which isn't known to be [async-signal-safe], as
///    that could cause undefined behavior. The extent to which this also
///    applies to Rust functions is unclear at this time.
///
/// # Safety
///
/// The child must avoid accessing any memory shared with the parent in a
/// way that invokes undefined behavior. It must avoid accessing any threading
/// runtime functions in a way that invokes undefined behavior. And it must
/// avoid invoking any undefined behavior through any function that is not
/// guaranteed to be async-signal-safe.
///
/// # References
///  - [POSIX]
///  - [Linux]
///
/// # Literary interlude
///
/// > Do not jump on ancient uncles.
/// > Do not yell at average mice.
/// > Do not wear a broom to breakfast.
/// > Do not ask a snake’s advice.
/// > Do not bathe in chocolate pudding.
/// > Do not talk to bearded bears.
/// > Do not smoke cigars on sofas.
/// > Do not dance on velvet chairs.
/// > Do not take a whale to visit
/// > Russell’s mother’s cousin’s yacht.
/// > And whatever else you do do
/// > It is better you
/// > Do not.
///
/// - "Rules", by Karla Kuskin
///
/// [`MAP_SHARED`]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/mmap.html
/// [not considered unsafe]: https://doc.rust-lang.org/reference/behavior-not-considered-unsafe.html#deadlocks
/// [`memfd_create`]: https://man7.org/linux/man-pages/man2/memfd_create.2.html
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/fork.html
/// [Linux]: https://man7.org/linux/man-pages/man2/fork.2.html
/// [async-signal-safe]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/V2_chap02.html#tag_15_04_03
#[cfg(linux_raw)]
pub unsafe fn fork() -> io::Result<Option<Pid>> {
    backend::runtime::syscalls::fork()
}

/// `execveat(dirfd, path.as_c_str(), argv, envp, flags)`—Execute a new
/// command using the current process.
///
/// # Safety
///
/// The `argv` and `envp` pointers must point to NUL-terminated arrays, and
/// their contents must be pointers to NUL-terminated byte arrays.
///
/// # References
///  - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man2/execveat.2.html
#[cfg(linux_raw)]
#[inline]
pub unsafe fn execveat<Fd: AsFd>(
    dirfd: Fd,
    path: &CStr,
    argv: *const *const u8,
    envp: *const *const u8,
    flags: AtFlags,
) -> io::Errno {
    backend::runtime::syscalls::execveat(dirfd.as_fd(), path, argv, envp, flags)
}

/// `execve(path.as_c_str(), argv, envp)`—Execute a new command using the
/// current process.
///
/// # Safety
///
/// The `argv` and `envp` pointers must point to NUL-terminated arrays, and
/// their contents must be pointers to NUL-terminated byte arrays.
///
/// # References
///  - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man2/execve.2.html
#[cfg(linux_raw)]
#[inline]
pub unsafe fn execve(path: &CStr, argv: *const *const u8, envp: *const *const u8) -> io::Errno {
    backend::runtime::syscalls::execve(path, argv, envp)
}
