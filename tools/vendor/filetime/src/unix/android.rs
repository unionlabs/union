use crate::FileTime;
use std::ffi::CString;
use std::fs::File;
use std::io;
use std::os::unix::prelude::*;
use std::path::Path;

pub fn set_file_times(p: &Path, atime: FileTime, mtime: FileTime) -> io::Result<()> {
    set_times(p, Some(atime), Some(mtime), false)
}

pub fn set_file_mtime(p: &Path, mtime: FileTime) -> io::Result<()> {
    set_times(p, None, Some(mtime), false)
}

pub fn set_file_atime(p: &Path, atime: FileTime) -> io::Result<()> {
    set_times(p, Some(atime), None, false)
}

pub fn set_file_handle_times(
    f: &File,
    atime: Option<FileTime>,
    mtime: Option<FileTime>,
) -> io::Result<()> {
    let times = [super::to_timespec(&atime), super::to_timespec(&mtime)];

    // On Android NDK before version 19, `futimens` is not available.
    //
    // For better compatibility, we reimplement `futimens` using `utimensat`,
    // the same way as bionic libc uses it to implement `futimens`.
    let rc = unsafe { libc::utimensat(f.as_raw_fd(), core::ptr::null(), times.as_ptr(), 0) };
    if rc == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

pub fn set_symlink_file_times(p: &Path, atime: FileTime, mtime: FileTime) -> io::Result<()> {
    set_times(p, Some(atime), Some(mtime), true)
}

fn set_times(
    p: &Path,
    atime: Option<FileTime>,
    mtime: Option<FileTime>,
    symlink: bool,
) -> io::Result<()> {
    let flags = if symlink {
        libc::AT_SYMLINK_NOFOLLOW
    } else {
        0
    };

    let p = CString::new(p.as_os_str().as_bytes())?;
    let times = [super::to_timespec(&atime), super::to_timespec(&mtime)];
    let rc = unsafe { libc::utimensat(libc::AT_FDCWD, p.as_ptr(), times.as_ptr(), flags) };
    if rc == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}
