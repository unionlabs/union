use crate::error::{Error, Result};
use crate::gen::fs;
use crate::paths;
use std::io;
use std::path::Path;

pub(crate) fn write(path: impl AsRef<Path>, content: &[u8]) -> Result<()> {
    let path = path.as_ref();

    let mut create_dir_error = None;
    if fs::exists(path) {
        if let Ok(existing) = fs::read(path) {
            if existing == content {
                // Avoid bumping modified time with unchanged contents.
                return Ok(());
            }
        }
        best_effort_remove(path);
    } else {
        let parent = path.parent().unwrap();
        create_dir_error = fs::create_dir_all(parent).err();
    }

    match fs::write(path, content) {
        // As long as write succeeded, ignore any create_dir_all error.
        Ok(()) => Ok(()),
        // If create_dir_all and write both failed, prefer the first error.
        Err(err) => Err(Error::Fs(create_dir_error.unwrap_or(err))),
    }
}

pub(crate) fn symlink_file(original: impl AsRef<Path>, link: impl AsRef<Path>) -> Result<()> {
    let original = original.as_ref();
    let link = link.as_ref();

    let mut create_dir_error = None;
    if fs::exists(link) {
        best_effort_remove(link);
    } else {
        let parent = link.parent().unwrap();
        create_dir_error = fs::create_dir_all(parent).err();
    }

    match paths::symlink_or_copy(original, link) {
        // As long as symlink_or_copy succeeded, ignore any create_dir_all error.
        Ok(()) => Ok(()),
        Err(err) => {
            if err.kind() == io::ErrorKind::AlreadyExists {
                // This is fine, a different simultaneous build script already
                // created the same link or copy. The cxx_build target directory
                // is laid out such that the same path never refers to two
                // different targets during the same multi-crate build, so if
                // some other build script already created the same path then we
                // know it refers to the identical target that the current build
                // script was trying to create.
                Ok(())
            } else {
                // If create_dir_all and symlink_or_copy both failed, prefer the
                // first error.
                Err(Error::Fs(create_dir_error.unwrap_or(err)))
            }
        }
    }
}

pub(crate) fn symlink_dir(original: impl AsRef<Path>, link: impl AsRef<Path>) -> Result<()> {
    let original = original.as_ref();
    let link = link.as_ref();

    let mut create_dir_error = None;
    if fs::exists(link) {
        best_effort_remove(link);
    } else {
        let parent = link.parent().unwrap();
        create_dir_error = fs::create_dir_all(parent).err();
    }

    match fs::symlink_dir(original, link) {
        // As long as symlink_dir succeeded, ignore any create_dir_all error.
        Ok(()) => Ok(()),
        // If create_dir_all and symlink_dir both failed, prefer the first error.
        Err(err) => Err(Error::Fs(create_dir_error.unwrap_or(err))),
    }
}

fn best_effort_remove(path: &Path) {
    use std::fs;

    if cfg!(windows) {
        // On Windows, the correct choice of remove_file vs remove_dir needs to
        // be used according to what the symlink *points to*. Trying to use
        // remove_file to remove a symlink which points to a directory fails
        // with "Access is denied".
        if let Ok(metadata) = fs::metadata(path) {
            if metadata.is_dir() {
                let _ = fs::remove_dir_all(path);
            } else {
                let _ = fs::remove_file(path);
            }
        } else if fs::symlink_metadata(path).is_ok() {
            // The symlink might exist but be dangling, in which case there is
            // no standard way to determine what "kind" of symlink it is. Try
            // deleting both ways.
            if fs::remove_dir_all(path).is_err() {
                let _ = fs::remove_file(path);
            }
        }
    } else {
        // On non-Windows, we check metadata not following symlinks. All
        // symlinks are removed using remove_file.
        if let Ok(metadata) = fs::symlink_metadata(path) {
            if metadata.is_dir() {
                let _ = fs::remove_dir_all(path);
            } else {
                let _ = fs::remove_file(path);
            }
        }
    }
}
