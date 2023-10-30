use crate::FileTime;
use std::fs::{self, File};
use std::io;
use std::os::unix::prelude::*;
use std::path::Path;

pub fn set_file_times(p: &Path, atime: FileTime, mtime: FileTime) -> io::Result<()> {
    let fd = open_redox(p, 0).map_err(|err| io::Error::from_raw_os_error(err.errno))?;
    let res = set_file_times_redox(fd, atime, mtime);
    let _ = syscall::close(fd);
    res
}

pub fn set_file_mtime(p: &Path, mtime: FileTime) -> io::Result<()> {
    let fd = open_redox(p, 0).map_err(|err| io::Error::from_raw_os_error(err.errno))?;
    let mut st = syscall::Stat::default();
    let res = match syscall::fstat(fd, &mut st) {
        Err(err) => Err(io::Error::from_raw_os_error(err.errno)),
        Ok(_) => set_file_times_redox(
            fd,
            FileTime {
                seconds: st.st_atime as i64,
                nanos: st.st_atime_nsec as u32,
            },
            mtime,
        ),
    };
    let _ = syscall::close(fd);
    res
}

pub fn set_file_atime(p: &Path, atime: FileTime) -> io::Result<()> {
    let fd = open_redox(p, 0).map_err(|err| io::Error::from_raw_os_error(err.errno))?;
    let mut st = syscall::Stat::default();
    let res = match syscall::fstat(fd, &mut st) {
        Err(err) => Err(io::Error::from_raw_os_error(err.errno)),
        Ok(_) => set_file_times_redox(
            fd,
            atime,
            FileTime {
                seconds: st.st_mtime as i64,
                nanos: st.st_mtime_nsec as u32,
            },
        ),
    };
    let _ = syscall::close(fd);
    res
}

pub fn set_symlink_file_times(p: &Path, atime: FileTime, mtime: FileTime) -> io::Result<()> {
    let fd = open_redox(p, syscall::O_NOFOLLOW)
        .map_err(|err| io::Error::from_raw_os_error(err.errno))?;
    let res = set_file_times_redox(fd, atime, mtime);
    let _ = syscall::close(fd);
    res
}

pub fn set_file_handle_times(
    f: &File,
    atime: Option<FileTime>,
    mtime: Option<FileTime>,
) -> io::Result<()> {
    let (atime1, mtime1) = match (atime, mtime) {
        (Some(a), Some(b)) => (a, b),
        (None, None) => return Ok(()),
        (Some(a), None) => {
            let meta = f.metadata()?;
            (a, FileTime::from_last_modification_time(&meta))
        }
        (None, Some(b)) => {
            let meta = f.metadata()?;
            (FileTime::from_last_access_time(&meta), b)
        }
    };
    set_file_times_redox(f.as_raw_fd() as usize, atime1, mtime1)
}

fn open_redox(path: &Path, flags: usize) -> syscall::Result<usize> {
    match path.to_str() {
        Some(string) => syscall::open(string, flags),
        None => Err(syscall::Error::new(syscall::EINVAL)),
    }
}

fn set_file_times_redox(fd: usize, atime: FileTime, mtime: FileTime) -> io::Result<()> {
    use syscall::TimeSpec;

    fn to_timespec(ft: &FileTime) -> TimeSpec {
        TimeSpec {
            tv_sec: ft.seconds(),
            tv_nsec: ft.nanoseconds() as i32,
        }
    }

    let times = [to_timespec(&atime), to_timespec(&mtime)];
    match syscall::futimens(fd, &times) {
        Ok(_) => Ok(()),
        Err(err) => Err(io::Error::from_raw_os_error(err.errno)),
    }
}

pub fn from_last_modification_time(meta: &fs::Metadata) -> FileTime {
    FileTime {
        seconds: meta.mtime(),
        nanos: meta.mtime_nsec() as u32,
    }
}

pub fn from_last_access_time(meta: &fs::Metadata) -> FileTime {
    FileTime {
        seconds: meta.atime(),
        nanos: meta.atime_nsec() as u32,
    }
}

pub fn from_creation_time(_meta: &fs::Metadata) -> Option<FileTime> {
    None
}
