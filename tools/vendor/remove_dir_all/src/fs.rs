use std::{
    fs::{self, OpenOptions},
    io,
    mem::size_of,
    os::windows::prelude::*,
    path::{Path, PathBuf},
};

use rayon::prelude::*;
use winapi::shared::minwindef::*;
use winapi::um::fileapi::*;
use winapi::um::minwinbase::*;
use winapi::um::winbase::*;
use winapi::um::winnt::*;

/// Reliably removes a directory and all of its children.
///
/// ```rust
/// use std::fs;
/// use remove_dir_all::*;
///
/// fs::create_dir("./temp/").unwrap();
/// remove_dir_all("./temp/").unwrap();
/// ```
pub fn remove_dir_all<P: AsRef<Path>>(path: P) -> io::Result<()> {
    // On Windows it is not enough to just recursively remove the contents of a
    // directory and then the directory itself. Deleting does not happen
    // instantaneously, but is delayed by IO being completed in the fs stack.
    //
    // Further, typical Windows machines can handle many more concurrent IOs
    // than a single threaded application is capable of submitting: the
    // overlapped (async) calls available do not cover the operations needed to
    // perform directory removal.
    //
    // To work around this, we use a work stealing scheduler and submit
    // deletions concurrently with directory scanning, and delete sibling
    // directories in parallel. This allows the slight latency of
    // STATUS_DELETE_PENDING to only have logarithmic effect: a very deep tree
    // will pay wall clock time for that overhead per level as the tree traverse
    // completes, but not for every interior not as a simple recursive deletion
    // would result in.
    //
    // Earlier versions of this crate moved the contents of the directory being
    // deleted to become siblings of `base_dir`, which required write access to
    // the parent directory under all circumstances; this is no longer required
    // - though it may be re-instated if in-use files turn out to be handled
    //   very poorly with this new threaded implementation.
    //
    // There is a single small race condition where external side effects may be
    // left: when deleting a hard linked readonly file, the syscalls required
    // are:
    // - open
    // - set rw
    // - unlink (SetFileDispositionDelete)
    // - set ro
    //
    // A crash or power failure could lead to the loss of the readonly bit on
    // the hardlinked inode.
    //
    // To handle files with names like `CON` and `morse .. .`,  and when a
    // directory structure is so deep it needs long path names the path is first
    // converted to the Win32 file namespace by calling `canonicalize()`.

    let path = _remove_dir_contents(path)?;
    let metadata = path.metadata()?;
    if metadata.permissions().readonly() {
        delete_readonly(metadata, &path)?;
    } else {
        log::trace!("removing {}", &path.display());
        fs::remove_dir(&path).map_err(|e| {
            log::debug!("error removing {}", &path.display());
            e
        })?;
        log::trace!("removed {}", &path.display());
    }
    Ok(())
}

/// Returns the canonicalised path, for one of caller's convenience.
pub fn _remove_dir_contents<P: AsRef<Path>>(path: P) -> io::Result<PathBuf> {
    let path = path.as_ref().canonicalize()?;
    _delete_dir_contents(&path)?;
    Ok(path)
}

fn _delete_dir_contents(path: &PathBuf) -> io::Result<()> {
    log::trace!("scanning {}", &path.display());
    let iter = path.read_dir()?.par_bridge();
    iter.try_for_each(|dir_entry| -> io::Result<()> {
        let dir_entry = dir_entry?;
        let metadata = dir_entry.metadata()?;
        let is_dir = dir_entry.file_type()?.is_dir();
        let dir_path = dir_entry.path();
        if is_dir {
            _delete_dir_contents(&dir_path)?;
        }
        log::trace!("removing {}", &dir_path.display());
        if metadata.permissions().readonly() {
            delete_readonly(metadata, &dir_path).map_err(|e| {
                log::debug!("error removing {}", &dir_path.display());
                e
            })?;
        } else if is_dir {
            fs::remove_dir(&dir_path).map_err(|e| {
                log::debug!("error removing {}", &dir_path.display());
                e
            })?;
        } else {
            fs::remove_file(&dir_path).map_err(|e| {
                log::debug!("error removing {}", &dir_path.display());
                e
            })?;
        }
        log::trace!("removed {}", &dir_path.display());
        Ok(())
    })?;
    log::trace!("scanned {}", &path.display());
    Ok(())
}

// Delete a file or directory that is readonly
fn delete_readonly(metadata: fs::Metadata, path: &Path) -> io::Result<()> {
    // Open, drop the readonly bit, set delete-on-close, close.
    let mut opts = OpenOptions::new();
    opts.access_mode(DELETE | FILE_READ_ATTRIBUTES | FILE_WRITE_ATTRIBUTES);
    opts.custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT);

    let file = opts.open(path)?;
    let mut perms = metadata.permissions();
    perms.set_readonly(false);
    file.set_permissions(perms)?;

    let mut info = FILE_DISPOSITION_INFO {
        DeleteFile: TRUE as u8,
    };
    let result = unsafe {
        SetFileInformationByHandle(
            file.as_raw_handle(),
            FileDispositionInfo,
            &mut info as *mut FILE_DISPOSITION_INFO as LPVOID,
            size_of::<FILE_DISPOSITION_INFO>() as u32,
        )
    };

    if result == 0 {
        return Err(io::Error::last_os_error());
    }

    file.set_permissions(metadata.permissions())?;
    drop(file);
    Ok(())
}
