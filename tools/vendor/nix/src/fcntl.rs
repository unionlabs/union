use crate::errno::Errno;
use libc::{self, c_char, c_int, c_uint, size_t, ssize_t};
use std::ffi::OsString;
#[cfg(not(target_os = "redox"))]
use std::os::raw;
use std::os::unix::ffi::OsStringExt;
use std::os::unix::io::RawFd;
use crate::sys::stat::Mode;
use crate::{NixPath, Result};

#[cfg(any(target_os = "android", target_os = "linux"))]
use std::ptr; // For splice and copy_file_range
#[cfg(any(target_os = "android", target_os = "linux"))]
use crate::sys::uio::IoVec; // For vmsplice

#[cfg(any(
    target_os = "linux",
    target_os = "android",
    target_os = "emscripten",
    target_os = "fuchsia",
    any(target_os = "wasi", target_env = "wasi"),
    target_env = "uclibc",
    target_os = "freebsd"
))]
pub use self::posix_fadvise::*;

#[cfg(not(target_os = "redox"))]
libc_bitflags! {
    pub struct AtFlags: c_int {
        AT_REMOVEDIR;
        AT_SYMLINK_FOLLOW;
        AT_SYMLINK_NOFOLLOW;
        #[cfg(any(target_os = "android", target_os = "linux"))]
        AT_NO_AUTOMOUNT;
        #[cfg(any(target_os = "android", target_os = "linux"))]
        AT_EMPTY_PATH;
        #[cfg(any(target_os = "illumos", target_os = "solaris"))]
        AT_EACCESS;
    }
}

libc_bitflags!(
    /// Configuration options for opened files.
    pub struct OFlag: c_int {
        /// Mask for the access mode of the file.
        O_ACCMODE;
        /// Use alternate I/O semantics.
        #[cfg(target_os = "netbsd")]
        O_ALT_IO;
        /// Open the file in append-only mode.
        O_APPEND;
        /// Generate a signal when input or output becomes possible.
        #[cfg(not(any(target_os = "illumos", target_os = "solaris")))]
        O_ASYNC;
        /// Closes the file descriptor once an `execve` call is made.
        ///
        /// Also sets the file offset to the beginning of the file.
        O_CLOEXEC;
        /// Create the file if it does not exist.
        O_CREAT;
        /// Try to minimize cache effects of the I/O for this file.
        #[cfg(any(target_os = "android",
                  target_os = "dragonfly",
                  target_os = "freebsd",
                  target_os = "linux",
                  target_os = "netbsd"))]
        O_DIRECT;
        /// If the specified path isn't a directory, fail.
        #[cfg(not(any(target_os = "illumos", target_os = "solaris")))]
        O_DIRECTORY;
        /// Implicitly follow each `write()` with an `fdatasync()`.
        #[cfg(any(target_os = "android",
                  target_os = "ios",
                  target_os = "linux",
                  target_os = "macos",
                  target_os = "netbsd",
                  target_os = "openbsd"))]
        O_DSYNC;
        /// Error out if a file was not created.
        O_EXCL;
        /// Open for execute only.
        #[cfg(target_os = "freebsd")]
        O_EXEC;
        /// Open with an exclusive file lock.
        #[cfg(any(target_os = "dragonfly",
                  target_os = "freebsd",
                  target_os = "ios",
                  target_os = "macos",
                  target_os = "netbsd",
                  target_os = "openbsd",
                  target_os = "redox"))]
        O_EXLOCK;
        /// Same as `O_SYNC`.
        #[cfg(any(target_os = "dragonfly",
                  target_os = "freebsd",
                  target_os = "ios",
                  all(target_os = "linux", not(target_env = "musl")),
                  target_os = "macos",
                  target_os = "netbsd",
                  target_os = "openbsd",
                  target_os = "redox"))]
        O_FSYNC;
        /// Allow files whose sizes can't be represented in an `off_t` to be opened.
        #[cfg(any(target_os = "android", target_os = "linux"))]
        O_LARGEFILE;
        /// Do not update the file last access time during `read(2)`s.
        #[cfg(any(target_os = "android", target_os = "linux"))]
        O_NOATIME;
        /// Don't attach the device as the process' controlling terminal.
        #[cfg(not(target_os = "redox"))]
        O_NOCTTY;
        /// Same as `O_NONBLOCK`.
        #[cfg(not(target_os = "redox"))]
        O_NDELAY;
        /// `open()` will fail if the given path is a symbolic link.
        O_NOFOLLOW;
        /// When possible, open the file in nonblocking mode.
        O_NONBLOCK;
        /// Don't deliver `SIGPIPE`.
        #[cfg(target_os = "netbsd")]
        O_NOSIGPIPE;
        /// Obtain a file descriptor for low-level access.
        ///
        /// The file itself is not opened and other file operations will fail.
        #[cfg(any(target_os = "android", target_os = "linux", target_os = "redox"))]
        O_PATH;
        /// Only allow reading.
        ///
        /// This should not be combined with `O_WRONLY` or `O_RDWR`.
        O_RDONLY;
        /// Allow both reading and writing.
        ///
        /// This should not be combined with `O_WRONLY` or `O_RDONLY`.
        O_RDWR;
        /// Similar to `O_DSYNC` but applies to `read`s instead.
        #[cfg(any(target_os = "linux", target_os = "netbsd", target_os = "openbsd"))]
        O_RSYNC;
        /// Skip search permission checks.
        #[cfg(target_os = "netbsd")]
        O_SEARCH;
        /// Open with a shared file lock.
        #[cfg(any(target_os = "dragonfly",
                  target_os = "freebsd",
                  target_os = "ios",
                  target_os = "macos",
                  target_os = "netbsd",
                  target_os = "openbsd",
                  target_os = "redox"))]
        O_SHLOCK;
        /// Implicitly follow each `write()` with an `fsync()`.
        #[cfg(not(target_os = "redox"))]
        O_SYNC;
        /// Create an unnamed temporary file.
        #[cfg(any(target_os = "android", target_os = "linux"))]
        O_TMPFILE;
        /// Truncate an existing regular file to 0 length if it allows writing.
        O_TRUNC;
        /// Restore default TTY attributes.
        #[cfg(target_os = "freebsd")]
        O_TTY_INIT;
        /// Only allow writing.
        ///
        /// This should not be combined with `O_RDONLY` or `O_RDWR`.
        O_WRONLY;
    }
);

// The conversion is not identical on all operating systems.
#[allow(clippy::useless_conversion)]
pub fn open<P: ?Sized + NixPath>(path: &P, oflag: OFlag, mode: Mode) -> Result<RawFd> {
    let fd = path.with_nix_path(|cstr| {
        unsafe { libc::open(cstr.as_ptr(), oflag.bits(), mode.bits() as c_uint) }
    })?;

    Errno::result(fd)
}

// The conversion is not identical on all operating systems.
#[allow(clippy::useless_conversion)]
#[cfg(not(target_os = "redox"))]
pub fn openat<P: ?Sized + NixPath>(
    dirfd: RawFd,
    path: &P,
    oflag: OFlag,
    mode: Mode,
) -> Result<RawFd> {
    let fd = path.with_nix_path(|cstr| {
        unsafe { libc::openat(dirfd, cstr.as_ptr(), oflag.bits(), mode.bits() as c_uint) }
    })?;
    Errno::result(fd)
}

#[cfg(not(target_os = "redox"))]
pub fn renameat<P1: ?Sized + NixPath, P2: ?Sized + NixPath>(
    old_dirfd: Option<RawFd>,
    old_path: &P1,
    new_dirfd: Option<RawFd>,
    new_path: &P2,
) -> Result<()> {
    let res = old_path.with_nix_path(|old_cstr| {
        new_path.with_nix_path(|new_cstr| unsafe {
            libc::renameat(
                at_rawfd(old_dirfd),
                old_cstr.as_ptr(),
                at_rawfd(new_dirfd),
                new_cstr.as_ptr(),
            )
        })
    })??;
    Errno::result(res).map(drop)
}

#[cfg(all(
    target_os = "linux",
    target_env = "gnu",
))]
libc_bitflags! {
    pub struct RenameFlags: u32 {
        RENAME_EXCHANGE;
        RENAME_NOREPLACE;
        RENAME_WHITEOUT;
    }
}

#[cfg(all(
    target_os = "linux",
    target_env = "gnu",
))]
pub fn renameat2<P1: ?Sized + NixPath, P2: ?Sized + NixPath>(
    old_dirfd: Option<RawFd>,
    old_path: &P1,
    new_dirfd: Option<RawFd>,
    new_path: &P2,
    flags: RenameFlags,
) -> Result<()> {
    let res = old_path.with_nix_path(|old_cstr| {
        new_path.with_nix_path(|new_cstr| unsafe {
            libc::renameat2(
                at_rawfd(old_dirfd),
                old_cstr.as_ptr(),
                at_rawfd(new_dirfd),
                new_cstr.as_ptr(),
                flags.bits(),
            )
        })
    })??;
    Errno::result(res).map(drop)
}

fn wrap_readlink_result(mut v: Vec<u8>, len: ssize_t) -> Result<OsString> {
    unsafe { v.set_len(len as usize) }
    v.shrink_to_fit();
    Ok(OsString::from_vec(v.to_vec()))
}

fn readlink_maybe_at<P: ?Sized + NixPath>(
    dirfd: Option<RawFd>,
    path: &P,
    v: &mut Vec<u8>,
) -> Result<libc::ssize_t> {
    path.with_nix_path(|cstr| unsafe {
        match dirfd {
            #[cfg(target_os = "redox")]
            Some(_) => unreachable!(),
            #[cfg(not(target_os = "redox"))]
            Some(dirfd) => libc::readlinkat(
                dirfd,
                cstr.as_ptr(),
                v.as_mut_ptr() as *mut c_char,
                v.capacity() as size_t,
            ),
            None => libc::readlink(
                cstr.as_ptr(),
                v.as_mut_ptr() as *mut c_char,
                v.capacity() as size_t,
            ),
        }
    })
}

fn inner_readlink<P: ?Sized + NixPath>(dirfd: Option<RawFd>, path: &P) -> Result<OsString> {
    let mut v = Vec::with_capacity(libc::PATH_MAX as usize);
    // simple case: result is strictly less than `PATH_MAX`
    let res = readlink_maybe_at(dirfd, path, &mut v)?;
    let len = Errno::result(res)?;
    debug_assert!(len >= 0);
    if (len as usize) < v.capacity() {
        return wrap_readlink_result(v, res);
    }
    // Uh oh, the result is too long...
    // Let's try to ask lstat how many bytes to allocate.
    let reported_size = match dirfd {
        #[cfg(target_os = "redox")]
        Some(_) => unreachable!(),
        #[cfg(any(target_os = "android", target_os = "linux"))]
        Some(dirfd) => {
            let flags = if path.is_empty() { AtFlags::AT_EMPTY_PATH } else { AtFlags::empty() };
            super::sys::stat::fstatat(dirfd, path, flags | AtFlags::AT_SYMLINK_NOFOLLOW)
        },
        #[cfg(not(any(target_os = "android", target_os = "linux", target_os = "redox")))]
        Some(dirfd) => super::sys::stat::fstatat(dirfd, path, AtFlags::AT_SYMLINK_NOFOLLOW),
        None => super::sys::stat::lstat(path)
    }
        .map(|x| x.st_size)
        .unwrap_or(0);
    let mut try_size = if reported_size > 0 {
        // Note: even if `lstat`'s apparently valid answer turns out to be
        // wrong, we will still read the full symlink no matter what.
        reported_size as usize + 1
    } else {
        // If lstat doesn't cooperate, or reports an error, be a little less
        // precise.
        (libc::PATH_MAX as usize).max(128) << 1
    };
    loop {
        v.reserve_exact(try_size);
        let res = readlink_maybe_at(dirfd, path, &mut v)?;
        let len = Errno::result(res)?;
        debug_assert!(len >= 0);
        if (len as usize) < v.capacity() {
            break wrap_readlink_result(v, res);
        } else {
            // Ugh! Still not big enough!
            match try_size.checked_shl(1) {
                Some(next_size) => try_size = next_size,
                // It's absurd that this would happen, but handle it sanely
                // anyway.
                None => break Err(Errno::ENAMETOOLONG),
            }
        }
    }
}

pub fn readlink<P: ?Sized + NixPath>(path: &P) -> Result<OsString> {
    inner_readlink(None, path)
}

#[cfg(not(target_os = "redox"))]
pub fn readlinkat<P: ?Sized + NixPath>(dirfd: RawFd, path: &P) -> Result<OsString> {
    inner_readlink(Some(dirfd), path)
}

/// Computes the raw fd consumed by a function of the form `*at`.
#[cfg(not(target_os = "redox"))]
pub(crate) fn at_rawfd(fd: Option<RawFd>) -> raw::c_int {
    match fd {
        None => libc::AT_FDCWD,
        Some(fd) => fd,
    }
}

#[cfg(any(target_os = "android", target_os = "linux"))]
libc_bitflags!(
    /// Additional flags for file sealing, which allows for limiting operations on a file.
    pub struct SealFlag: c_int {
        /// Prevents further calls to `fcntl()` with `F_ADD_SEALS`.
        F_SEAL_SEAL;
        /// The file cannot be reduced in size.
        F_SEAL_SHRINK;
        /// The size of the file cannot be increased.
        F_SEAL_GROW;
        /// The file contents cannot be modified.
        F_SEAL_WRITE;
    }
);

libc_bitflags!(
    /// Additional configuration flags for `fcntl`'s `F_SETFD`.
    pub struct FdFlag: c_int {
        /// The file descriptor will automatically be closed during a successful `execve(2)`.
        FD_CLOEXEC;
    }
);

#[cfg(not(target_os = "redox"))]
#[derive(Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum FcntlArg<'a> {
    F_DUPFD(RawFd),
    F_DUPFD_CLOEXEC(RawFd),
    F_GETFD,
    F_SETFD(FdFlag), // FD_FLAGS
    F_GETFL,
    F_SETFL(OFlag), // O_NONBLOCK
    F_SETLK(&'a libc::flock),
    F_SETLKW(&'a libc::flock),
    F_GETLK(&'a mut libc::flock),
    #[cfg(any(target_os = "linux", target_os = "android"))]
    F_OFD_SETLK(&'a libc::flock),
    #[cfg(any(target_os = "linux", target_os = "android"))]
    F_OFD_SETLKW(&'a libc::flock),
    #[cfg(any(target_os = "linux", target_os = "android"))]
    F_OFD_GETLK(&'a mut libc::flock),
    #[cfg(any(target_os = "android", target_os = "linux"))]
    F_ADD_SEALS(SealFlag),
    #[cfg(any(target_os = "android", target_os = "linux"))]
    F_GET_SEALS,
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    F_FULLFSYNC,
    #[cfg(any(target_os = "linux", target_os = "android"))]
    F_GETPIPE_SZ,
    #[cfg(any(target_os = "linux", target_os = "android"))]
    F_SETPIPE_SZ(c_int),
    // TODO: Rest of flags
}

#[cfg(target_os = "redox")]
#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum FcntlArg {
    F_DUPFD(RawFd),
    F_DUPFD_CLOEXEC(RawFd),
    F_GETFD,
    F_SETFD(FdFlag), // FD_FLAGS
    F_GETFL,
    F_SETFL(OFlag), // O_NONBLOCK
}
pub use self::FcntlArg::*;

// TODO: Figure out how to handle value fcntl returns
pub fn fcntl(fd: RawFd, arg: FcntlArg) -> Result<c_int> {
    let res = unsafe {
        match arg {
            F_DUPFD(rawfd) => libc::fcntl(fd, libc::F_DUPFD, rawfd),
            F_DUPFD_CLOEXEC(rawfd) => libc::fcntl(fd, libc::F_DUPFD_CLOEXEC, rawfd),
            F_GETFD => libc::fcntl(fd, libc::F_GETFD),
            F_SETFD(flag) => libc::fcntl(fd, libc::F_SETFD, flag.bits()),
            F_GETFL => libc::fcntl(fd, libc::F_GETFL),
            F_SETFL(flag) => libc::fcntl(fd, libc::F_SETFL, flag.bits()),
            #[cfg(not(target_os = "redox"))]
            F_SETLK(flock) => libc::fcntl(fd, libc::F_SETLK, flock),
            #[cfg(not(target_os = "redox"))]
            F_SETLKW(flock) => libc::fcntl(fd, libc::F_SETLKW, flock),
            #[cfg(not(target_os = "redox"))]
            F_GETLK(flock) => libc::fcntl(fd, libc::F_GETLK, flock),
            #[cfg(any(target_os = "android", target_os = "linux"))]
            F_OFD_SETLK(flock) => libc::fcntl(fd, libc::F_OFD_SETLK, flock),
            #[cfg(any(target_os = "android", target_os = "linux"))]
            F_OFD_SETLKW(flock) => libc::fcntl(fd, libc::F_OFD_SETLKW, flock),
            #[cfg(any(target_os = "android", target_os = "linux"))]
            F_OFD_GETLK(flock) => libc::fcntl(fd, libc::F_OFD_GETLK, flock),
            #[cfg(any(target_os = "android", target_os = "linux"))]
            F_ADD_SEALS(flag) => libc::fcntl(fd, libc::F_ADD_SEALS, flag.bits()),
            #[cfg(any(target_os = "android", target_os = "linux"))]
            F_GET_SEALS => libc::fcntl(fd, libc::F_GET_SEALS),
            #[cfg(any(target_os = "macos", target_os = "ios"))]
            F_FULLFSYNC => libc::fcntl(fd, libc::F_FULLFSYNC),
            #[cfg(any(target_os = "linux", target_os = "android"))]
            F_GETPIPE_SZ => libc::fcntl(fd, libc::F_GETPIPE_SZ),
            #[cfg(any(target_os = "linux", target_os = "android"))]
            F_SETPIPE_SZ(size) => libc::fcntl(fd, libc::F_SETPIPE_SZ, size),
        }
    };

    Errno::result(res)
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum FlockArg {
    LockShared,
    LockExclusive,
    Unlock,
    LockSharedNonblock,
    LockExclusiveNonblock,
    UnlockNonblock,
}

#[cfg(not(target_os = "redox"))]
pub fn flock(fd: RawFd, arg: FlockArg) -> Result<()> {
    use self::FlockArg::*;

    let res = unsafe {
        match arg {
            LockShared => libc::flock(fd, libc::LOCK_SH),
            LockExclusive => libc::flock(fd, libc::LOCK_EX),
            Unlock => libc::flock(fd, libc::LOCK_UN),
            LockSharedNonblock => libc::flock(fd, libc::LOCK_SH | libc::LOCK_NB),
            LockExclusiveNonblock => libc::flock(fd, libc::LOCK_EX | libc::LOCK_NB),
            UnlockNonblock => libc::flock(fd, libc::LOCK_UN | libc::LOCK_NB),
        }
    };

    Errno::result(res).map(drop)
}

#[cfg(any(target_os = "android", target_os = "linux"))]
libc_bitflags! {
    /// Additional flags to `splice` and friends.
    pub struct SpliceFFlags: c_uint {
        /// Request that pages be moved instead of copied.
        ///
        /// Not applicable to `vmsplice`.
        SPLICE_F_MOVE;
        /// Do not block on I/O.
        SPLICE_F_NONBLOCK;
        /// Hint that more data will be coming in a subsequent splice.
        ///
        /// Not applicable to `vmsplice`.
        SPLICE_F_MORE;
        /// Gift the user pages to the kernel.
        ///
        /// Not applicable to `splice`.
        SPLICE_F_GIFT;
    }
}

/// Copy a range of data from one file to another
///
/// The `copy_file_range` system call performs an in-kernel copy between
/// file descriptors `fd_in` and `fd_out` without the additional cost of
/// transferring data from the kernel to user space and then back into the
/// kernel. It copies up to `len` bytes of data from file descriptor `fd_in` to
/// file descriptor `fd_out`, overwriting any data that exists within the
/// requested range of the target file.
///
/// If the `off_in` and/or `off_out` arguments are used, the values
/// will be mutated to reflect the new position within the file after
/// copying. If they are not used, the relevant filedescriptors will be seeked
/// to the new position.
///
/// On successful completion the number of bytes actually copied will be
/// returned.
#[cfg(any(target_os = "android", target_os = "linux"))]
pub fn copy_file_range(
    fd_in: RawFd,
    off_in: Option<&mut libc::loff_t>,
    fd_out: RawFd,
    off_out: Option<&mut libc::loff_t>,
    len: usize,
) -> Result<usize> {
    let off_in = off_in
        .map(|offset| offset as *mut libc::loff_t)
        .unwrap_or(ptr::null_mut());
    let off_out = off_out
        .map(|offset| offset as *mut libc::loff_t)
        .unwrap_or(ptr::null_mut());

    let ret = unsafe {
        libc::syscall(
            libc::SYS_copy_file_range,
            fd_in,
            off_in,
            fd_out,
            off_out,
            len,
            0,
        )
    };
    Errno::result(ret).map(|r| r as usize)
}

#[cfg(any(target_os = "linux", target_os = "android"))]
pub fn splice(
    fd_in: RawFd,
    off_in: Option<&mut libc::loff_t>,
    fd_out: RawFd,
    off_out: Option<&mut libc::loff_t>,
    len: usize,
    flags: SpliceFFlags,
) -> Result<usize> {
    let off_in = off_in
        .map(|offset| offset as *mut libc::loff_t)
        .unwrap_or(ptr::null_mut());
    let off_out = off_out
        .map(|offset| offset as *mut libc::loff_t)
        .unwrap_or(ptr::null_mut());

    let ret = unsafe { libc::splice(fd_in, off_in, fd_out, off_out, len, flags.bits()) };
    Errno::result(ret).map(|r| r as usize)
}

#[cfg(any(target_os = "linux", target_os = "android"))]
pub fn tee(fd_in: RawFd, fd_out: RawFd, len: usize, flags: SpliceFFlags) -> Result<usize> {
    let ret = unsafe { libc::tee(fd_in, fd_out, len, flags.bits()) };
    Errno::result(ret).map(|r| r as usize)
}

#[cfg(any(target_os = "linux", target_os = "android"))]
pub fn vmsplice(fd: RawFd, iov: &[IoVec<&[u8]>], flags: SpliceFFlags) -> Result<usize> {
    let ret = unsafe {
        libc::vmsplice(
            fd,
            iov.as_ptr() as *const libc::iovec,
            iov.len(),
            flags.bits(),
        )
    };
    Errno::result(ret).map(|r| r as usize)
}

#[cfg(any(target_os = "linux"))]
libc_bitflags!(
    /// Mode argument flags for fallocate determining operation performed on a given range.
    pub struct FallocateFlags: c_int {
        /// File size is not changed.
        ///
        /// offset + len can be greater than file size.
        FALLOC_FL_KEEP_SIZE;
        /// Deallocates space by creating a hole.
        ///
        /// Must be ORed with FALLOC_FL_KEEP_SIZE. Byte range starts at offset and continues for len bytes.
        FALLOC_FL_PUNCH_HOLE;
        /// Removes byte range from a file without leaving a hole.
        ///
        /// Byte range to collapse starts at offset and continues for len bytes.
        FALLOC_FL_COLLAPSE_RANGE;
        /// Zeroes space in specified byte range.
        ///
        /// Byte range starts at offset and continues for len bytes.
        FALLOC_FL_ZERO_RANGE;
        /// Increases file space by inserting a hole within the file size.
        ///
        /// Does not overwrite existing data. Hole starts at offset and continues for len bytes.
        FALLOC_FL_INSERT_RANGE;
        /// Shared file data extants are made private to the file.
        ///
        /// Gaurantees that a subsequent write will not fail due to lack of space.
        FALLOC_FL_UNSHARE_RANGE;
    }
);

/// Manipulates file space.
///
/// Allows the caller to directly manipulate the allocated disk space for the
/// file referred to by fd.
#[cfg(any(target_os = "linux"))]
pub fn fallocate(
    fd: RawFd,
    mode: FallocateFlags,
    offset: libc::off_t,
    len: libc::off_t,
) -> Result<()> {
    let res = unsafe { libc::fallocate(fd, mode.bits(), offset, len) };
    Errno::result(res).map(drop)
}

#[cfg(any(
    target_os = "linux",
    target_os = "android",
    target_os = "emscripten",
    target_os = "fuchsia",
    any(target_os = "wasi", target_env = "wasi"),
    target_env = "uclibc",
    target_os = "freebsd"
))]
mod posix_fadvise {
    use crate::errno::Errno;
    use std::os::unix::io::RawFd;
    use crate::Result;

    libc_enum! {
        #[repr(i32)]
        #[non_exhaustive]
        pub enum PosixFadviseAdvice {
            POSIX_FADV_NORMAL,
            POSIX_FADV_SEQUENTIAL,
            POSIX_FADV_RANDOM,
            POSIX_FADV_NOREUSE,
            POSIX_FADV_WILLNEED,
            POSIX_FADV_DONTNEED,
        }
    }

    pub fn posix_fadvise(
        fd: RawFd,
        offset: libc::off_t,
        len: libc::off_t,
        advice: PosixFadviseAdvice,
    ) -> Result<()> {
        let res = unsafe { libc::posix_fadvise(fd, offset, len, advice as libc::c_int) };

        if res == 0 {
            Ok(())
        } else {
            Err(Errno::from_i32(res))
        }
    }
}

#[cfg(any(
    target_os = "linux",
    target_os = "android",
    target_os = "emscripten",
    target_os = "fuchsia",
    any(target_os = "wasi", target_env = "wasi"),
    target_os = "freebsd"
))]
pub fn posix_fallocate(fd: RawFd, offset: libc::off_t, len: libc::off_t) -> Result<()> {
    let res = unsafe { libc::posix_fallocate(fd, offset, len) };
    match Errno::result(res) {
        Err(err) => Err(err),
        Ok(0) => Ok(()),
        Ok(errno) => Err(Errno::from_i32(errno)),
    }
}
