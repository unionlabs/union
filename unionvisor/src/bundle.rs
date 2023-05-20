use color_eyre::Result;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::{fs, io};
use tracing::error;
use tracing::{debug, field::display as as_display, warn};

/// Bundles should have the following structure on the filesystem:
/// ```text
/// bins
/// └── v0.5.0
///     └── uniond
/// ```
pub struct Bundle {
    /// The home of the bundle
    home: PathBuf,
    /// The directory containing all versions' directories
    versions_dir: PathBuf,
    current: String,
    /// The name of the binary that lives in each version's directory
    binary_name: OsString,
}

pub enum BinaryAvailability {
    NotFound,
    PermissionDenied,
    Ok,
}

impl Bundle {
    /// Creates a new bindir. If a symlink named "current" is present, no further action is taken. Otherwise
    /// `name` is symlinked to "bins/current".
    pub fn new(
        home: impl Into<PathBuf>,
        versions_dir: impl Into<PathBuf>,
        name: &str,
        binary_name: impl Into<OsString>,
    ) -> Result<Self> {
        let bundle = Bundle {
            home: home.into(),
            versions_dir: versions_dir.into(),
            current: "current".to_owned(),
            binary_name: binary_name.into(),
        };

        // If there exists no symlink to current yet, we create it.
        match fs::read_link(bundle.current()) {
            Err(err) => match err.kind() {
                io::ErrorKind::NotFound => bundle.swap(name).map_err(|err| {
                    warn!(target: "unionvisor", "unable to swap fallback binary to current");
                    err
                })?,
                _ => return Err(err.into()),
            },
            Ok(path) => {
                debug!(target: "unionvisor", "existing symlink found at {}, pointing to {}, continuing using that", bundle.current().display(), path.display())
            }
        }
        Ok(bundle)
    }

    /// Returns the path to the current binary and checks if it is executable.
    pub fn current_checked(&self) -> Result<PathBuf> {
        let current = self.current();
        is_available_logged(&current, &self.versions_dir)?;
        Ok(current)
    }

    /// Checks if the program with `name` is available in the bindir and runnable.
    pub fn is_available(&self, name: &str) -> Result<BinaryAvailability> {
        let path = self.get_path_to(name);
        is_available_logged(path, &self.versions_dir)
    }

    pub fn current(&self) -> PathBuf {
        self.home.join(&self.current)
    }

    /// Swaps the symlink of the current binary with the binary associated with `name`.
    pub fn swap(&self, name: &str) -> Result<()> {
        let old = self.current();

        if old.exists() {
            debug!(target: "unionvisor", "removing old symlink at {}", as_display(old.display()));
            std::fs::remove_file(old)?;
        }

        let new = self.get_path_to(name);
        let to = self.current();
        debug!(target: "unionvisor", "creating symlink from {} to {}", as_display(new.display()), as_display(to.display()));
        // soft_link is deprecated as it does not work on windows, but we do not care about
        // windows anyway.
        std::os::unix::fs::symlink(new, to)?;
        Ok(())
    }

    /// Obtains the path to the binary within the bindir with name `name`.
    pub fn get_path_to(&self, name: &str) -> PathBuf {
        self.versions_dir.join(name).join(&self.binary_name)
    }
}

/// Checks if the program with `name` is available in the bindir and runnable, and emits appropriate logs on failures.
fn is_available_logged(
    path: impl AsRef<Path>,
    bundle: impl AsRef<Path>,
) -> Result<BinaryAvailability> {
    let path = path.as_ref();
    let bundle = bundle.as_ref();
    let status = is_available(path, bundle)?;

    match status {
        BinaryAvailability::NotFound => {
            error!(target: "unionvisor", "could not find binary {} in {}", path.display(), bundle.display())
        }
        BinaryAvailability::PermissionDenied => {
            error!(target: "unionvisor", "could not execute binary {} in {}", path.display(), bundle.display())
        }
        BinaryAvailability::Ok => (),
    }
    Ok(status)
}
/// Checks if the program with `name` is available in the bindir and runnable.
fn is_available(path: impl AsRef<Path>, bindir: impl AsRef<Path>) -> Result<BinaryAvailability> {
    let path = path.as_ref();
    let bindir = bindir.as_ref();
    debug!(
        bindir = as_display(bindir.display()),
        "testing if binary {} is available by calling --help",
        as_display(path.display())
    );
    let mut child = Command::new(path)
        .arg("--help")
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .spawn()?;
    debug!(target: "unionvisor", "killing test call of {}", as_display(path.display()));
    if let Err(err) = child.kill() {
        match err.kind() {
            io::ErrorKind::NotFound => return Ok(BinaryAvailability::NotFound),
            io::ErrorKind::PermissionDenied => return Ok(BinaryAvailability::PermissionDenied),
            _ => return Err(err.into()),
        }
    }
    Ok(BinaryAvailability::Ok)
}

#[cfg(test)]
mod tests {
    use crate::testdata;

    use super::*;

    #[test]
    fn test_swap() {
        let dir = testdata::temp_dir_with(&["test_swap"]);
        let home = dir.into_path().join("test_swap");

        std::os::unix::fs::symlink(home.join("bins/bar/uniond"), home.join("bins/current"))
            .expect("should be able to symlink");

        let bindir = Bundle::new(home.clone(), home.join("bins"), "bar", "uniond")
            .expect("should be able to create a bindir");
        bindir.swap("foo").unwrap();
    }
}
