use color_eyre::Result;
use serde::{Deserialize, Serialize};
use std::ffi::OsString;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::{fs, io};
use tracing::error;
use tracing::{debug, field::display as as_display, warn};

/// Bundles should have the following structure on the filesystem:
/// ```text
/// result
/// ├── meta.json
/// └── versions
///     └── v0.5.0
///         └── uniond
/// ```
pub struct Bundle {
    /// The path of the bundle
    path: PathBuf,
    /// The deserialized meta info from `bundle/meta.json`
    meta: BundleMeta,
}

/// Version paths that have not been validated
pub struct UnvalidatedVersionPath(PathBuf);

/// Version paths that have been validated.
/// This means that the binary at this path produces valid output when called with --help
pub struct ValidVersionPath(PathBuf);

impl UnvalidatedVersionPath {
    fn validate(&self) -> Result<ValidVersionPath> {
        self.is_available_logged().map(|_| ValidVersionPath(self.0))
    }

    fn is_available_logged(&self) -> Result<BinaryAvailability> {
        let status = self.is_available()?;

        match status {
            BinaryAvailability::NotFound => {
                error!(target: "unionvisor", "could not find binary {} in bundle", self.0.display())
            }
            BinaryAvailability::PermissionDenied => {
                error!(target: "unionvisor", "could not execute binary {} in bundle", self.0.display())
            }
            BinaryAvailability::Ok => (),
        }
        Ok(status)
    }

    fn is_available(&self) -> Result<BinaryAvailability> {
        debug!(
            "testing if binary {} is available by calling --help",
            as_display(self.0.display())
        );
        let mut child = Command::new(&self.0)
            .arg("--help")
            .stderr(Stdio::null())
            .stdout(Stdio::null())
            .spawn()?;
        debug!(target: "unionvisor", "killing test call of {}", as_display(self.0.display()));
        if let Err(err) = child.kill() {
            match err.kind() {
                io::ErrorKind::NotFound => return Ok(BinaryAvailability::NotFound),
                io::ErrorKind::PermissionDenied => return Ok(BinaryAvailability::PermissionDenied),
                _ => return Err(err.into()),
            }
        }
        Ok(BinaryAvailability::Ok)
    }
}

/// Bundle meta info found in `bundle/meta.json`
#[derive(Serialize, Deserialize)]
pub struct BundleMeta {
    /// The name of the binary in `bundle/bins/$VERSION/`
    binary_name: OsString,
    /// The fallback version directory in `bundle/bins/`
    fallback_version: OsString,
    /// The directory containing a directory for each version
    versions_directory: PathBuf,
}

pub enum BinaryAvailability {
    NotFound,
    PermissionDenied,
    Ok,
}

impl Bundle {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self> {
        let path: PathBuf = path.into();

        // Read `bundle/meta.json` and deserialize into `BundleMeta`
        let meta = path.join("meta");
        let meta = fs::read_to_string(meta)?;
        let meta = serde_json::from_str(&meta)?;

        let bundle = Bundle { path, meta };

        Ok(bundle)
    }

    /// Obtains the path to the binary within the bundle with version `version`.
    pub fn path_to(&self, version: &str) -> UnvalidatedVersionPath {
        UnvalidatedVersionPath(
            self.path
                .join(&self.meta.versions_directory)
                .join(version)
                .join(&self.meta.binary_name),
        )
    }

    // pub fn current_checked(&self) -> Result<PathBuf> {
    //     let current = self.current();
    //     is_available_logged(&current, &self.versions_dir)?;
    //     Ok(current)
    // }

    // /// Swaps the symlink of the current binary with the binary associated with `name`.
    // pub fn swap(&self, name: &str) -> Result<()> {
    //     let old = self.current();

    //     if old.exists() {
    //         debug!(target: "unionvisor", "removing old symlink at {}", as_display(old.display()));
    //         std::fs::remove_file(old)?;
    //     }

    //     let new = self.get_path_to(name);
    //     let to = self.current();
    //     debug!(target: "unionvisor", "creating symlink from {} to {}", as_display(new.display()), as_display(to.display()));
    //     // soft_link is deprecated as it does not work on windows, but we do not care about
    //     // windows anyway.
    //     std::os::unix::fs::symlink(new, to)?;
    //     Ok(())
    // }
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
