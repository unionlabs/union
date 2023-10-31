use crate::error::Result;
use crate::gen::fs;
use std::ffi::OsStr;
use std::path::{Component, Path, PathBuf};

pub(crate) fn manifest_dir() -> Result<PathBuf> {
    crate::env_os("CARGO_MANIFEST_DIR").map(PathBuf::from)
}

pub(crate) fn out_dir() -> Result<PathBuf> {
    crate::env_os("OUT_DIR").map(PathBuf::from)
}

// Given a path provided by the user, determines where generated files related
// to that path should go in our out dir. In particular we don't want to
// accidentally write generated code upward of our out dir, even if the user
// passed a path containing lots of `..` or an absolute path.
pub(crate) fn local_relative_path(path: &Path) -> PathBuf {
    let mut rel_path = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(_) | Component::RootDir | Component::CurDir => {}
            Component::ParentDir => drop(rel_path.pop()), // noop if empty
            Component::Normal(name) => rel_path.push(name),
        }
    }
    rel_path
}

pub(crate) trait PathExt {
    fn with_appended_extension(&self, suffix: impl AsRef<OsStr>) -> PathBuf;
}

impl PathExt for Path {
    fn with_appended_extension(&self, suffix: impl AsRef<OsStr>) -> PathBuf {
        let mut file_name = self.file_name().unwrap().to_owned();
        file_name.push(suffix);
        self.with_file_name(file_name)
    }
}

#[cfg(unix)]
pub(crate) use self::fs::symlink_file as symlink_or_copy;

#[cfg(windows)]
pub(crate) fn symlink_or_copy(
    original: impl AsRef<Path>,
    link: impl AsRef<Path>,
) -> fs::Result<()> {
    // Pre-Windows 10, symlinks require admin privileges. Since Windows 10, they
    // require Developer Mode. If it fails, fall back to copying the file.
    let original = original.as_ref();
    let link = link.as_ref();
    if fs::symlink_file(original, link).is_err() {
        fs::copy(original, link)?;
    }
    Ok(())
}

#[cfg(not(any(unix, windows)))]
pub(crate) fn symlink_or_copy(
    original: impl AsRef<Path>,
    copy: impl AsRef<Path>,
) -> fs::Result<()> {
    fs::copy(original, copy)?;
    Ok(())
}
