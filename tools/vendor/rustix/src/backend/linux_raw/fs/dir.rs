use crate::fd::{AsFd, BorrowedFd, OwnedFd};
use crate::ffi::{CStr, CString};
use crate::fs::{
    fcntl_getfl, fstat, fstatfs, fstatvfs, openat, FileType, Mode, OFlags, Stat, StatFs, StatVfs,
};
use crate::io;
#[cfg(feature = "process")]
use crate::process::fchdir;
use crate::utils::as_ptr;
use alloc::borrow::ToOwned;
use alloc::vec::Vec;
use core::fmt;
use core::mem::size_of;
use linux_raw_sys::general::{linux_dirent64, SEEK_SET};

/// `DIR*`
pub struct Dir {
    /// The `OwnedFd` that we read directory entries from.
    fd: OwnedFd,

    buf: Vec<u8>,
    pos: usize,
    next: Option<u64>,
}

impl Dir {
    /// Construct a `Dir` that reads entries from the given directory
    /// file descriptor.
    #[inline]
    pub fn read_from<Fd: AsFd>(fd: Fd) -> io::Result<Self> {
        Self::_read_from(fd.as_fd())
    }

    #[inline]
    fn _read_from(fd: BorrowedFd<'_>) -> io::Result<Self> {
        let flags = fcntl_getfl(fd)?;
        let fd_for_dir = openat(fd, cstr!("."), flags | OFlags::CLOEXEC, Mode::empty())?;

        Ok(Self {
            fd: fd_for_dir,
            buf: Vec::new(),
            pos: 0,
            next: None,
        })
    }

    /// `rewinddir(self)`
    #[inline]
    pub fn rewind(&mut self) {
        self.pos = self.buf.len();
        self.next = Some(0);
    }

    /// `readdir(self)`, where `None` means the end of the directory.
    pub fn read(&mut self) -> Option<io::Result<DirEntry>> {
        if let Some(next) = self.next.take() {
            match crate::backend::fs::syscalls::_seek(self.fd.as_fd(), next as i64, SEEK_SET) {
                Ok(_) => (),
                Err(err) => return Some(Err(err)),
            }
        }

        // Compute linux_dirent64 field offsets.
        let z = linux_dirent64 {
            d_ino: 0_u64,
            d_off: 0_i64,
            d_type: 0_u8,
            d_reclen: 0_u16,
            d_name: Default::default(),
        };
        let base = as_ptr(&z) as usize;
        let offsetof_d_reclen = (as_ptr(&z.d_reclen) as usize) - base;
        let offsetof_d_name = (as_ptr(&z.d_name) as usize) - base;
        let offsetof_d_ino = (as_ptr(&z.d_ino) as usize) - base;
        let offsetof_d_type = (as_ptr(&z.d_type) as usize) - base;

        // Test if we need more entries, and if so, read more.
        if self.buf.len() - self.pos < size_of::<linux_dirent64>() {
            match self.read_more()? {
                Ok(()) => (),
                Err(e) => return Some(Err(e)),
            }
        }

        // We successfully read an entry. Extract the fields.
        let pos = self.pos;

        // Do an unaligned u16 load.
        let d_reclen = u16::from_ne_bytes([
            self.buf[pos + offsetof_d_reclen],
            self.buf[pos + offsetof_d_reclen + 1],
        ]);
        assert!(self.buf.len() - pos >= d_reclen as usize);
        self.pos += d_reclen as usize;

        // Read the NUL-terminated name from the `d_name` field. Without
        // `unsafe`, we need to scan for the NUL twice: once to obtain a size
        // for the slice, and then once within `CStr::from_bytes_with_nul`.
        let name_start = pos + offsetof_d_name;
        let name_len = self.buf[name_start..]
            .iter()
            .position(|x| *x == b'\0')
            .unwrap();
        let name = CStr::from_bytes_with_nul(&self.buf[name_start..][..=name_len]).unwrap();
        let name = name.to_owned();
        assert!(name.as_bytes().len() <= self.buf.len() - name_start);

        // Do an unaligned u64 load.
        let d_ino = u64::from_ne_bytes([
            self.buf[pos + offsetof_d_ino],
            self.buf[pos + offsetof_d_ino + 1],
            self.buf[pos + offsetof_d_ino + 2],
            self.buf[pos + offsetof_d_ino + 3],
            self.buf[pos + offsetof_d_ino + 4],
            self.buf[pos + offsetof_d_ino + 5],
            self.buf[pos + offsetof_d_ino + 6],
            self.buf[pos + offsetof_d_ino + 7],
        ]);

        let d_type = self.buf[pos + offsetof_d_type];

        // Check that our types correspond to the `linux_dirent64` types.
        let _ = linux_dirent64 {
            d_ino,
            d_off: 0,
            d_type,
            d_reclen,
            d_name: Default::default(),
        };

        Some(Ok(DirEntry {
            d_ino,
            d_type,
            name,
        }))
    }

    fn read_more(&mut self) -> Option<io::Result<()>> {
        let og_len = self.buf.len();
        // Capacity increment currently chosen by wild guess.
        self.buf
            .resize(self.buf.capacity() + 32 * size_of::<linux_dirent64>(), 0);
        let nread = match crate::backend::fs::syscalls::getdents(self.fd.as_fd(), &mut self.buf) {
            Ok(nread) => nread,
            Err(err) => {
                self.buf.resize(og_len, 0);
                return Some(Err(err));
            }
        };
        self.buf.resize(nread, 0);
        self.pos = 0;
        if nread == 0 {
            None
        } else {
            Some(Ok(()))
        }
    }

    /// `fstat(self)`
    #[inline]
    pub fn stat(&self) -> io::Result<Stat> {
        fstat(&self.fd)
    }

    /// `fstatfs(self)`
    #[inline]
    pub fn statfs(&self) -> io::Result<StatFs> {
        fstatfs(&self.fd)
    }

    /// `fstatvfs(self)`
    #[inline]
    pub fn statvfs(&self) -> io::Result<StatVfs> {
        fstatvfs(&self.fd)
    }

    /// `fchdir(self)`
    #[cfg(feature = "process")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "process")))]
    #[inline]
    pub fn chdir(&self) -> io::Result<()> {
        fchdir(&self.fd)
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
        f.debug_struct("Dir").field("fd", &self.fd).finish()
    }
}

/// `struct dirent`
#[derive(Debug)]
pub struct DirEntry {
    d_ino: u64,
    d_type: u8,
    name: CString,
}

impl DirEntry {
    /// Returns the file name of this directory entry.
    #[inline]
    pub fn file_name(&self) -> &CStr {
        &self.name
    }

    /// Returns the type of this directory entry.
    #[inline]
    pub fn file_type(&self) -> FileType {
        FileType::from_dirent_d_type(self.d_type)
    }

    /// Return the inode number of this directory entry.
    #[inline]
    pub fn ino(&self) -> u64 {
        self.d_ino
    }
}
