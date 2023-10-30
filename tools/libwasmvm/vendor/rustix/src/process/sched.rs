use crate::process::Pid;
use crate::{backend, io};

/// `CpuSet` represents a bit-mask of CPUs.
///
/// `CpuSet`s are used by [`sched_setaffinity`] and [`sched_getaffinity`], for
/// example.
///
/// # References
///  - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man3/CPU_SET.3.html
/// [`sched_setaffinity`]: crate::process::sched_setaffinity
/// [`sched_getaffinity`]: crate::process::sched_getaffinity
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CpuSet {
    cpu_set: backend::process::types::RawCpuSet,
}

impl CpuSet {
    /// The maximum number of CPU in `CpuSet`.
    pub const MAX_CPU: usize = backend::process::types::CPU_SETSIZE;

    /// Create a new and empty `CpuSet`.
    #[inline]
    pub fn new() -> Self {
        Self {
            cpu_set: backend::process::types::raw_cpu_set_new(),
        }
    }

    /// Test to see if a CPU is in the `CpuSet`.
    ///
    /// `field` is the CPU id to test.
    #[inline]
    pub fn is_set(&self, field: usize) -> bool {
        backend::process::cpu_set::CPU_ISSET(field, &self.cpu_set)
    }

    /// Add a CPU to `CpuSet`.
    ///
    /// `field` is the CPU id to add.
    #[inline]
    pub fn set(&mut self, field: usize) {
        backend::process::cpu_set::CPU_SET(field, &mut self.cpu_set)
    }

    /// Remove a CPU from `CpuSet`.
    ///
    /// `field` is the CPU id to remove.
    #[inline]
    pub fn unset(&mut self, field: usize) {
        backend::process::cpu_set::CPU_CLR(field, &mut self.cpu_set)
    }

    /// Count the number of CPUs set in the `CpuSet`.
    #[cfg(linux_kernel)]
    #[inline]
    pub fn count(&self) -> u32 {
        backend::process::cpu_set::CPU_COUNT(&self.cpu_set)
    }

    /// Zeroes the `CpuSet`.
    #[inline]
    pub fn clear(&mut self) {
        backend::process::cpu_set::CPU_ZERO(&mut self.cpu_set)
    }
}

impl Default for CpuSet {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// `sched_setaffinity(pid, cpuset)`—Set a thread's CPU affinity mask.
///
/// `pid` is the thread ID to update. If pid is `None`, then the current thread
/// is updated.
///
/// The `CpuSet` argument specifies the set of CPUs on which the thread will
/// be eligible to run.
///
/// # References
///  - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man2/sched_setaffinity.2.html
#[inline]
pub fn sched_setaffinity(pid: Option<Pid>, cpuset: &CpuSet) -> io::Result<()> {
    backend::process::syscalls::sched_setaffinity(pid, &cpuset.cpu_set)
}

/// `sched_getaffinity(pid)`—Get a thread's CPU affinity mask.
///
/// `pid` is the thread ID to check. If pid is `None`, then the current thread
/// is checked.
///
/// Returns the set of CPUs on which the thread is eligible to run.
///
/// # References
///  - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man2/sched_getaffinity.2.html
#[inline]
pub fn sched_getaffinity(pid: Option<Pid>) -> io::Result<CpuSet> {
    let mut cpuset = CpuSet::new();
    backend::process::syscalls::sched_getaffinity(pid, &mut cpuset.cpu_set).and(Ok(cpuset))
}
