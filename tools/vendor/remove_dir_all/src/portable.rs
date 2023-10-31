use std::io;
use std::path::Path;

#[cfg(windows)]
use crate::fs::_remove_dir_contents;

#[cfg(not(windows))]
use crate::unix::_remove_dir_contents;

/// Deletes the contents of `path`, but not the directory iteself.
///
/// If `path` is a symlink to a directory, deletes the contents
/// of that directory.  Fails if `path` does not exist.
pub fn remove_dir_contents<P: AsRef<Path>>(path: P) -> io::Result<()> {
    // This wrapper function exists because the core function
    // for Windows, in crate::fs, returns a PathBuf, which our
    // caller shouldn't see.
    _remove_dir_contents(path)?;
    Ok(())
}

/// Makes `path` an empty directory: if it does not exist, it is
/// created it as an empty directory (as if with
/// `std::fs::create_dir`); if it does exist, its contents are
/// deleted (as if with `remove_dir_contents`).
///
/// It is an error if `path` exists but is not a directory (or
/// a symlink to one).
pub fn ensure_empty_dir<P: AsRef<Path>>(path: P) -> io::Result<()> {
    match std::fs::create_dir(&path) {
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists
            => remove_dir_contents(path),
        otherwise => otherwise,
    }
}

#[cfg(test)]
mod test {
    use tempfile::TempDir;
    use crate::remove_dir_all;
    use crate::remove_dir_contents;
    use crate::ensure_empty_dir;
    use std::fs::{self, File};
    use std::path::PathBuf;
    use std::io;

    fn expect_failure<T>(k: io::ErrorKind, r: io::Result<T>) -> io::Result<()> {
        match r {
            Err(e) if e.kind() == k => Ok(()),
            Err(e) => Err(e),
            Ok(_) => Err(io::Error::new(
                io::ErrorKind::Other,
                "unexpected success".to_string(),
            )),
        }
    }

    struct Prep {
        _tmp: TempDir,
        ours: PathBuf,
        file: PathBuf,
    }

    fn prep() -> Result<Prep, io::Error> {
        let tmp = TempDir::new()?;
        let ours = tmp.path().join("t.mkdir");
        let file = ours.join("file");
        fs::create_dir(&ours)?;
        File::create(&file)?;
        File::open(&file)?;
        Ok(Prep { _tmp: tmp, ours, file })
    }

    #[test]
    fn mkdir_rm() -> Result<(), io::Error> {
        let p = prep()?;

        expect_failure(io::ErrorKind::Other, remove_dir_contents(&p.file))?;

        remove_dir_contents(&p.ours)?;
        expect_failure(io::ErrorKind::NotFound, File::open(&p.file))?;

        remove_dir_contents(&p.ours)?;
        remove_dir_all(&p.ours)?;
        expect_failure(io::ErrorKind::NotFound, remove_dir_contents(&p.ours))?;
        Ok(())
    }

    #[test]
    fn ensure_rm() -> Result<(), io::Error> {
        let p = prep()?;

        expect_failure(io::ErrorKind::Other, ensure_empty_dir(&p.file))?;

        ensure_empty_dir(&p.ours)?;
        expect_failure(io::ErrorKind::NotFound, File::open(&p.file))?;
        ensure_empty_dir(&p.ours)?;

        remove_dir_all(&p.ours)?;
        ensure_empty_dir(&p.ours)?;
        File::create(&p.file)?;

        Ok(())
    }
}
