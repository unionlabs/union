//! Memory management declarations.

use crate::Result;
#[cfg(not(target_os = "android"))]
use crate::NixPath;
use crate::errno::Errno;
#[cfg(not(target_os = "android"))]
use crate::fcntl::OFlag;
use libc::{self, c_int, c_void, size_t, off_t};
#[cfg(not(target_os = "android"))]
use crate::sys::stat::Mode;
use std::os::unix::io::RawFd;

libc_bitflags!{
    /// Desired memory protection of a memory mapping.
    pub struct ProtFlags: c_int {
        /// Pages cannot be accessed.
        PROT_NONE;
        /// Pages can be read.
        PROT_READ;
        /// Pages can be written.
        PROT_WRITE;
        /// Pages can be executed
        PROT_EXEC;
        /// Apply protection up to the end of a mapping that grows upwards.
        #[cfg(any(target_os = "android", target_os = "linux"))]
        PROT_GROWSDOWN;
        /// Apply protection down to the beginning of a mapping that grows downwards.
        #[cfg(any(target_os = "android", target_os = "linux"))]
        PROT_GROWSUP;
    }
}

libc_bitflags!{
    /// Additional parameters for [`mmap`].
    pub struct MapFlags: c_int {
        /// Compatibility flag. Ignored.
        MAP_FILE;
        /// Share this mapping. Mutually exclusive with `MAP_PRIVATE`.
        MAP_SHARED;
        /// Create a private copy-on-write mapping. Mutually exclusive with `MAP_SHARED`.
        MAP_PRIVATE;
        /// Place the mapping at exactly the address specified in `addr`.
        MAP_FIXED;
        /// To be used with `MAP_FIXED`, to forbid the system
        /// to select a different address than the one specified.
        #[cfg(target_os = "freebsd")]
        MAP_EXCL;
        /// Synonym for `MAP_ANONYMOUS`.
        MAP_ANON;
        /// The mapping is not backed by any file.
        MAP_ANONYMOUS;
        /// Put the mapping into the first 2GB of the process address space.
        #[cfg(any(all(any(target_os = "android", target_os = "linux"),
                      any(target_arch = "x86", target_arch = "x86_64")),
                  all(target_os = "linux", target_env = "musl", any(target_arch = "x86", target_arch = "x86_64")),
                  all(target_os = "freebsd", target_pointer_width = "64")))]
        MAP_32BIT;
        /// Used for stacks; indicates to the kernel that the mapping should extend downward in memory.
        #[cfg(any(target_os = "android", target_os = "linux"))]
        MAP_GROWSDOWN;
        /// Compatibility flag. Ignored.
        #[cfg(any(target_os = "android", target_os = "linux"))]
        MAP_DENYWRITE;
        /// Compatibility flag. Ignored.
        #[cfg(any(target_os = "android", target_os = "linux"))]
        MAP_EXECUTABLE;
        /// Mark the mmaped region to be locked in the same way as `mlock(2)`.
        #[cfg(any(target_os = "android", target_os = "linux"))]
        MAP_LOCKED;
        /// Do not reserve swap space for this mapping.
        ///
        /// This was removed in FreeBSD 11 and is unused in DragonFlyBSD.
        #[cfg(not(any(target_os = "dragonfly", target_os = "freebsd")))]
        MAP_NORESERVE;
        /// Populate page tables for a mapping.
        #[cfg(any(target_os = "android", target_os = "linux"))]
        MAP_POPULATE;
        /// Only meaningful when used with `MAP_POPULATE`. Don't perform read-ahead.
        #[cfg(any(target_os = "android", target_os = "linux"))]
        MAP_NONBLOCK;
        /// Allocate the mapping using "huge pages."
        #[cfg(any(target_os = "android", target_os = "linux"))]
        MAP_HUGETLB;
        /// Make use of 64KB huge page (must be supported by the system)
        #[cfg(target_os = "linux")]
        MAP_HUGE_64KB;
        /// Make use of 512KB huge page (must be supported by the system)
        #[cfg(target_os = "linux")]
        MAP_HUGE_512KB;
        /// Make use of 1MB huge page (must be supported by the system)
        #[cfg(target_os = "linux")]
        MAP_HUGE_1MB;
        /// Make use of 2MB huge page (must be supported by the system)
        #[cfg(target_os = "linux")]
        MAP_HUGE_2MB;
        /// Make use of 8MB huge page (must be supported by the system)
        #[cfg(target_os = "linux")]
        MAP_HUGE_8MB;
        /// Make use of 16MB huge page (must be supported by the system)
        #[cfg(target_os = "linux")]
        MAP_HUGE_16MB;
        /// Make use of 32MB huge page (must be supported by the system)
        #[cfg(target_os = "linux")]
        MAP_HUGE_32MB;
        /// Make use of 256MB huge page (must be supported by the system)
        #[cfg(target_os = "linux")]
        MAP_HUGE_256MB;
        /// Make use of 512MB huge page (must be supported by the system)
        #[cfg(target_os = "linux")]
        MAP_HUGE_512MB;
        /// Make use of 1GB huge page (must be supported by the system)
        #[cfg(target_os = "linux")]
        MAP_HUGE_1GB;
        /// Make use of 2GB huge page (must be supported by the system)
        #[cfg(target_os = "linux")]
        MAP_HUGE_2GB;
        /// Make use of 16GB huge page (must be supported by the system)
        #[cfg(target_os = "linux")]
        MAP_HUGE_16GB;

        /// Lock the mapped region into memory as with `mlock(2)`.
        #[cfg(target_os = "netbsd")]
        MAP_WIRED;
        /// Causes dirtied data in the specified range to be flushed to disk only when necessary.
        #[cfg(any(target_os = "dragonfly", target_os = "freebsd"))]
        MAP_NOSYNC;
        /// Rename private pages to a file.
        ///
        /// This was removed in FreeBSD 11 and is unused in DragonFlyBSD.
        #[cfg(any(target_os = "netbsd", target_os = "openbsd"))]
        MAP_RENAME;
        /// Region may contain semaphores.
        #[cfg(any(target_os = "dragonfly", target_os = "freebsd", target_os = "netbsd", target_os = "openbsd"))]
        MAP_HASSEMAPHORE;
        /// Region grows down, like a stack.
        #[cfg(any(target_os = "android", target_os = "dragonfly", target_os = "freebsd", target_os = "linux", target_os = "openbsd"))]
        MAP_STACK;
        /// Pages in this mapping are not retained in the kernel's memory cache.
        #[cfg(any(target_os = "ios", target_os = "macos"))]
        MAP_NOCACHE;
        /// Allows the W/X bit on the page, it's necessary on aarch64 architecture.
        #[cfg(any(target_os = "ios", target_os = "macos"))]
        MAP_JIT;
        /// Allows to use large pages, underlying alignment based on size.
        #[cfg(target_os = "freebsd")]
        MAP_ALIGNED_SUPER;
        /// Pages will be discarded in the core dumps.
        #[cfg(target_os = "openbsd")]
        MAP_CONCEAL;
    }
}

#[cfg(any(target_os = "linux", target_os = "netbsd"))]
libc_bitflags!{
    /// Options for [`mremap`].
    pub struct MRemapFlags: c_int {
        /// Permit the kernel to relocate the mapping to a new virtual address, if necessary.
        #[cfg(target_os = "linux")]
        MREMAP_MAYMOVE;
        /// Place the mapping at exactly the address specified in `new_address`.
        #[cfg(target_os = "linux")]
        MREMAP_FIXED;
        /// Permits to use the old and new address as hints to relocate the mapping.
        #[cfg(target_os = "netbsd")]
        MAP_FIXED;
        /// Allows to duplicate the mapping to be able to apply different flags on the copy.
        #[cfg(target_os = "netbsd")]
        MAP_REMAPDUP;
    }
}

libc_enum!{
    /// Usage information for a range of memory to allow for performance optimizations by the kernel.
    ///
    /// Used by [`madvise`].
    #[repr(i32)]
    #[non_exhaustive]
    pub enum MmapAdvise {
        /// No further special treatment. This is the default.
        MADV_NORMAL,
        /// Expect random page references.
        MADV_RANDOM,
        /// Expect sequential page references.
        MADV_SEQUENTIAL,
        /// Expect access in the near future.
        MADV_WILLNEED,
        /// Do not expect access in the near future.
        MADV_DONTNEED,
        /// Free up a given range of pages and its associated backing store.
        #[cfg(any(target_os = "android", target_os = "linux"))]
        MADV_REMOVE,
        /// Do not make pages in this range available to the child after a `fork(2)`.
        #[cfg(any(target_os = "android", target_os = "linux"))]
        MADV_DONTFORK,
        /// Undo the effect of `MADV_DONTFORK`.
        #[cfg(any(target_os = "android", target_os = "linux"))]
        MADV_DOFORK,
        /// Poison the given pages.
        ///
        /// Subsequent references to those pages are treated like hardware memory corruption.
        #[cfg(any(target_os = "android", target_os = "linux"))]
        MADV_HWPOISON,
        /// Enable Kernel Samepage Merging (KSM) for the given pages.
        #[cfg(any(target_os = "android", target_os = "linux"))]
        MADV_MERGEABLE,
        /// Undo the effect of `MADV_MERGEABLE`
        #[cfg(any(target_os = "android", target_os = "linux"))]
        MADV_UNMERGEABLE,
        /// Preserve the memory of each page but offline the original page.
        #[cfg(any(target_os = "android",
            all(target_os = "linux", any(
                target_arch = "aarch64",
                target_arch = "arm",
                target_arch = "powerpc",
                target_arch = "powerpc64",
                target_arch = "s390x",
                target_arch = "x86",
                target_arch = "x86_64",
                target_arch = "sparc64"))))]
        MADV_SOFT_OFFLINE,
        /// Enable Transparent Huge Pages (THP) for pages in the given range.
        #[cfg(any(target_os = "android", target_os = "linux"))]
        MADV_HUGEPAGE,
        /// Undo the effect of `MADV_HUGEPAGE`.
        #[cfg(any(target_os = "android", target_os = "linux"))]
        MADV_NOHUGEPAGE,
        /// Exclude the given range from a core dump.
        #[cfg(any(target_os = "android", target_os = "linux"))]
        MADV_DONTDUMP,
        /// Undo the effect of an earlier `MADV_DONTDUMP`.
        #[cfg(any(target_os = "android", target_os = "linux"))]
        MADV_DODUMP,
        /// Specify that the application no longer needs the pages in the given range.
        MADV_FREE,
        /// Request that the system not flush the current range to disk unless it needs to.
        #[cfg(any(target_os = "dragonfly", target_os = "freebsd"))]
        MADV_NOSYNC,
        /// Undoes the effects of `MADV_NOSYNC` for any future pages dirtied within the given range.
        #[cfg(any(target_os = "dragonfly", target_os = "freebsd"))]
        MADV_AUTOSYNC,
        /// Region is not included in a core file.
        #[cfg(any(target_os = "dragonfly", target_os = "freebsd"))]
        MADV_NOCORE,
        /// Include region in a core file
        #[cfg(any(target_os = "dragonfly", target_os = "freebsd"))]
        MADV_CORE,
        #[cfg(any(target_os = "freebsd"))]
        MADV_PROTECT,
        /// Invalidate the hardware page table for the given region.
        #[cfg(target_os = "dragonfly")]
        MADV_INVAL,
        /// Set the offset of the page directory page to `value` for the virtual page table.
        #[cfg(target_os = "dragonfly")]
        MADV_SETMAP,
        /// Indicates that the application will not need the data in the given range.
        #[cfg(any(target_os = "ios", target_os = "macos"))]
        MADV_ZERO_WIRED_PAGES,
        #[cfg(any(target_os = "ios", target_os = "macos"))]
        MADV_FREE_REUSABLE,
        #[cfg(any(target_os = "ios", target_os = "macos"))]
        MADV_FREE_REUSE,
        #[cfg(any(target_os = "ios", target_os = "macos"))]
        MADV_CAN_REUSE,
    }
}

libc_bitflags!{
    /// Configuration flags for [`msync`].
    pub struct MsFlags: c_int {
        /// Schedule an update but return immediately.
        MS_ASYNC;
        /// Invalidate all cached data.
        MS_INVALIDATE;
        /// Invalidate pages, but leave them mapped.
        #[cfg(any(target_os = "ios", target_os = "macos"))]
        MS_KILLPAGES;
        /// Deactivate pages, but leave them mapped.
        #[cfg(any(target_os = "ios", target_os = "macos"))]
        MS_DEACTIVATE;
        /// Perform an update and wait for it to complete.
        MS_SYNC;
    }
}

libc_bitflags!{
    /// Flags for [`mlockall`].
    pub struct MlockAllFlags: c_int {
        /// Lock pages that are currently mapped into the address space of the process.
        MCL_CURRENT;
        /// Lock pages which will become mapped into the address space of the process in the future.
        MCL_FUTURE;
    }
}

/// Locks all memory pages that contain part of the address range with `length`
/// bytes starting at `addr`.
///
/// Locked pages never move to the swap area.
///
/// # Safety
///
/// `addr` must meet all the requirements described in the [`mlock(2)`] man page.
///
/// [`mlock(2)`]: https://man7.org/linux/man-pages/man2/mlock.2.html
pub unsafe fn mlock(addr: *const c_void, length: size_t) -> Result<()> {
    Errno::result(libc::mlock(addr, length)).map(drop)
}

/// Unlocks all memory pages that contain part of the address range with
/// `length` bytes starting at `addr`.
///
/// # Safety
///
/// `addr` must meet all the requirements described in the [`munlock(2)`] man
/// page.
///
/// [`munlock(2)`]: https://man7.org/linux/man-pages/man2/munlock.2.html
pub unsafe fn munlock(addr: *const c_void, length: size_t) -> Result<()> {
    Errno::result(libc::munlock(addr, length)).map(drop)
}

/// Locks all memory pages mapped into this process' address space.
///
/// Locked pages never move to the swap area. For more information, see [`mlockall(2)`].
///
/// [`mlockall(2)`]: https://man7.org/linux/man-pages/man2/mlockall.2.html
pub fn mlockall(flags: MlockAllFlags) -> Result<()> {
    unsafe { Errno::result(libc::mlockall(flags.bits())) }.map(drop)
}

/// Unlocks all memory pages mapped into this process' address space.
///
/// For more information, see [`munlockall(2)`].
///
/// [`munlockall(2)`]: https://man7.org/linux/man-pages/man2/munlockall.2.html
pub fn munlockall() -> Result<()> {
    unsafe { Errno::result(libc::munlockall()) }.map(drop)
}

/// allocate memory, or map files or devices into memory
///
/// # Safety
///
/// See the [`mmap(2)`] man page for detailed requirements.
///
/// [`mmap(2)`]: https://man7.org/linux/man-pages/man2/mmap.2.html
pub unsafe fn mmap(addr: *mut c_void, length: size_t, prot: ProtFlags, flags: MapFlags, fd: RawFd, offset: off_t) -> Result<*mut c_void> {
    let ret = libc::mmap(addr, length, prot.bits(), flags.bits(), fd, offset);

    if ret == libc::MAP_FAILED {
        Err(Errno::last())
    } else {
        Ok(ret)
    }
}

/// Expands (or shrinks) an existing memory mapping, potentially moving it at
/// the same time.
///
/// # Safety
///
/// See the `mremap(2)` [man page](https://man7.org/linux/man-pages/man2/mremap.2.html) for
/// detailed requirements.
#[cfg(any(target_os = "linux", target_os = "netbsd"))]
pub unsafe fn mremap(
    addr: *mut c_void,
    old_size: size_t,
    new_size: size_t,
    flags: MRemapFlags,
    new_address: Option<* mut c_void>,
) -> Result<*mut c_void> {
    #[cfg(target_os = "linux")]
    let ret = libc::mremap(addr, old_size, new_size, flags.bits(), new_address.unwrap_or(std::ptr::null_mut()));
    #[cfg(target_os = "netbsd")]
    let ret = libc::mremap(
        addr,
        old_size,
        new_address.unwrap_or(std::ptr::null_mut()),
        new_size,
        flags.bits(),
        );

    if ret == libc::MAP_FAILED {
        Err(Errno::last())
    } else {
        Ok(ret)
    }
}

/// remove a mapping
///
/// # Safety
///
/// `addr` must meet all the requirements described in the [`munmap(2)`] man
/// page.
///
/// [`munmap(2)`]: https://man7.org/linux/man-pages/man2/munmap.2.html
pub unsafe fn munmap(addr: *mut c_void, len: size_t) -> Result<()> {
    Errno::result(libc::munmap(addr, len)).map(drop)
}

/// give advice about use of memory
///
/// # Safety
///
/// See the [`madvise(2)`] man page.  Take special care when using
/// [`MmapAdvise::MADV_FREE`].
///
/// [`madvise(2)`]: https://man7.org/linux/man-pages/man2/madvise.2.html
pub unsafe fn madvise(addr: *mut c_void, length: size_t, advise: MmapAdvise) -> Result<()> {
    Errno::result(libc::madvise(addr, length, advise as i32)).map(drop)
}

/// Set protection of memory mapping.
///
/// See [`mprotect(3)`](https://pubs.opengroup.org/onlinepubs/9699919799/functions/mprotect.html) for
/// details.
///
/// # Safety
///
/// Calls to `mprotect` are inherently unsafe, as changes to memory protections can lead to
/// SIGSEGVs.
///
/// ```
/// # use nix::libc::size_t;
/// # use nix::sys::mman::{mmap, mprotect, MapFlags, ProtFlags};
/// # use std::ptr;
/// const ONE_K: size_t = 1024;
/// let mut slice: &mut [u8] = unsafe {
///     let mem = mmap(ptr::null_mut(), ONE_K, ProtFlags::PROT_NONE,
///                    MapFlags::MAP_ANON | MapFlags::MAP_PRIVATE, -1, 0).unwrap();
///     mprotect(mem, ONE_K, ProtFlags::PROT_READ | ProtFlags::PROT_WRITE).unwrap();
///     std::slice::from_raw_parts_mut(mem as *mut u8, ONE_K)
/// };
/// assert_eq!(slice[0], 0x00);
/// slice[0] = 0xFF;
/// assert_eq!(slice[0], 0xFF);
/// ```
pub unsafe fn mprotect(addr: *mut c_void, length: size_t, prot: ProtFlags) -> Result<()> {
    Errno::result(libc::mprotect(addr, length, prot.bits())).map(drop)
}

/// synchronize a mapped region
///
/// # Safety
///
/// `addr` must meet all the requirements described in the [`msync(2)`] man
/// page.
///
/// [`msync(2)`]: https://man7.org/linux/man-pages/man2/msync.2.html
pub unsafe fn msync(addr: *mut c_void, length: size_t, flags: MsFlags) -> Result<()> {
    Errno::result(libc::msync(addr, length, flags.bits())).map(drop)
}

/// Creates and opens a new, or opens an existing, POSIX shared memory object.
///
/// For more information, see [`shm_open(3)`].
///
/// [`shm_open(3)`]: https://man7.org/linux/man-pages/man3/shm_open.3.html
#[cfg(not(target_os = "android"))]
pub fn shm_open<P: ?Sized + NixPath>(name: &P, flag: OFlag, mode: Mode) -> Result<RawFd> {
    let ret = name.with_nix_path(|cstr| {
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        unsafe {
            libc::shm_open(cstr.as_ptr(), flag.bits(), mode.bits() as libc::c_uint)
        }
        #[cfg(not(any(target_os = "macos", target_os = "ios")))]
        unsafe {
            libc::shm_open(cstr.as_ptr(), flag.bits(), mode.bits() as libc::mode_t)
        }
    })?;

    Errno::result(ret)
}

/// Performs the converse of [`shm_open`], removing an object previously created.
///
/// For more information, see [`shm_unlink(3)`].
///
/// [`shm_unlink(3)`]: https://man7.org/linux/man-pages/man3/shm_unlink.3.html
#[cfg(not(target_os = "android"))]
pub fn shm_unlink<P: ?Sized + NixPath>(name: &P) -> Result<()> {
    let ret = name.with_nix_path(|cstr| {
        unsafe { libc::shm_unlink(cstr.as_ptr()) }
    })?;

    Errno::result(ret).map(drop)
}
