use std::{
    ffi::{OsStr, OsString},
    fs::create_dir_all,
    path::{Path, PathBuf},
    process::{Child, ExitStatus},
    time::Duration,
};

use color_eyre::Result;
use tracing::{debug, error, field::display as as_display, info, warn};

use crate::{
    logging::LogFormat,
    symlinker::Symlinker,
    watcher::{FileReader, FileReaderError},
};

/// A process supervisor for the uniond binary, which can start, gracefully exit and backup uniond data.
pub struct Supervisor {
    /// The path where the subprocess is called, containing configuration files and more importantly, the
    /// data dir.
    root: PathBuf,
    /// Symlinker manages the `root/current` symlink and swaps the link to a new version on upgrade
    symlinker: Symlinker,
    /// The child process that is being supervised
    child: Option<Child>,
}

impl Drop for Supervisor {
    fn drop(&mut self) {
        if self.child.is_some() {
            // Make a best effort at cleaning up processes.
            let _ = self.kill();
        }
    }
}

impl Supervisor {
    pub fn new(root: impl Into<PathBuf>, symlinker: Symlinker) -> Self {
        Self {
            root: root.into(),
            symlinker,
            child: None,
        }
    }

    /// Starts running the uniond binary in non-blocking mode.
    pub fn spawn<S: AsRef<OsStr>, I: IntoIterator<Item = S>>(
        &mut self,
        logformat: LogFormat,
        args: I,
    ) -> Result<()> {
        let program = self.symlinker.current_validated()?;
        let handle = std::process::Command::new(program.0)
            .args(vec!["--log_format", logformat.as_str()])
            .arg("start")
            .args(args)
            .args(vec![
                OsString::from("--home"),
                self.home_dir().into_os_string(),
            ])
            .stderr(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::inherit())
            .spawn()?;
        self.child = Some(handle);
        Ok(())
    }

    fn home_dir(&self) -> PathBuf {
        self.root.join("home")
    }

    /// Backup the current uniond home directory to the provided path. The location will be "{dir}/data".
    pub fn backup(&self, backup_dir: impl AsRef<Path>) -> Result<()> {
        use fs_extra::dir::{copy, CopyOptions};
        let backup_dir = backup_dir.as_ref();
        debug!(target: "unionvisor", "creating backup dir at {}",  as_display(backup_dir.display()));
        create_dir_all(backup_dir)?;
        let home_dir = self.home_dir();
        let options = CopyOptions::new().overwrite(true);
        debug!(target: "unionvisor", "backing up {} to {}",  as_display(home_dir.display()),  as_display(backup_dir.display()));
        copy(home_dir, backup_dir, &options)?;
        Ok(())
    }

    pub fn try_wait(&mut self) -> Result<Option<ExitStatus>> {
        if let Some(child) = &mut self.child {
            Ok(child.try_wait().map_err(|err| {
                debug!(target: "unionvisor", "unknown error while try_waiting for child: {:?}", err);
                err
            })?)
        } else {
            unreachable!("try_waiting for a child should only happen after spawn")
        }
    }

    pub fn kill(&mut self) -> Result<()> {
        if let Some(ref mut child) = self.child.take() {
            child.kill()?;
        } else {
            debug_assert!(false, "killing a child should only happen after spawn");
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
    root: impl Into<PathBuf>,
    logformat: LogFormat,
    symlinker: &Symlinker,
    args: &I,
    pol_interval: Duration,
) -> color_eyre::Result<(), RuntimeError> {
    let root = root.into();
    let mut supervisor = Supervisor::new(root.clone(), symlinker.clone());
    let home = supervisor.home_dir();
    let mut watcher = FileReader::new(home.join("data/upgrade-info.json"));

    info!(target: "unionvisor", "spawning supervisor process for the current uniond binary");
    supervisor.spawn(logformat, args.clone()).map_err(|err| {
        warn!(target: "supervisor", "failed to spawn initial binary call");
        err
    })?;
    info!(target: "unionvisor", "spawned uniond, starting poll for upgrade signals");
    std::thread::sleep(Duration::from_millis(300));
    loop {
        if let Some(code) = supervisor.try_wait()? {
            error!(target: "unionvisor", "uniond exited with code: {}", code);
            return Err(RuntimeError::EarlyExit { code });
        }

        match watcher.poll() {
            Err(FileReaderError::FileNotFound) | Ok(None) => continue,
            Err(err) => {
                warn!(target: "unionvisor", "unknown error while polling for upgrades: {}", err.to_string());
                return Err(RuntimeError::Other(err.into()));
            }
            Ok(Some(upgrade)) => {
                // If the daemon restarts, then upgrade-info.json may be stale. We need to
                // resolve the current symlink and compare it with the info.
                info!(
                    target = "unionvisor",
                    "detected an upgrade signal: {:?}", upgrade
                );

                // let symlink = supervisor.symlink();

                let current_version = symlinker.current_version()?;
                let upgrade_name = OsString::from(&upgrade.name);
                if current_version == upgrade_name {
                    debug!(target: "unionvisor", "detected upgrade {}, but already running that binary. sleeping for {} milliseconds.", &upgrade.name, pol_interval.as_millis());
                    std::thread::sleep(pol_interval);
                    continue;
                }

                info!(
                    target: "unionvisor",
                    name = upgrade.name.as_str(),
                    height = upgrade.height,
                    "upgrade detected"
                );
                debug!(target: "unionvisor", "checking binary availability");

                symlinker
                    .bundle
                    .path_to(&upgrade_name)
                    .validate()
                    .map_err(|err| {
                        error!(target: "unionvisor", "binary {} unavailable", &upgrade.name);
                        RuntimeError::BinaryUnavailable {
                            err,
                            name: upgrade.name.clone(),
                        }
                    })?;

                debug!(target: "unionvisor", "killing supervisor process");
                supervisor.kill()?;
                let backup_dir = root.join("home_backup");

                // If we fail to backup, the file system is incorrectly configured (permissions) or we are running
                // out of disk space. Either way we exit the node as now the server itself has become unreliable.
                debug!(target: "unionvisor", "backing up current home");
                supervisor.backup(&backup_dir)?;

                debug!(target: "unionvisor", "creating new symlink for {}", &upgrade.name);
                symlinker.swap(&upgrade_name)?;

                supervisor = Supervisor::new(root.clone(), symlinker.clone());

                // If this upgrade fails, we'll revert the local DB and exit the node, ensuring we keep the filesystem in
                // the last correct state.
                debug!(target: "unionvisor", "spawning new supervisor process for {}", &upgrade.name);
                supervisor.spawn(logformat, args.clone()).map_err(|err| {
                    error!(target: "unionvisor", err = err.to_string().as_str(), "spawning new supervisor process for {} failed", &upgrade.name);
                    // This error is most likely caused by incorrect args because of an upgrade. We can reduce the chance of that happening
                    // by introducing a configuration file with name -> args mappings.
                    err
                })?;
            }
        }
        debug!(target: "unionvisor", "no upgrade detected, sleeping for {} milliseconds.", &pol_interval.as_millis());
        std::thread::sleep(pol_interval);
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tracing_test::traced_test;

    use super::*;
    use crate::{bundle::Bundle, testdata};

    #[test]
    #[traced_test]
    /// Will keep upgrading the `current` version until it hits the signal for upgrade3,
    /// which it cannot provide.
    fn test_run_and_upgrade() {
        let tmp = testdata::temp_dir_with(&["test_run"]);
        let root = tmp.into_path().join("test_run");
        let bundle = Bundle::new(root.join("bundle")).unwrap();
        let symlinker = Symlinker::new(root.clone(), bundle);

        // Usually this is made as part of the init process, but we're not test that here.
        symlinker
            .make_fallback_link()
            .expect("fallback link should be made");

        let err = run_and_upgrade(
            root.clone(),
            LogFormat::Plain,
            &symlinker,
            &vec![root.join("home/data").as_os_str()],
            Duration::from_secs(1),
        )
        .unwrap_err();

        if let RuntimeError::BinaryUnavailable { name, err: _ } = err {
            assert_eq!(name, "upgrade3");
        } else {
            panic!("didn't receive expected error: {err:?}")
        }
    }

    #[test]
    #[traced_test]
    fn test_run_and_upgrade_restart() {
        let tmp = testdata::temp_dir_with(&["test_run"]);
        let root = tmp.into_path().join("test_run");
        let bundle = Bundle::new(root.join("bundle")).unwrap();
        let symlinker = Symlinker::new(root.clone(), bundle);

        // Usually this is made as part of the init process, but we're not test that here.
        symlinker
            .make_fallback_link()
            .expect("fallback link should be made");

        let err = run_and_upgrade(
            root.clone(),
            LogFormat::Plain,
            &symlinker,
            &vec![root.join("home/data").as_os_str()],
            Duration::from_secs(1),
        )
        .unwrap_err();

        if let RuntimeError::BinaryUnavailable { name, err: _ } = err {
            assert_eq!(name, "upgrade3");
        } else {
            panic!("didn't receive expected error: {err:?}")
        }
    }

    #[test]
    #[traced_test]
    fn test_backup() {
        let tmp = testdata::temp_dir_with(&["test_backup", "bundle"]);
        let tmp = tmp.into_path();
        let root = tmp.join("test_backup");
        let bundle = Bundle::new(tmp.join("bundle")).unwrap();
        let symlinker = Symlinker::new(root.clone(), bundle);
        let supervisor: Supervisor = Supervisor::new(root.clone(), symlinker);
        supervisor.backup(root.join("home_backup")).unwrap();
        assert_file_contains(root.join("home_backup/home/data/foo.db"), "foo");
        assert_file_contains(root.join("home/data/foo.db"), "foo");
        assert_file_contains(root.join("home_backup/home/data/bar.db"), "bar");
        assert_file_contains(root.join("home/data/bar.db"), "bar");
    }

    fn assert_file_contains(file: impl AsRef<Path>, want: &str) {
        let contents = fs::read_to_string(file.as_ref()).unwrap();
        assert_eq!(contents, want);
    }

    #[test]
    #[traced_test]
    fn test_early_exit() {
        let tmp_dir = testdata::temp_dir_with(&["test_early_exit"]);
        let root = tmp_dir.into_path().join("test_early_exit");
        let bundle = Bundle::new(root.join("bundle")).expect("should be able to create a bundle");
        let symlinker = Symlinker::new(root.clone(), bundle);

        // Usually this is made as part of the init process, but we're not test that here.
        symlinker
            .make_fallback_link()
            .expect("fallback link should be made");

        let err = run_and_upgrade(
            root.clone(),
            LogFormat::Plain,
            &symlinker,
            &vec![root.join("data").as_os_str()],
            Duration::from_secs(1),
        )
        .unwrap_err();

        assert!(matches!(dbg!(err), RuntimeError::EarlyExit { .. }));
    }
}
