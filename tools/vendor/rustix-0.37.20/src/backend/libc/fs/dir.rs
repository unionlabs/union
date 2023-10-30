use super::super::c;
use super::super::conv::owned_fd;
use super::super::offset::libc_ino_t;
#[cfg(not(any(solarish, target_os = "haiku")))]
use super::types::FileType;
use crate::fd::{AsFd, BorrowedFd};
use crate::ffi::{CStr, CString};
use crate::fs::{fcntl_getfl, fstat, openat, Mode, OFlags, Stat};
#[cfg(not(any(
    solarish,
    target_os = "haiku",
    target_os = "netbsd",
    target_os = "redox",
    target_os = "wasi",
)))]
use crate::fs::{fstatfs, StatFs};
#[cfg(not(any(solarish, target_os = "haiku", target_os = "redox", target_os = "wasi")))]
use crate::fs::{fstatvfs, StatVfs};
use crate::io;
#[cfg(not(any(target_os = "fuchsia", target_os = "wasi")))]
use crate::process::fchdir;
use alloc::borrow::ToOwned;
#[cfg(not(linux_like))]
use c::readdir as libc_readdir;
#[cfg(linux_like)]
use c::readdir64 as libc_readdir;
use core::fmt;
use core::ptr::NonNull;
use libc_errno::{errno, set_errno, Errno};

/// `DIR*`
#[repr(transparent)]
pub struct Dir(NonNull<c::DIR>);

impl Dir {
    /// Construct a `Dir` that reads entries from the given directory
    /// file descriptor.
    #[inline]
    pub fn read_from<Fd: AsFd>(fd: Fd) -> io::Result<Self> {
        Self::_read_from(fd.as_fd())
    }

    #[inline]
    fn _read_from(fd: BorrowedFd<'_>) -> io::Result<Self> {
        // Given an arbitrary `OwnedFd`, it's impossible to know whether the
        // user holds a `dup`'d copy which could continue to modify the
        // file description state, which would cause Undefined Behavior after
        // our call to `fdopendir`. To prevent this, we obtain an independent
        // `OwnedFd`.
        let flags = fcntl_getfl(fd)?;
        let fd_for_dir = openat(fd, cstr!("."), flags | OFlags::CLOEXEC, Mode::empty())?;

        let raw = owned_fd(fd_for_dir);
        unsafe {
            let libc_dir = c::fdopendir(raw);

            if let Some(libc_dir) = NonNull::new(libc_dir) {
                Ok(Self(libc_dir))
            } else {
                let err = io::Errno::last_os_error();
                let _ = c::close(raw);
                Err(err)
            }
        }
    }

    /// `rewinddir(self)`
    #[inline]
    pub fn rewind(&mut self) {
        unsafe { c::rewinddir(self.0.as_ptr()) }
    }

    /// `readdir(self)`, where `None` means the end of the directory.
    pub fn read(&mut self) -> Option<io::Result<DirEntry>> {
        set_errno(Errno(0));
        let dirent_ptr = unsafe { libc_readdir(self.0.as_ptr()) };
        if dirent_ptr.is_null() {
            let curr_errno = errno().0;
            if curr_errno == 0 {
                // We successfully reached the end of the stream.
                None
            } else {
                // `errno` is unknown or non-zero, so an error occurred.
                Some(Err(io::Errno(curr_errno)))
            }
        } else {
            // We successfully read an entry.
            unsafe {
                let dirent = &*dirent_ptr;

                // We have our own copy of OpenBSD's dirent; check that the
                // layout minimally matches libc's.
                #[cfg(target_os = "openbsd")]
                check_dirent_layout(dirent);

                let result = DirEntry {
                    #[cfg(not(any(solarish, target_os = "aix", target_os = "haiku")))]
                    d_type: dirent.d_type,

                    #[cfg(not(any(freebsdlike, netbsdlike)))]
                    d_ino: dirent.d_ino,

                    #[cfg(any(freebsdlike, netbsdlike))]
                    d_fileno: dirent.d_fileno,

                    name: CStr::from_ptr(dirent.d_name.as_ptr()).to_owned(),
                };

                Some(Ok(result))
            }
        }
    }

    /// `fstat(self)`
    #[inline]
    pub fn stat(&self) -> io::Result<Stat> {
        fstat(unsafe { BorrowedFd::borrow_raw(c::dirfd(self.0.as_ptr())) })
    }

    /// `fstatfs(self)`
    #[cfg(not(any(
        solarish,
        target_os = "haiku",
        target_os = "netbsd",
        target_os = "redox",
        target_os = "wasi",
    )))]
    #[inline]
    pub fn statfs(&self) -> io::Result<StatFs> {
        fstatfs(unsafe { BorrowedFd::borrow_raw(c::dirfd(self.0.as_ptr())) })
    }

    /// `fstatvfs(self)`
    #[cfg(not(any(solarish, target_os = "haiku", target_os = "redox", target_os = "wasi")))]
    #[inline]
    pub fn statvfs(&self) -> io::Result<StatVfs> {
        fstatvfs(unsafe { BorrowedFd::borrow_raw(c::dirfd(self.0.as_ptr())) })
    }

    /// `fchdir(self)`
    #[cfg(not(any(target_os = "fuchsia", target_os = "wasi")))]
    #[inline]
    pub fn chdir(&self) -> io::Result<()> {
        fchdir(unsafe { BorrowedFd::borrow_raw(c::dirfd(self.0.as_ptr())) })
    }
}

/// `Dir` implements `Send` but not `Sync`, because we use `readdir` which is
/// not guaranteed to be thread-safe. Users can wrap this in a `Mutex` if they
/// need `Sync`, which is effectively what'd need to do to implement `Sync`
/// ourselves.
unsafe impl Send for Dir {}

impl Drop for Dir {
    #[inline]
    fn drop(&mut self) {
        unsafe { c::closedir(self.0.as_ptr()) };
    }
}

impl Iterator for Dir {
    type Item = io::Result<DirEntry>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        Self::read(self)
    }
}

impl fmt::Debug for Dir {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Dir")
            .field("fd", unsafe { &c::dirfd(self.0.as_ptr()) })
            .finish()
    }
}

/// `struct dirent`
#[derive(Debug)]
pub struct DirEntry {
    #[cfg(not(any(solarish, target_os = "aix", target_os = "haiku")))]
    d_type: u8,

    #[cfg(not(any(freebsdlike, netbsdlike)))]
    d_ino: libc_ino_t,

    #[cfg(any(freebsdlike, netbsdlike))]
    d_fileno: libc_ino_t,

    name: CString,
}

impl DirEntry {
    /// Returns the file name of this directory entry.
    #[inline]
    pub fn file_name(&self) -> &CStr {
        &self.name
    }

    /// Returns the type of this directory entry.
    #[cfg(not(any(solarish, target_os = "aix", target_os = "haiku")))]
    #[inline]
    pub fn file_type(&self) -> FileType {
        FileType::from_dirent_d_type(self.d_type)
    }

    /// Return the inode number of this directory entry.
    #[cfg(not(any(freebsdlike, netbsdlike)))]
    #[inline]
    pub fn ino(&self) -> u64 {
        self.d_ino as u64
    }

    /// Return the inode number of this directory entry.
    #[cfg(any(freebsdlike, netbsdlike))]
    #[inline]
    pub fn ino(&self) -> u64 {
        #[allow(clippy::useless_conversion)]
        self.d_fileno.into()
    }
}

/// libc's OpenBSD `dirent` has a private field so we can't construct it
/// directly, so we declare it ourselves to make all fields accessible.
#[cfg(target_os = "openbsd")]
#[repr(C)]
#[derive(Debug)]
struct libc_dirent {
    d_fileno: c::ino_t,
    d_off: c::off_t,
    d_reclen: u16,
    d_type: u8,
    d_namlen: u8,
    __d_padding: [u8; 4],
    d_name: [c::c_char; 256],
}

/// We have our own copy of OpenBSD's dirent; check that the layout
/// minimally matches libc's.
#[cfg(target_os = "openbsd")]
fn check_dirent_layout(dirent: &c::dirent) {
    use crate::utils::as_ptr;
    use core::mem::{align_of, size_of};

    // Check that the basic layouts match.
    assert_eq!(size_of::<libc_dirent>(), size_of::<c::dirent>());
    assert_eq!(align_of::<libc_dirent>(), align_of::<c::dirent>());

    // Check that the field offsets match.
    assert_eq!(
        {
            let z = libc_dirent {
                d_fileno: 0_u64,
                d_off: 0_i64,
                d_reclen: 0_u16,
                d_type: 0_u8,
                d_namlen: 0_u8,
                __d_padding: [0_u8; 4],
                d_name: [0 as c::c_char; 256],
            };
            let base = as_ptr(&z) as usize;
            (
                (as_ptr(&z.d_fileno) as usize) - base,
                (as_ptr(&z.d_off) as usize) - base,
                (as_ptr(&z.d_reclen) as usize) - base,
                (as_ptr(&z.d_type) as usize) - base,
                (as_ptr(&z.d_namlen) as usize) - base,
                (as_ptr(&z.d_name) as usize) - base,
            )
        },
        {
            let z = dirent;
            let base = as_ptr(z) as usize;
            (
                (as_ptr(&z.d_fileno) as usize) - base,
                (as_ptr(&z.d_off) as usize) - base,
                (as_ptr(&z.d_reclen) as usize) - base,
                (as_ptr(&z.d_type) as usize) - base,
                (as_ptr(&z.d_namlen) as usize) - base,
                (as_ptr(&z.d_name) as usize) - base,
            )
        }
    );
}
