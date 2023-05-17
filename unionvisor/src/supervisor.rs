use crate::{
    bindir::Bindir,
    logging::LogFormat,
    watcher::{FileReader, FileReaderError},
};
use color_eyre::{eyre::eyre, Result};
use std::{
    ffi::{OsStr, OsString},
    fs::{self, create_dir_all},
    path::{Path, PathBuf},
    process::{Child, ExitStatus},
    time::Duration,
};
use tracing::{debug, error, field::display as as_display, info, warn};

/// A process supervisor for the uniond binary, which can start, gracefully exit and backup uniond data.
pub struct Supervisor {
    /// The path where the subprocess is called, containing configuration files and more importantly, the
    /// data dir.
    root: PathBuf,

    /// The binary to run which will be supervised. This should be the name of the binary, which will be run by the
    /// supervisor as ./{binary}.
    binary: PathBuf,

    child: Option<Child>,
}

impl Supervisor {
    pub fn new(root: impl Into<PathBuf>, current: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            binary: current.into(),
            child: None,
        }
    }

    /// Starts running the uniond binary in non-blocking mode.
    pub fn spawn<S: AsRef<OsStr>, I: IntoIterator<Item = S>>(
        &mut self,
        logformat: LogFormat,
        args: I,
    ) -> Result<()> {
        assert!(&self.binary.exists());
        let handle = std::process::Command::new(&self.binary)
            .args(vec!["--log_format", logformat.as_str()])
            .arg("start")
            .args(args)
            .stderr(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::inherit())
            .spawn()?;
        self.child = Some(handle);
        Ok(())
    }

    /// Returns the {root}/{binary} symlink that is being used by this supervisor.
    pub fn symlink(&self) -> PathBuf {
        self.binary.clone()
    }

    fn data_dir(&self) -> PathBuf {
        self.root.join("data")
    }

    /// Backup the current data directory to the provided path. The location will be "{dir}/data".
    pub fn backup(&self, dir: impl AsRef<Path>) -> Result<()> {
        use fs_extra::dir::{copy, CopyOptions};
        let dir = dir.as_ref();
        debug!(target: "unionvisor", "creating backup dir at {}",  as_display(dir.display()));
        create_dir_all(dir)?;
        let data_dir = self.data_dir();
        let options = CopyOptions::new().overwrite(true);
        debug!(target: "unionvisor", "backing up {} to {}",  as_display(data_dir.display()),  as_display(dir.display()));
        copy(data_dir, dir, &options)?;
        Ok(())
    }

    /// Revert a backup at `dir`.
    pub fn revert(&self, dir: impl AsRef<Path>) -> Result<()> {
        use fs_extra::dir::{copy, CopyOptions};
        let dir = dir.as_ref();
        let options = CopyOptions::new().overwrite(true);
        copy(dir, &self.root, &options)?;
        Ok(())
    }

    pub fn try_wait(&mut self) -> Result<Option<ExitStatus>> {
        if let Some(child) = &mut self.child {
            Ok(child.try_wait()?)
        } else {
            unreachable!("try_waiting for a child should only happen after spawn")
        }
    }

    pub fn kill(&mut self) -> Result<()> {
        if let Some(ref mut child) = self.child {
            child.kill()?;
        } else {
            unreachable!("killing a child should only happen after spawn")
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("binary {} unavailable: {}", name, err)]
    BinaryUnavailable {
        name: String,
        err: color_eyre::Report,
    },
    #[error("early exit detected: {}", code)]
    EarlyExit { code: ExitStatus },
    #[error("{}", 0)]
    Other(#[from] color_eyre::Report),
}

pub fn run_and_upgrade<S: AsRef<OsStr>, I: IntoIterator<Item = S> + Clone>(
    home: impl Into<PathBuf>,
    logformat: LogFormat,
    bindir: Bindir,
    args: I,
    pol_interval: Duration,
) -> color_eyre::Result<(), RuntimeError> {
    let current = bindir.current();
    let home = home.into();
    let mut supervisor = Supervisor::new(home.clone(), current.clone());
    let mut watcher = FileReader::new(home.join("data/upgrade-info.json"));

    info!(target: "unionvisor", "spawning supervisor process for the uniond binary");
    debug!(
        target: "unionvisor",
        binary =  as_display(current.display()),
        home =  as_display(home.display()),
        "spawning supervisor process for the uniond binary"
    );
    supervisor
        .spawn(logformat.clone(), args.clone())
        .map_err(|err| {
            warn!(target: "supervisor", "failed to spawn initial binary call");
            err
        })?;
    std::thread::sleep(Duration::from_millis(300));
    loop {
        if let Some(code) = supervisor.try_wait()? {
            error!(target: "unionvisor", "uniond exited with code: {}", code);
            return Err(RuntimeError::EarlyExit { code });
        }

        match watcher.poll() {
            Err(FileReaderError::FileNotFound) => continue,
            Err(err) => {
                warn!(target: "supervisor", "unknown error while polling for upgrades: {}", err.to_string());
                return Err(RuntimeError::Other(err.into()));
            }
            Ok(None) => continue,
            Ok(Some(new)) => {
                // If the daemon restarts, then upgrade-info.json may be stale. We need to
                // resolve the current symlink and compare it with the info.
                let symlink = supervisor.symlink();
                let actual =
                    fs::read_link(symlink).map_err(|err| RuntimeError::Other(err.into()))?;
                let name = actual
                    .file_name()
                    .ok_or(RuntimeError::Other(eyre!("could not read symlink")))?;
                if name == OsString::from(&new.name) {
                    debug!(target: "unionvisor", "detected upgrade {}, but already running that binary. sleeping for {} milliseconds.", &new.name, pol_interval.as_millis());
                    std::thread::sleep(pol_interval);
                    continue;
                }

                info!(
                    target: "unionvisor",
                    name = new.name.as_str(),
                    height = new.height,
                "upgrade detected"
                );
                debug!(target: "unionvisor", "checking binary availability");
                bindir.is_available(&new.name).map_err(|err| {
                    error!(target: "unionvisor", "binary {} unavailable", &new.name);
                    RuntimeError::BinaryUnavailable {
                        err,
                        name: new.name.clone(),
                    }
                })?;
                debug!(target: "unionvisor", "killing supervisor process");
                supervisor.kill()?;
                let backup_file = home.join("backup");

                // If we fail to backup, the file system is incorrectly configured (permissions) or we are running
                // out of disk space. Either way we exit the node as now the server itself has become unreliable.
                debug!(target: "unionvisor", "backing up current database");
                supervisor.backup(&backup_file)?;

                debug!(target: "unionvisor", "creating new symlink for {}", &new.name);
                bindir.swap(&new.name)?;

                // Store the old supervisor incase of reverts.
                let old = supervisor;
                supervisor = Supervisor::new(home.clone(), current.clone());

                // If this upgrade fails, we'll revert the local DB and exit the node, ensuring we keep the filesystem in
                // the last correct state.
                debug!(target: "unionvisor", "spawning new supervisor process for {}", &new.name);
                supervisor.spawn(logformat.clone(), args.clone()).map_err(|err| {
                    error!(target: "unionvisor", err = err.to_string().as_str(), "spawning new supervisor process for {} failed", &new.name);
                    // An error here is uber fubar. 
                    if let Err(err) = old.revert(backup_file) {
                        error!(target: "unionvisor", err = err.to_string().as_str(), "reverting backup failed");
                        return err;
                    }
                    // This error is most likely caused by incorrect args because of an upgrade. We can reduce the chance of that happening
                    // by introducing a configuration file with name -> args mappings.
                    err
                })?;
            }
        }
        debug!(target: "unionvisor", "no upgrade detected, sleeping for {} milliseconds.", &pol_interval.as_millis());
        std::thread::sleep(pol_interval)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testdata;
    use std::fs;
    use tracing_test::traced_test;

    #[test]
    #[traced_test]
    fn test_run_and_upgrade() {
        let tmp_dir = testdata::temp_dir_with(&["test_run"]);
        let path = tmp_dir.into_path().join("test_run");
        let bindir = Bindir::new(path.clone(), path.join("bins"), "genesis", "uniond").unwrap();
        let err = run_and_upgrade(
            path.clone(),
            LogFormat::Plain,
            bindir,
            vec![path.join("data").as_os_str()],
            Duration::from_secs(1),
        )
        .unwrap_err();
        if let RuntimeError::BinaryUnavailable { name, err: _ } = err {
            assert_eq!(name, "upgrade3")
        } else {
            panic!("didn't receive expected error: {:?}", err)
        }
    }

    #[test]
    #[traced_test]
    fn test_run_and_upgrade_restart() {
        let tmp_dir = testdata::temp_dir_with(&["test_restart"]);
        let path = tmp_dir.into_path().join("test_restart");
        let bindir = Bindir::new(path.clone(), path.join("bins"), "upgrade1", "uniond").unwrap();
        let err = run_and_upgrade(
            path.clone(),
            LogFormat::Plain,
            bindir,
            vec![path.join("data").as_os_str()],
            Duration::from_secs(1),
        )
        .unwrap_err();
        if let RuntimeError::BinaryUnavailable { name, err: _ } = err {
            assert_eq!(name, "upgrade3")
        } else {
            panic!("didn't receive expected error: {:?}", err)
        }
    }

    #[test]
    #[traced_test]
    fn test_backup() {
        let tmp = testdata::temp_dir_with(&["test_backup"]);
        let home = tmp.into_path().join("test_backup");
        let supervisor: Supervisor = Supervisor::new(home.clone(), "");
        supervisor.backup(home.join("backup")).unwrap();
        assert_file_contains(home.join("backup/data/foo.db"), "foo");
        assert_file_contains(home.join("data/foo.db"), "foo");
        assert_file_contains(home.join("backup/data/bar.db"), "bar");
        assert_file_contains(home.join("data/bar.db"), "bar");
    }

    fn assert_file_contains(file: impl AsRef<Path>, want: &str) {
        let contents = fs::read_to_string(file.as_ref()).unwrap();
        assert_eq!(contents, want);
    }

    #[test]
    #[traced_test]
    fn test_revert() {
        let tmp = testdata::temp_dir_with(&["test_revert"]);
        let home = tmp.into_path().join("test_revert");
        let supervisor: Supervisor = Supervisor::new(home.clone(), "");
        supervisor.backup(home.join("backup")).unwrap();
        assert_file_contains(home.join("backup/data/foo.db"), "foo");
        std::fs::remove_file(home.join("data/foo.db")).unwrap();
        supervisor.revert(home.join("backup/data")).unwrap();
        assert_file_contains(home.join("data/foo.db"), "foo");
    }

    #[test]
    #[traced_test]
    fn test_early_exit() {
        let tmp_dir = testdata::temp_dir_with(&["test_early_exit"]);
        let home = tmp_dir.into_path().join("test_early_exit");
        assert!(home.join("bins/genesis/uniond.sh").exists());
        let bindir = Bindir::new(home.clone(), home.join("bins"), "genesis", "uniond.sh")
            .expect("should be able to create a bindir");
        assert!(bindir.current().exists());
        let err = run_and_upgrade(
            home.clone(),
            LogFormat::Plain,
            bindir,
            vec![home.join("data").as_os_str()],
            Duration::from_secs(1),
        )
        .unwrap_err();
        assert!(matches!(dbg!(err), RuntimeError::EarlyExit { .. }))
    }
}
