use std::{ffi::OsString, fs, io, path::PathBuf};

use thiserror::Error;
use tracing::debug;

use crate::bundle::{Bundle, UnvalidatedVersionPath, ValidVersionPath, ValidateVersionPathError};

/// Symlinker maintains a symlink `root/current` to a binary at a [`Bundle`]'s [`ValidVersionPath`]
#[derive(Clone)]
pub struct Symlinker {
    pub root: PathBuf,
    pub bundle: Bundle,
}

#[derive(Error, Debug)]
pub enum SymlinkerError {
    #[error("cannot remove old symlink: {0}")]
    CantRemoveSymlink(io::Error),
    #[error("cannot create symlink: {0}")]
    CantCreateSymlink(io::Error),
}

impl Symlinker {
    /// Constructs a new [`Symlinker`].
    ///
    /// # Arguments
    ///
    /// * `root` - The path where the `root` symlink should be put
    /// * `bundle` - The [`Bundle`] containing binaries to which `root/current` can point
    pub fn new(root: impl Into<PathBuf>, bundle: Bundle) -> Self {
        let root = root.into();
        Self { root, bundle }
    }

    /// Swaps the `root/current` symlink to point to `bundle/meta.versions_dir/new_version/meta.binary_name`
    /// where meta = Symlinker.bundle.meta
    ///
    /// # Arguments
    ///
    /// * `new_version` the new version the symlink should point to
    pub fn swap(&self, new_version: impl Into<OsString>) -> Result<(), SymlinkerError> {
        let new_version = new_version.into();
        let new_path = self.bundle.path_to(new_version).validate()?;
        let current = self.current_path();

        if current.exists() {
            debug!(target: "unionvisor", "removing old symlink at {}", &current.display());
            std::fs::remove_file(&current).map_err(SymlinkerError::CantRemoveSymlink)?;
        }

        debug!(target: "unionvisor", "creating symlink from {} to {}", &current.display(), new_path.0.display());
        std::os::unix::fs::symlink(new_path.0, current)
            .map_err(SymlinkerError::CantCreateSymlink)?;

        Ok(())
    }

    /// Creates a link `root/current` that points to `bundle/meta.versions_dir/meta.fallback_version/meta.binary_name`
    /// where meta = Symlinker.bundle.meta
    ///
    /// Note that this initial link is unvalidated.
    pub fn make_fallback_link(&self) -> Result<()> {
        let fallback_path = self.bundle.fallback_path()?;
        let current = self.current_path();

        debug!(target: "unionvisor", "creating fallback symlink from {} to {}", current.display(), fallback_path.0.display());
        std::os::unix::fs::symlink(fallback_path.0, current)?;

        Ok(())
    }

    /// Only used by the `Symlinker` internally. Consumers of the current link should use [`current_validated`]
    fn current_path(&self) -> PathBuf {
        self.root.join("current")
    }

    /// Returns a [`ValidVersionPath`] if teh `current` symlink is valid.
    pub fn current_validated(&self) -> Result<ValidVersionPath, ValidateVersionPathError> {
        UnvalidatedVersionPath::new(self.current_path()).validate()
    }

    /// Reads the `root/current` link and determines the binary version based on the path.
    pub fn current_version(&self) -> Result<OsString, CurrentVersionError> {
        use CurrentVersionError::*;
        let version_in_bundle = fs::read_link(self.current_path()).map_err(ReadLink)?;
        let mut actual = version_in_bundle.clone();

        actual.pop(); // pop meta.binary_name (such as `uniond`) from the path
        let version = actual.file_name();

        match version {
            None => Err(InvalidBundleStructure(version_in_bundle)),
            Some(v) => Ok(v.to_os_string()),
        }
    }
}

#[derive(Debug, Error)]
enum CurrentVersionError {
    #[error("cannot read current link")]
    ReadLink(#[source] io::Error),
    #[error("invalid bundle structure: binary parent directory is not a version {0}")]
    InvalidBundleStructure(PathBuf),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testdata;

    #[test]
    fn test_swap() {
        let tmp = testdata::temp_dir_with(&["test_swap"]);
        let root = tmp.into_path().join("test_swap");
        let bundle = Bundle::new(root.join("bundle")).unwrap();
        let symlinker = Symlinker::new(root, bundle);

        symlinker
            .make_fallback_link()
            .expect("fallback link should be made");

        symlinker.swap("foo").unwrap();
    }
}
