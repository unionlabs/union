//! Get filesystem statistics
//!
//! See [the man pages](https://pubs.opengroup.org/onlinepubs/9699919799/functions/fstatvfs.html)
//! for more details.
use std::mem;
use std::os::unix::io::AsRawFd;

use libc::{self, c_ulong};

use crate::{Result, NixPath, errno::Errno};

#[cfg(not(target_os = "redox"))]
libc_bitflags!(
    /// File system mount Flags
    #[repr(C)]
    #[derive(Default)]
    pub struct FsFlags: c_ulong {
        /// Read Only
        ST_RDONLY;
        /// Do not allow the set-uid bits to have an effect
        ST_NOSUID;
        /// Do not interpret character or block-special devices
        #[cfg(any(target_os = "android", target_os = "linux"))]
        ST_NODEV;
        /// Do not allow execution of binaries on the filesystem
        #[cfg(any(target_os = "android", target_os = "linux"))]
        ST_NOEXEC;
        /// All IO should be done synchronously
        #[cfg(any(target_os = "android", target_os = "linux"))]
        ST_SYNCHRONOUS;
        /// Allow mandatory locks on the filesystem
        #[cfg(any(target_os = "android", target_os = "linux"))]
        ST_MANDLOCK;
        /// Write on file/directory/symlink
        #[cfg(target_os = "linux")]
        ST_WRITE;
        /// Append-only file
        #[cfg(target_os = "linux")]
        ST_APPEND;
        /// Immutable file
        #[cfg(target_os = "linux")]
        ST_IMMUTABLE;
        /// Do not update access times on files
        #[cfg(any(target_os = "android", target_os = "linux"))]
        ST_NOATIME;
        /// Do not update access times on files
        #[cfg(any(target_os = "android", target_os = "linux"))]
        ST_NODIRATIME;
        /// Update access time relative to modify/change time
        #[cfg(any(target_os = "android", all(target_os = "linux", not(target_env = "musl"))))]
        ST_RELATIME;
    }
);

/// Wrapper around the POSIX `statvfs` struct
///
/// For more information see the [`statvfs(3)` man pages](https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/sys_statvfs.h.html).
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Statvfs(libc::statvfs);

impl Statvfs {
    /// get the file system block size
    pub fn block_size(&self) -> c_ulong {
        self.0.f_bsize
    }

    /// Get the fundamental file system block size
    pub fn fragment_size(&self) -> c_ulong {
        self.0.f_frsize
    }

    /// Get the number of blocks.
    ///
    /// Units are in units of `fragment_size()`
    pub fn blocks(&self) -> libc::fsblkcnt_t {
        self.0.f_blocks
    }

    /// Get the number of free blocks in the file system
    pub fn blocks_free(&self) -> libc::fsblkcnt_t {
        self.0.f_bfree
    }

    /// Get the number of free blocks for unprivileged users
    pub fn blocks_available(&self) -> libc::fsblkcnt_t {
        self.0.f_bavail
    }

    /// Get the total number of file inodes
    pub fn files(&self) -> libc::fsfilcnt_t {
        self.0.f_files
    }

    /// Get the number of free file inodes
    pub fn files_free(&self) -> libc::fsfilcnt_t {
        self.0.f_ffree
    }

    /// Get the number of free file inodes for unprivileged users
    pub fn files_available(&self) -> libc::fsfilcnt_t {
        self.0.f_favail
    }

    /// Get the file system id
    pub fn filesystem_id(&self) -> c_ulong {
        self.0.f_fsid
    }

    /// Get the mount flags
    #[cfg(not(target_os = "redox"))]
    pub fn flags(&self) -> FsFlags {
        FsFlags::from_bits_truncate(self.0.f_flag)
    }

    /// Get the maximum filename length
    pub fn name_max(&self) -> c_ulong {
        self.0.f_namemax
    }

}

/// Return a `Statvfs` object with information about the `path`
pub fn statvfs<P: ?Sized + NixPath>(path: &P) -> Result<Statvfs> {
    unsafe {
        Errno::clear();
        let mut stat = mem::MaybeUninit::<libc::statvfs>::uninit();
        let res = path.with_nix_path(|path|
            libc::statvfs(path.as_ptr(), stat.as_mut_ptr())
        )?;

        Errno::result(res).map(|_| Statvfs(stat.assume_init()))
    }
}

/// Return a `Statvfs` object with information about `fd`
pub fn fstatvfs<T: AsRawFd>(fd: &T) -> Result<Statvfs> {
    unsafe {
        Errno::clear();
        let mut stat = mem::MaybeUninit::<libc::statvfs>::uninit();
        Errno::result(libc::fstatvfs(fd.as_raw_fd(), stat.as_mut_ptr()))
            .map(|_| Statvfs(stat.assume_init()))
    }
}

#[cfg(test)]
mod test {
    use std::fs::File;
    use crate::sys::statvfs::*;

    #[test]
    fn statvfs_call() {
        statvfs(&b"/"[..]).unwrap();
    }

    #[test]
    fn fstatvfs_call() {
        let root = File::open("/").unwrap();
        fstatvfs(&root).unwrap();
    }
}
