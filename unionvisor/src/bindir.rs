use color_eyre::Result;
use fs_extra::error;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::{fs, io};
use tracing::{debug, warn, field::display as as_display};

pub struct Bindir {
    home: PathBuf,
    bindir: PathBuf,
    current: String,
    binary_name: OsString,
}

impl Bindir {
    /// Creates a new bindir. If a symlink named "current" is present, no further action is taken. Otherwise
    /// `name` is symlinked to "bins/current".
    pub fn new(
        home: impl Into<PathBuf>,
        bindir: impl Into<PathBuf>,
        name: &str,
        binary_name: impl Into<OsString>,
    ) -> Result<Self> {
        let dir = Bindir {
            home: home.into(),
            bindir: bindir.into(),
            current: "current".to_owned(),
            binary_name: binary_name.into(),
        };

        // If there exists no symlink to current yet, we create it.
        if let Err(err) = fs::read_link(dir.current()) {
            match err.kind() {
                io::ErrorKind::NotFound => dir.swap(name).map_err(|err| {
                    warn!(target: "unionvisor", "unable to swap fallback binary to current");
                    err
                })?,
                _ => return Err(err.into()),
            }
        }
        Ok(dir)
    }

    /// Checks if the program with `name` is available in the bindir and runnable.
    pub fn is_available(&self, name: &str) -> Result<bool> {
        let path = self.get_path_to(name);
        debug!(
            bindir = as_display(self.bindir.display()),
            "testing if binary {} is available by calling --help",
            as_display(path.display())
        );
        let mut child = Command::new(&path)
            .arg("--help")
            .stderr(Stdio::null())
            .stdout(Stdio::null())
            .spawn()?;
        debug!(target: "unionvisor", "killing test call of {}", as_display(path.display()));
        child.kill()?;
        Ok(true)
    }

    pub fn current(&self) -> PathBuf {
        self.home.join(&self.current)
    }

    /// Swaps the symlink of the current binary with the binary associated with `name`.
    pub fn swap(&self, name: &str) -> Result<()> {
        let old = self.current();
        debug!(target: "unionvisor", "removing old symlink at {}", as_display(old.display()));
        if let Err(err) = std::fs::remove_file(old) {
            if err.kind() != io::ErrorKind::NotFound {
                return Err(err.into());
            }
        };
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
        self.bindir.join(name).join(&self.binary_name)
    }
}

#[cfg(test)]
mod tests {
    use crate::testdata;

    use super::*;

    #[test]
    fn test_swap() {
        let dir = testdata::temp_dir_with(&["test_swap"]);
        let home = dir.into_path().join("test_swap");

        std::os::unix::fs::symlink(home.join("bins/bar.foo"), home.join("bins/current"))
            .expect("should be able to symlink");

        let bindir = Bindir::new(home.clone(), home.join("bins"), "bar.foo", "").expect("should be able to create a bindir");
        bindir.swap("foo.bar").unwrap();
    }
}
