use super::super::c;
use bitflags::bitflags;

bitflags! {
    /// `PROT_*` flags for use with [`mmap`].
    ///
    /// For `PROT_NONE`, use `ProtFlags::empty()`.
    ///
    /// [`mmap`]: crate::io::mmap
    pub struct ProtFlags: c::c_int {
        /// `PROT_READ`
        const READ = c::PROT_READ;
        /// `PROT_WRITE`
        const WRITE = c::PROT_WRITE;
        /// `PROT_EXEC`
        const EXEC = c::PROT_EXEC;
    }
}

bitflags! {
    /// `PROT_*` flags for use with [`mprotect`].
    ///
    /// For `PROT_NONE`, use `MprotectFlags::empty()`.
    ///
    /// [`mprotect`]: crate::io::mprotect
    pub struct MprotectFlags: c::c_int {
        /// `PROT_READ`
        const READ = c::PROT_READ;
        /// `PROT_WRITE`
        const WRITE = c::PROT_WRITE;
        /// `PROT_EXEC`
        const EXEC = c::PROT_EXEC;
        /// `PROT_GROWSUP`
        #[cfg(linux_kernel)]
        const GROWSUP = c::PROT_GROWSUP;
        /// `PROT_GROWSDOWN`
        #[cfg(linux_kernel)]
        const GROWSDOWN = c::PROT_GROWSDOWN;
    }
}

bitflags! {
    /// `MAP_*` flags for use with [`mmap`].
    ///
    /// For `MAP_ANONYMOUS` (aka `MAP_ANON`), see [`mmap_anonymous`].
    ///
    /// [`mmap`]: crate::io::mmap
    /// [`mmap_anonymous`]: crates::io::mmap_anonymous
    pub struct MapFlags: c::c_int {
        /// `MAP_SHARED`
        const SHARED = c::MAP_SHARED;
        /// `MAP_SHARED_VALIDATE`
        #[cfg(not(any(
            bsd,
            solarish,
            target_os = "android",
            target_os = "emscripten",
            target_os = "fuchsia",
            target_os = "haiku",
            target_os = "redox",
        )))]
        const SHARED_VALIDATE = c::MAP_SHARED_VALIDATE;
        /// `MAP_PRIVATE`
        const PRIVATE = c::MAP_PRIVATE;
        /// `MAP_DENYWRITE`
        #[cfg(not(any(
            bsd,
            solarish,
            target_os = "haiku",
            target_os = "redox",
        )))]
        const DENYWRITE = c::MAP_DENYWRITE;
        /// `MAP_FIXED`
        const FIXED = c::MAP_FIXED;
        /// `MAP_FIXED_NOREPLACE`
        #[cfg(not(any(
            bsd,
            solarish,
            target_os = "android",
            target_os = "emscripten",
            target_os = "fuchsia",
            target_os = "haiku",
            target_os = "redox",
        )))]
        const FIXED_NOREPLACE = c::MAP_FIXED_NOREPLACE;
        /// `MAP_GROWSDOWN`
        #[cfg(not(any(
            bsd,
            solarish,
            target_os = "haiku",
            target_os = "redox",
        )))]
        const GROWSDOWN = c::MAP_GROWSDOWN;
        /// `MAP_HUGETLB`
        #[cfg(not(any(
            bsd,
            solarish,
            target_os = "haiku",
            target_os = "redox",
        )))]
        const HUGETLB = c::MAP_HUGETLB;
        /// `MAP_HUGE_2MB`
        #[cfg(not(any(
            bsd,
            solarish,
            target_os = "android",
            target_os = "emscripten",
            target_os = "fuchsia",
            target_os = "haiku",
            target_os = "redox",
        )))]
        const HUGE_2MB = c::MAP_HUGE_2MB;
        /// `MAP_HUGE_1GB`
        #[cfg(not(any(
            bsd,
            solarish,
            target_os = "android",
            target_os = "emscripten",
            target_os = "fuchsia",
            target_os = "haiku",
            target_os = "redox",
        )))]
        const HUGE_1GB = c::MAP_HUGE_1GB;
        /// `MAP_LOCKED`
        #[cfg(not(any(
            bsd,
            solarish,
            target_os = "haiku",
            target_os = "redox",
        )))]
        const LOCKED = c::MAP_LOCKED;
        /// `MAP_NOCORE`
        #[cfg(freebsdlike)]
        const NOCORE = c::MAP_NOCORE;
        /// `MAP_NORESERVE`
        #[cfg(not(any(freebsdlike, target_os = "redox")))]
        const NORESERVE = c::MAP_NORESERVE;
        /// `MAP_NOSYNC`
        #[cfg(freebsdlike)]
        const NOSYNC = c::MAP_NOSYNC;
        /// `MAP_POPULATE`
        #[cfg(not(any(
            bsd,
            solarish,
            target_os = "haiku",
            target_os = "redox",
        )))]
        const POPULATE = c::MAP_POPULATE;
        /// `MAP_STACK`
        #[cfg(not(any(
            apple,
            solarish,
            target_os = "dragonfly",
            target_os = "haiku",
            target_os = "netbsd",
            target_os = "redox",
        )))]
        const STACK = c::MAP_STACK;
        /// `MAP_PREFAULT_READ`
        #[cfg(target_os = "freebsd")]
        const PREFAULT_READ = c::MAP_PREFAULT_READ;
        /// `MAP_SYNC`
        #[cfg(not(any(
            bsd,
            solarish,
            target_os = "android",
            target_os = "emscripten",
            target_os = "fuchsia",
            target_os = "haiku",
            target_os = "redox",
            all(
                linux_kernel,
                any(target_arch = "mips", target_arch = "mips64"),
            )
        )))]
        const SYNC = c::MAP_SYNC;
        /// `MAP_UNINITIALIZED`
        #[cfg(any())]
        const UNINITIALIZED = c::MAP_UNINITIALIZED;
    }
}

#[cfg(any(target_os = "emscripten", target_os = "linux"))]
bitflags! {
    /// `MREMAP_*` flags for use with [`mremap`].
    ///
    /// For `MREMAP_FIXED`, see [`mremap_fixed`].
    ///
    /// [`mremap`]: crate::io::mremap
    /// [`mremap_fixed`]: crate::io::mremap_fixed
    pub struct MremapFlags: i32 {
        /// `MREMAP_MAYMOVE`
        const MAYMOVE = c::MREMAP_MAYMOVE;
    }
}

bitflags! {
    /// `MS_*` flags for use with [`msync`].
    ///
    /// [`msync`]: crate::io::msync
    pub struct MsyncFlags: i32 {
        /// `MS_SYNC`—Requests an update and waits for it to complete.
        const SYNC = c::MS_SYNC;
        /// `MS_ASYNC`—Specifies that an update be scheduled, but the call
        /// returns immediately.
        const ASYNC = c::MS_ASYNC;
        /// `MS_INVALIDATE`—Asks to invalidate other mappings of the same
        /// file (so that they can be updated with the fresh values just
        /// written).
        const INVALIDATE = c::MS_INVALIDATE;
    }
}

#[cfg(linux_kernel)]
bitflags! {
    /// `MLOCK_*` flags for use with [`mlock_with`].
    ///
    /// [`mlock_with`]: crate::io::mlock_with
    pub struct MlockFlags: i32 {
        /// `MLOCK_ONFAULT`
        const ONFAULT = c::MLOCK_ONFAULT as _;
    }
}

/// `POSIX_MADV_*` constants for use with [`madvise`].
///
/// [`madvise`]: crate::mm::madvise
#[cfg(not(target_os = "redox"))]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(i32)]
#[non_exhaustive]
pub enum Advice {
    /// `POSIX_MADV_NORMAL`
    #[cfg(not(any(target_os = "android", target_os = "haiku")))]
    Normal = c::POSIX_MADV_NORMAL,

    /// `POSIX_MADV_NORMAL`
    #[cfg(any(target_os = "android", target_os = "haiku"))]
    Normal = c::MADV_NORMAL,

    /// `POSIX_MADV_SEQUENTIAL`
    #[cfg(not(any(target_os = "android", target_os = "haiku")))]
    Sequential = c::POSIX_MADV_SEQUENTIAL,

    /// `POSIX_MADV_SEQUENTIAL`
    #[cfg(any(target_os = "android", target_os = "haiku"))]
    Sequential = c::MADV_SEQUENTIAL,

    /// `POSIX_MADV_RANDOM`
    #[cfg(not(any(target_os = "android", target_os = "haiku")))]
    Random = c::POSIX_MADV_RANDOM,

    /// `POSIX_MADV_RANDOM`
    #[cfg(any(target_os = "android", target_os = "haiku"))]
    Random = c::MADV_RANDOM,

    /// `POSIX_MADV_WILLNEED`
    #[cfg(not(any(target_os = "android", target_os = "haiku")))]
    WillNeed = c::POSIX_MADV_WILLNEED,

    /// `POSIX_MADV_WILLNEED`
    #[cfg(any(target_os = "android", target_os = "haiku"))]
    WillNeed = c::MADV_WILLNEED,

    /// `POSIX_MADV_DONTNEED`
    #[cfg(not(any(target_os = "android", target_os = "emscripten", target_os = "haiku")))]
    DontNeed = c::POSIX_MADV_DONTNEED,

    /// `POSIX_MADV_DONTNEED`
    #[cfg(any(target_os = "android", target_os = "haiku"))]
    DontNeed = i32::MAX - 1,

    /// `MADV_DONTNEED`
    // `MADV_DONTNEED` has the same value as `POSIX_MADV_DONTNEED`. We don't
    // have a separate `posix_madvise` from `madvise`, so we expose a special
    // value which we special-case.
    #[cfg(target_os = "linux")]
    LinuxDontNeed = i32::MAX,

    /// `MADV_DONTNEED`
    #[cfg(target_os = "android")]
    LinuxDontNeed = c::MADV_DONTNEED,
    /// `MADV_FREE`
    #[cfg(linux_kernel)]
    LinuxFree = c::MADV_FREE,
    /// `MADV_REMOVE`
    #[cfg(linux_kernel)]
    LinuxRemove = c::MADV_REMOVE,
    /// `MADV_DONTFORK`
    #[cfg(linux_kernel)]
    LinuxDontFork = c::MADV_DONTFORK,
    /// `MADV_DOFORK`
    #[cfg(linux_kernel)]
    LinuxDoFork = c::MADV_DOFORK,
    /// `MADV_HWPOISON`
    #[cfg(linux_kernel)]
    LinuxHwPoison = c::MADV_HWPOISON,
    /// `MADV_SOFT_OFFLINE`
    #[cfg(all(linux_kernel, not(any(target_arch = "mips", target_arch = "mips64"))))]
    LinuxSoftOffline = c::MADV_SOFT_OFFLINE,
    /// `MADV_MERGEABLE`
    #[cfg(linux_kernel)]
    LinuxMergeable = c::MADV_MERGEABLE,
    /// `MADV_UNMERGEABLE`
    #[cfg(linux_kernel)]
    LinuxUnmergeable = c::MADV_UNMERGEABLE,
    /// `MADV_HUGEPAGE` (since Linux 2.6.38)
    #[cfg(linux_kernel)]
    LinuxHugepage = c::MADV_HUGEPAGE,
    /// `MADV_NOHUGEPAGE` (since Linux 2.6.38)
    #[cfg(linux_kernel)]
    LinuxNoHugepage = c::MADV_NOHUGEPAGE,
    /// `MADV_DONTDUMP` (since Linux 3.4)
    #[cfg(linux_kernel)]
    LinuxDontDump = c::MADV_DONTDUMP,
    /// `MADV_DODUMP` (since Linux 3.4)
    #[cfg(linux_kernel)]
    LinuxDoDump = c::MADV_DODUMP,
    /// `MADV_WIPEONFORK` (since Linux 4.14)
    #[cfg(linux_kernel)]
    LinuxWipeOnFork = libc::MADV_WIPEONFORK as i32,
    /// `MADV_KEEPONFORK` (since Linux 4.14)
    #[cfg(linux_kernel)]
    LinuxKeepOnFork = libc::MADV_KEEPONFORK as i32,
    /// `MADV_COLD` (since Linux 5.4)
    #[cfg(linux_kernel)]
    LinuxCold = libc::MADV_COLD as i32,
    /// `MADV_PAGEOUT` (since Linux 5.4)
    #[cfg(linux_kernel)]
    LinuxPageOut = libc::MADV_PAGEOUT as i32,
    /// `MADV_POPULATE_READ` (since Linux 5.14)
    #[cfg(linux_kernel)]
    LinuxPopulateRead = libc::MADV_POPULATE_READ as i32,
    /// `MADV_POPULATE_WRITE` (since Linux 5.14)
    #[cfg(linux_kernel)]
    LinuxPopulateWrite = libc::MADV_POPULATE_WRITE as i32,
    /// `MADV_DONTNEED_LOCKED` (since Linux 5.18)
    #[cfg(linux_kernel)]
    LinuxDontneedLocked = libc::MADV_DONTNEED_LOCKED as i32,
}

#[cfg(target_os = "emscripten")]
#[allow(non_upper_case_globals)]
impl Advice {
    /// `POSIX_MADV_DONTNEED`
    pub const DontNeed: Self = Self::Normal;
}

#[cfg(linux_kernel)]
bitflags! {
    /// `O_*` flags for use with [`userfaultfd`].
    ///
    /// [`userfaultfd`]: crate::io::userfaultfd
    pub struct UserfaultfdFlags: c::c_int {
        /// `O_CLOEXEC`
        const CLOEXEC = c::O_CLOEXEC;
        /// `O_NONBLOCK`
        const NONBLOCK = c::O_NONBLOCK;
    }
}
