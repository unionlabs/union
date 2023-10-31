use crate::backend::c;
use bitflags::bitflags;

bitflags! {
    /// `PROT_*` flags for use with [`mmap`].
    ///
    /// For `PROT_NONE`, use `ProtFlags::empty()`.
    ///
    /// [`mmap`]: crate::io::mmap
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
    pub struct ProtFlags: u32 {
        /// `PROT_READ`
        const READ = bitcast!(c::PROT_READ);
        /// `PROT_WRITE`
        const WRITE = bitcast!(c::PROT_WRITE);
        /// `PROT_EXEC`
        const EXEC = bitcast!(c::PROT_EXEC);
    }
}

bitflags! {
    /// `PROT_*` flags for use with [`mprotect`].
    ///
    /// For `PROT_NONE`, use `MprotectFlags::empty()`.
    ///
    /// [`mprotect`]: crate::io::mprotect
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
    pub struct MprotectFlags: u32 {
        /// `PROT_READ`
        const READ = bitcast!(c::PROT_READ);
        /// `PROT_WRITE`
        const WRITE = bitcast!(c::PROT_WRITE);
        /// `PROT_EXEC`
        const EXEC = bitcast!(c::PROT_EXEC);
        /// `PROT_GROWSUP`
        #[cfg(linux_kernel)]
        const GROWSUP = bitcast!(c::PROT_GROWSUP);
        /// `PROT_GROWSDOWN`
        #[cfg(linux_kernel)]
        const GROWSDOWN = bitcast!(c::PROT_GROWSDOWN);
    }
}

bitflags! {
    /// `MAP_*` flags for use with [`mmap`].
    ///
    /// For `MAP_ANONYMOUS` (aka `MAP_ANON`), see [`mmap_anonymous`].
    ///
    /// [`mmap`]: crate::io::mmap
    /// [`mmap_anonymous`]: crates::io::mmap_anonymous
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
    pub struct MapFlags: u32 {
        /// `MAP_SHARED`
        const SHARED = bitcast!(c::MAP_SHARED);
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
        const SHARED_VALIDATE = bitcast!(c::MAP_SHARED_VALIDATE);
        /// `MAP_PRIVATE`
        const PRIVATE = bitcast!(c::MAP_PRIVATE);
        /// `MAP_DENYWRITE`
        #[cfg(not(any(
            bsd,
            solarish,
            target_os = "haiku",
            target_os = "redox",
        )))]
        const DENYWRITE = bitcast!(c::MAP_DENYWRITE);
        /// `MAP_FIXED`
        const FIXED = bitcast!(c::MAP_FIXED);
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
        const FIXED_NOREPLACE = bitcast!(c::MAP_FIXED_NOREPLACE);
        /// `MAP_GROWSDOWN`
        #[cfg(not(any(
            bsd,
            solarish,
            target_os = "haiku",
            target_os = "redox",
        )))]
        const GROWSDOWN = bitcast!(c::MAP_GROWSDOWN);
        /// `MAP_HUGETLB`
        #[cfg(not(any(
            bsd,
            solarish,
            target_os = "haiku",
            target_os = "redox",
        )))]
        const HUGETLB = bitcast!(c::MAP_HUGETLB);
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
        const HUGE_2MB = bitcast!(c::MAP_HUGE_2MB);
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
        const HUGE_1GB = bitcast!(c::MAP_HUGE_1GB);
        /// `MAP_LOCKED`
        #[cfg(not(any(
            bsd,
            solarish,
            target_os = "haiku",
            target_os = "redox",
        )))]
        const LOCKED = bitcast!(c::MAP_LOCKED);
        /// `MAP_NOCORE`
        #[cfg(freebsdlike)]
        const NOCORE = bitcast!(c::MAP_NOCORE);
        /// `MAP_NORESERVE`
        #[cfg(not(any(freebsdlike, target_os = "redox")))]
        const NORESERVE = bitcast!(c::MAP_NORESERVE);
        /// `MAP_NOSYNC`
        #[cfg(freebsdlike)]
        const NOSYNC = bitcast!(c::MAP_NOSYNC);
        /// `MAP_POPULATE`
        #[cfg(not(any(
            bsd,
            solarish,
            target_os = "haiku",
            target_os = "redox",
        )))]
        const POPULATE = bitcast!(c::MAP_POPULATE);
        /// `MAP_STACK`
        #[cfg(not(any(
            apple,
            solarish,
            target_os = "dragonfly",
            target_os = "haiku",
            target_os = "netbsd",
            target_os = "redox",
        )))]
        const STACK = bitcast!(c::MAP_STACK);
        /// `MAP_PREFAULT_READ`
        #[cfg(target_os = "freebsd")]
        const PREFAULT_READ = bitcast!(c::MAP_PREFAULT_READ);
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
        const SYNC = bitcast!(c::MAP_SYNC);
        /// `MAP_UNINITIALIZED`
        #[cfg(any())]
        const UNINITIALIZED = bitcast!(c::MAP_UNINITIALIZED);
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
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
    pub struct MremapFlags: u32 {
        /// `MREMAP_MAYMOVE`
        const MAYMOVE = bitcast!(c::MREMAP_MAYMOVE);
    }
}

bitflags! {
    /// `MS_*` flags for use with [`msync`].
    ///
    /// [`msync`]: crate::io::msync
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
    pub struct MsyncFlags: u32 {
        /// `MS_SYNC`—Requests an update and waits for it to complete.
        const SYNC = bitcast!(c::MS_SYNC);
        /// `MS_ASYNC`—Specifies that an update be scheduled, but the call
        /// returns immediately.
        const ASYNC = bitcast!(c::MS_ASYNC);
        /// `MS_INVALIDATE`—Asks to invalidate other mappings of the same
        /// file (so that they can be updated with the fresh values just
        /// written).
        const INVALIDATE = bitcast!(c::MS_INVALIDATE);
    }
}

#[cfg(linux_kernel)]
bitflags! {
    /// `MLOCK_*` flags for use with [`mlock_with`].
    ///
    /// [`mlock_with`]: crate::io::mlock_with
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
    pub struct MlockFlags: u32 {
        /// `MLOCK_ONFAULT`
        const ONFAULT = bitcast!(c::MLOCK_ONFAULT);
    }
}

/// `POSIX_MADV_*` constants for use with [`madvise`].
///
/// [`madvise`]: crate::mm::madvise
#[cfg(not(target_os = "redox"))]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u32)]
#[non_exhaustive]
pub enum Advice {
    /// `POSIX_MADV_NORMAL`
    #[cfg(not(any(target_os = "android", target_os = "haiku")))]
    Normal = bitcast!(c::POSIX_MADV_NORMAL),

    /// `POSIX_MADV_NORMAL`
    #[cfg(any(target_os = "android", target_os = "haiku"))]
    Normal = bitcast!(c::MADV_NORMAL),

    /// `POSIX_MADV_SEQUENTIAL`
    #[cfg(not(any(target_os = "android", target_os = "haiku")))]
    Sequential = bitcast!(c::POSIX_MADV_SEQUENTIAL),

    /// `POSIX_MADV_SEQUENTIAL`
    #[cfg(any(target_os = "android", target_os = "haiku"))]
    Sequential = bitcast!(c::MADV_SEQUENTIAL),

    /// `POSIX_MADV_RANDOM`
    #[cfg(not(any(target_os = "android", target_os = "haiku")))]
    Random = bitcast!(c::POSIX_MADV_RANDOM),

    /// `POSIX_MADV_RANDOM`
    #[cfg(any(target_os = "android", target_os = "haiku"))]
    Random = bitcast!(c::MADV_RANDOM),

    /// `POSIX_MADV_WILLNEED`
    #[cfg(not(any(target_os = "android", target_os = "haiku")))]
    WillNeed = bitcast!(c::POSIX_MADV_WILLNEED),

    /// `POSIX_MADV_WILLNEED`
    #[cfg(any(target_os = "android", target_os = "haiku"))]
    WillNeed = bitcast!(c::MADV_WILLNEED),

    /// `POSIX_MADV_DONTNEED`
    #[cfg(not(any(target_os = "android", target_os = "emscripten", target_os = "haiku")))]
    DontNeed = bitcast!(c::POSIX_MADV_DONTNEED),

    /// `POSIX_MADV_DONTNEED`
    #[cfg(any(target_os = "android", target_os = "haiku"))]
    DontNeed = bitcast!(i32::MAX - 1),

    /// `MADV_DONTNEED`
    // `MADV_DONTNEED` has the same value as `POSIX_MADV_DONTNEED`. We don't
    // have a separate `posix_madvise` from `madvise`, so we expose a special
    // value which we special-case.
    #[cfg(target_os = "linux")]
    LinuxDontNeed = bitcast!(i32::MAX),

    /// `MADV_DONTNEED`
    #[cfg(target_os = "android")]
    LinuxDontNeed = bitcast!(c::MADV_DONTNEED),
    /// `MADV_FREE`
    #[cfg(linux_kernel)]
    LinuxFree = bitcast!(c::MADV_FREE),
    /// `MADV_REMOVE`
    #[cfg(linux_kernel)]
    LinuxRemove = bitcast!(c::MADV_REMOVE),
    /// `MADV_DONTFORK`
    #[cfg(linux_kernel)]
    LinuxDontFork = bitcast!(c::MADV_DONTFORK),
    /// `MADV_DOFORK`
    #[cfg(linux_kernel)]
    LinuxDoFork = bitcast!(c::MADV_DOFORK),
    /// `MADV_HWPOISON`
    #[cfg(linux_kernel)]
    LinuxHwPoison = bitcast!(c::MADV_HWPOISON),
    /// `MADV_SOFT_OFFLINE`
    #[cfg(all(linux_kernel, not(any(target_arch = "mips", target_arch = "mips64"))))]
    LinuxSoftOffline = bitcast!(c::MADV_SOFT_OFFLINE),
    /// `MADV_MERGEABLE`
    #[cfg(linux_kernel)]
    LinuxMergeable = bitcast!(c::MADV_MERGEABLE),
    /// `MADV_UNMERGEABLE`
    #[cfg(linux_kernel)]
    LinuxUnmergeable = bitcast!(c::MADV_UNMERGEABLE),
    /// `MADV_HUGEPAGE` (since Linux 2.6.38)
    #[cfg(linux_kernel)]
    LinuxHugepage = bitcast!(c::MADV_HUGEPAGE),
    /// `MADV_NOHUGEPAGE` (since Linux 2.6.38)
    #[cfg(linux_kernel)]
    LinuxNoHugepage = bitcast!(c::MADV_NOHUGEPAGE),
    /// `MADV_DONTDUMP` (since Linux 3.4)
    #[cfg(linux_kernel)]
    LinuxDontDump = bitcast!(c::MADV_DONTDUMP),
    /// `MADV_DODUMP` (since Linux 3.4)
    #[cfg(linux_kernel)]
    LinuxDoDump = bitcast!(c::MADV_DODUMP),
    /// `MADV_WIPEONFORK` (since Linux 4.14)
    #[cfg(linux_kernel)]
    LinuxWipeOnFork = bitcast!(c::MADV_WIPEONFORK),
    /// `MADV_KEEPONFORK` (since Linux 4.14)
    #[cfg(linux_kernel)]
    LinuxKeepOnFork = bitcast!(c::MADV_KEEPONFORK),
    /// `MADV_COLD` (since Linux 5.4)
    #[cfg(linux_kernel)]
    LinuxCold = bitcast!(c::MADV_COLD),
    /// `MADV_PAGEOUT` (since Linux 5.4)
    #[cfg(linux_kernel)]
    LinuxPageOut = bitcast!(c::MADV_PAGEOUT),
    /// `MADV_POPULATE_READ` (since Linux 5.14)
    #[cfg(linux_kernel)]
    LinuxPopulateRead = bitcast!(c::MADV_POPULATE_READ),
    /// `MADV_POPULATE_WRITE` (since Linux 5.14)
    #[cfg(linux_kernel)]
    LinuxPopulateWrite = bitcast!(c::MADV_POPULATE_WRITE),
    /// `MADV_DONTNEED_LOCKED` (since Linux 5.18)
    #[cfg(linux_kernel)]
    LinuxDontneedLocked = bitcast!(c::MADV_DONTNEED_LOCKED),
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
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
    pub struct UserfaultfdFlags: u32 {
        /// `O_CLOEXEC`
        const CLOEXEC = bitcast!(c::O_CLOEXEC);
        /// `O_NONBLOCK`
        const NONBLOCK = bitcast!(c::O_NONBLOCK);
    }
}
