use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::os::windows::prelude::*;
use std::path::{Path, PathBuf};
use std::{io, ptr};

use winapi::shared::minwindef::*;
use winapi::shared::winerror::*;
use winapi::um::errhandlingapi::*;
use winapi::um::fileapi::*;
use winapi::um::minwinbase::*;
use winapi::um::winbase::*;
use winapi::um::winnt::*;

pub const VOLUME_NAME_DOS: DWORD = 0x0;

struct RmdirContext<'a> {
    base_dir: &'a Path,
    readonly: bool,
    counter: u64,
}

/// Reliably removes a directory and all of its children.
///
/// ```rust
/// extern crate remove_dir_all;
///
/// use std::fs;
/// use remove_dir_all::*;
///
/// fn main() {
///     fs::create_dir("./temp/").unwrap();
///     remove_dir_all("./temp/").unwrap();
/// }
/// ```
pub fn remove_dir_all<P: AsRef<Path>>(path: P) -> io::Result<()> {
    // On Windows it is not enough to just recursively remove the contents of a
    // directory and then the directory itself. Deleting does not happen
    // instantaneously, but is scheduled.
    // To work around this, we move the file or directory to some `base_dir`
    // right before deletion to avoid races.
    //
    // As `base_dir` we choose the parent dir of the directory we want to
    // remove. We very probably have permission to create files here, as we
    // already need write permission in this dir to delete the directory. And it
    // should be on the same volume.
    //
    // To handle files with names like `CON` and `morse .. .`,  and when a
    // directory structure is so deep it needs long path names the path is first
    // converted to a `//?/`-path with `get_path()`.
    //
    // To make sure we don't leave a moved file laying around if the process
    // crashes before we can delete the file, we do all operations on an file
    // handle. By opening a file with `FILE_FLAG_DELETE_ON_CLOSE` Windows will
    // always delete the file when the handle closes.
    //
    // All files are renamed to be in the `base_dir`, and have their name
    // changed to "rm-<counter>". After every rename the counter is increased.
    // Rename should not overwrite possibly existing files in the base dir. So
    // if it fails with `AlreadyExists`, we just increase the counter and try
    // again.
    //
    // For read-only files and directories we first have to remove the read-only
    // attribute before we can move or delete them. This also removes the
    // attribute from possible hardlinks to the file, so just before closing we
    // restore the read-only attribute.
    //
    // If 'path' points to a directory symlink or junction we should not
    // recursively remove the target of the link, but only the link itself.
    //
    // Moving and deleting is guaranteed to succeed if we are able to open the
    // file with `DELETE` permission. If others have the file open we only have
    // `DELETE` permission if they have specified `FILE_SHARE_DELETE`. We can
    // also delete the file now, but it will not disappear until all others have
    // closed the file. But no-one can open the file after we have flagged it
    // for deletion.

    // Open the path once to get the canonical path, file type and attributes.
    let (path, metadata) = {
        let path = path.as_ref();
        let mut opts = OpenOptions::new();
        opts.access_mode(FILE_READ_ATTRIBUTES);
        opts.custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT);
        let file = opts.open(path)?;
        (get_path(&file)?, path.metadata()?)
    };

    let mut ctx = RmdirContext {
        base_dir: match path.parent() {
            Some(dir) => dir,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "Can't delete root directory",
                ))
            }
        },
        readonly: metadata.permissions().readonly(),
        counter: 0,
    };

    let filetype = metadata.file_type();
    if filetype.is_dir() {
        if !filetype.is_symlink() {
            remove_dir_all_recursive(path.as_ref(), &mut ctx)
        } else {
            remove_item(path.as_ref(), &mut ctx)
        }
    } else {
        Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "Not a directory",
        ))
    }
}

fn remove_item(path: &Path, ctx: &mut RmdirContext) -> io::Result<()> {
    if ctx.readonly {
        // remove read-only permision
        let mut permissions = path.metadata()?.permissions();
        permissions.set_readonly(false);

        fs::set_permissions(path, permissions)?;
    }

    let mut opts = OpenOptions::new();
    opts.access_mode(DELETE);
    opts.custom_flags(
        FILE_FLAG_BACKUP_SEMANTICS | // delete directory
                        FILE_FLAG_OPEN_REPARSE_POINT | // delete symlink
                        FILE_FLAG_DELETE_ON_CLOSE,
    );
    let file = opts.open(path)?;
    move_item(&file, ctx)?;

    if ctx.readonly {
        // restore read-only flag just in case there are other hard links
        match fs::metadata(&path) {
            Ok(metadata) => {
                let mut perm = metadata.permissions();
                perm.set_readonly(true);
                fs::set_permissions(&path, perm)?;
            }
            Err(ref err) if err.kind() == io::ErrorKind::NotFound => {}
            err => return err.map(|_| ()),
        }
    }

    Ok(())
}

fn move_item(file: &File, ctx: &mut RmdirContext) -> io::Result<()> {
    let mut tmpname = ctx.base_dir.join(format! {"rm-{}", ctx.counter});
    ctx.counter += 1;

    // Try to rename the file. If it already exists, just retry with an other
    // filename.
    while let Err(err) = rename(file, &tmpname, false) {
        if err.kind() != io::ErrorKind::AlreadyExists {
            return Err(err);
        };
        tmpname = ctx.base_dir.join(format!("rm-{}", ctx.counter));
        ctx.counter += 1;
    }

    Ok(())
}

fn rename(file: &File, new: &Path, replace: bool) -> io::Result<()> {
    // &self must be opened with DELETE permission
    use std::iter;
    #[cfg(target_pointer_width = "32")]
    const STRUCT_SIZE: usize = 12;
    #[cfg(target_pointer_width = "64")]
    const STRUCT_SIZE: usize = 20;

    // FIXME: check for internal NULs in 'new'
    let mut data: Vec<u16> = iter::repeat(0u16)
        .take(STRUCT_SIZE / 2)
        .chain(new.as_os_str().encode_wide())
        .collect();
    data.push(0);
    let size = data.len() * 2;

    unsafe {
        // Thanks to alignment guarantees on Windows this works
        // (8 for 32-bit and 16 for 64-bit)
        let info = data.as_mut_ptr() as *mut FILE_RENAME_INFO;
        // The type of ReplaceIfExists is BOOL, but it actually expects a
        // BOOLEAN. This means true is -1, not c::TRUE.
        (*info).ReplaceIfExists = if replace { -1 } else { FALSE };
        (*info).RootDirectory = ptr::null_mut();
        (*info).FileNameLength = (size - STRUCT_SIZE) as DWORD;
        let result = SetFileInformationByHandle(
            file.as_raw_handle(),
            FileRenameInfo,
            data.as_mut_ptr() as *mut _ as *mut _,
            size as DWORD,
        );

        if result == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
}

fn get_path(f: &File) -> io::Result<PathBuf> {
    fill_utf16_buf(
        |buf, sz| unsafe { GetFinalPathNameByHandleW(f.as_raw_handle(), buf, sz, VOLUME_NAME_DOS) },
        |buf| PathBuf::from(OsString::from_wide(buf)),
    )
}

fn remove_dir_all_recursive(path: &Path, ctx: &mut RmdirContext) -> io::Result<()> {
    let dir_readonly = ctx.readonly;
    for child in fs::read_dir(path)? {
        let child = child?;
        let child_type = child.file_type()?;
        ctx.readonly = child.metadata()?.permissions().readonly();
        if child_type.is_dir() {
            remove_dir_all_recursive(&child.path(), ctx)?;
        } else {
            remove_item(&child.path().as_ref(), ctx)?;
        }
    }
    ctx.readonly = dir_readonly;
    remove_item(path, ctx)
}

fn fill_utf16_buf<F1, F2, T>(mut f1: F1, f2: F2) -> io::Result<T>
where
    F1: FnMut(*mut u16, DWORD) -> DWORD,
    F2: FnOnce(&[u16]) -> T,
{
    // Start off with a stack buf but then spill over to the heap if we end up
    // needing more space.
    let mut stack_buf = [0u16; 512];
    let mut heap_buf = Vec::new();
    unsafe {
        let mut n = stack_buf.len();

        loop {
            let buf = if n <= stack_buf.len() {
                &mut stack_buf[..]
            } else {
                let extra = n - heap_buf.len();
                heap_buf.reserve(extra);
                heap_buf.set_len(n);
                &mut heap_buf[..]
            };

            // This function is typically called on windows API functions which
            // will return the correct length of the string, but these functions
            // also return the `0` on error. In some cases, however, the
            // returned "correct length" may actually be 0!
            //
            // To handle this case we call `SetLastError` to reset it to 0 and
            // then check it again if we get the "0 error value". If the "last
            // error" is still 0 then we interpret it as a 0 length buffer and
            // not an actual error.
            SetLastError(0);
            let k = match f1(buf.as_mut_ptr(), n as DWORD) {
                0 if GetLastError() == 0 => 0,
                0 => return Err(io::Error::last_os_error()),
                n => n,
            } as usize;
            if k == n && GetLastError() == ERROR_INSUFFICIENT_BUFFER {
                n *= 2;
            } else if k >= n {
                n = k;
            } else {
                return Ok(f2(&buf[..k]));
            }
        }
    }
}
