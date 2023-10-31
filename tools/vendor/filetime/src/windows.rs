use crate::FileTime;
use std::fs::{self, File, OpenOptions};
use std::io;
use std::os::windows::prelude::*;
use std::path::Path;
use std::ptr;
use windows_sys::Win32::Foundation::{FILETIME, HANDLE};
use windows_sys::Win32::Storage::FileSystem::*;

pub fn set_file_times(p: &Path, atime: FileTime, mtime: FileTime) -> io::Result<()> {
    let f = OpenOptions::new()
        .write(true)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS)
        .open(p)?;
    set_file_handle_times(&f, Some(atime), Some(mtime))
}

pub fn set_file_mtime(p: &Path, mtime: FileTime) -> io::Result<()> {
    let f = OpenOptions::new()
        .write(true)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS)
        .open(p)?;
    set_file_handle_times(&f, None, Some(mtime))
}

pub fn set_file_atime(p: &Path, atime: FileTime) -> io::Result<()> {
    let f = OpenOptions::new()
        .write(true)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS)
        .open(p)?;
    set_file_handle_times(&f, Some(atime), None)
}

pub fn set_file_handle_times(
    f: &File,
    atime: Option<FileTime>,
    mtime: Option<FileTime>,
) -> io::Result<()> {
    let atime = atime.map(to_filetime);
    let mtime = mtime.map(to_filetime);
    return unsafe {
        let ret = SetFileTime(
            f.as_raw_handle() as HANDLE,
            ptr::null(),
            atime
                .as_ref()
                .map(|p| p as *const FILETIME)
                .unwrap_or(ptr::null()),
            mtime
                .as_ref()
                .map(|p| p as *const FILETIME)
                .unwrap_or(ptr::null()),
        );
        if ret != 0 {
            Ok(())
        } else {
            Err(io::Error::last_os_error())
        }
    };

    fn to_filetime(ft: FileTime) -> FILETIME {
        let intervals = ft.seconds() * (1_000_000_000 / 100) + ((ft.nanoseconds() as i64) / 100);
        FILETIME {
            dwLowDateTime: intervals as u32,
            dwHighDateTime: (intervals >> 32) as u32,
        }
    }
}

pub fn set_symlink_file_times(p: &Path, atime: FileTime, mtime: FileTime) -> io::Result<()> {
    use std::os::windows::fs::OpenOptionsExt;

    let f = OpenOptions::new()
        .write(true)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS)
        .open(p)?;
    set_file_handle_times(&f, Some(atime), Some(mtime))
}

pub fn from_last_modification_time(meta: &fs::Metadata) -> FileTime {
    from_intervals(meta.last_write_time())
}

pub fn from_last_access_time(meta: &fs::Metadata) -> FileTime {
    from_intervals(meta.last_access_time())
}

pub fn from_creation_time(meta: &fs::Metadata) -> Option<FileTime> {
    Some(from_intervals(meta.creation_time()))
}

fn from_intervals(ticks: u64) -> FileTime {
    // Windows write times are in 100ns intervals, so do a little math to
    // get it into the right representation.
    FileTime {
        seconds: (ticks / (1_000_000_000 / 100)) as i64,
        nanos: ((ticks % (1_000_000_000 / 100)) * 100) as u32,
    }
}
