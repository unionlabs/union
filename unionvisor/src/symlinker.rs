use std::path::PathBuf;

use crate::bundle::{Bundle, UnvalidatedVersionPath, ValidVersionPath};
use color_eyre::Result;
use std::ffi::OsString;
use tracing::debug;

/// Symlinker will maintain a symlink `root/current` to point to a [`Bundle`]'s [`ValidVersionPath`]
pub struct Symlinker {
    root: PathBuf,
    bundle: Bundle,
}

impl Symlinker {
    pub fn new(root: PathBuf, bundle: Bundle) -> Self {
        Self { root, bundle }
    }

    pub fn swap(&self, new_version: &OsString) -> Result<()> {
        let new_path = self.bundle.path_to(new_version).validate()?;
        let current = self.current_path();

        if current.exists() {
            debug!(target: "unionvisor", "removing old symlink at {}", current.display());
            std::fs::remove_file(current)?;
        }

        debug!(target: "unionvisor", "creating symlink from {} to {}", current.display(), new_path.0.display());
        std::os::unix::fs::symlink(new_path.0, current);

        Ok(())
    }

    pub fn make_fallback_link(&self) -> Result<()> {
        let fallback_path = self.bundle.fallback_path()?;
        let current = self.current_path();

        debug!(target: "unionvisor", "creating fallback symlink from {} to {}", current.display(), fallback_path.0.display());
        std::os::unix::fs::symlink(fallback_path.0, current);

        Ok(())
    }

    /// Only used by the `Symlinker` internally. Consumers of the current link should use [`current_validated`]
    fn current_path(&self) -> PathBuf {
        self.root.join("current")
    }

    // pub fn previous_path(&self) -> PathBuf {
    //     self.root.join("previous")
    // }

    pub fn current_validated(&self) -> Result<ValidVersionPath> {
        UnvalidatedVersionPath::new(self.current_path()).validate()
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::testdata;

//     use super::*;

//     #[test]
//     fn test_swap() {
//         let dir = testdata::temp_dir_with(&["test_swap"]);
//         let home = dir.into_path().join("test_swap");

//         std::os::unix::fs::symlink(home.join("bins/bar/uniond"), home.join("bins/current"))
//             .expect("should be able to symlink");

//         let bindir = Bundle::new(home.clone(), home.join("bins"), "bar", "uniond")
//             .expect("should be able to create a bindir");
//         bindir.swap("foo").unwrap();
//     }
// }
