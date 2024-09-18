use std::{
    ffi::{OsStr, OsString},
    fs::{self, create_dir_all},
    io,
    path::{Path, PathBuf},
    process::{Child, ExitStatus},
    time::Duration,
};

use thiserror::Error;
use tracing::{error, field::display as as_display, info, warn};

use crate::{
    bundle::ValidateVersionPathError,
    logging::LogFormat,
    symlinker::{CurrentVersionError, Symlinker, SymlinkerError},
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
    ) -> Result<(), SpawnError> {
        let program = self.symlinker.current_validated()?;
        info!(
            "running {:?} pointing to {:?}",
            program.0.clone().into_os_string(),
            fs::read_link(program.0.clone())
                .expect("uniond is not a link!")
                .into_os_string()
        );
        let mut command = std::process::Command::new(program.0);
        let command = command
            .args(vec!["--log_format", logformat.as_str()])
            .arg("start")
            .args(args)
            .args(vec![
                OsString::from("--home"),
                self.home_dir().into_os_string(),
            ])
            .stderr(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::inherit());

        let child = command
            .spawn()
            .map_err(|source| SpawnError::SpawnChildError {
                source,
                command: format!("{command:?}"),
            })?;
        self.child = Some(child);
        Ok(())
    }

    fn home_dir(&self) -> PathBuf {
        self.root.join("home")
    }

    /// Backup the current uniond home directory to the provided path. The location will be "{dir}/data".
    pub fn backup(&self, backup_dir: impl AsRef<Path>) -> Result<(), BackupError> {
        use fs_extra::dir::{copy, CopyOptions};
        let backup_dir = backup_dir.as_ref();
        info!(target: "unionvisor", "creating backup dir at {}",  as_display(backup_dir.display()));
        create_dir_all(backup_dir)
            .map_err(|source| BackupError::CreateDir(backup_dir.to_owned(), source))?;
        let home_dir = self.home_dir();
        let options = CopyOptions::new().overwrite(true);
        info!(target: "unionvisor", "backing up {} to {}. This might take a while",  as_display(home_dir.display()),  as_display(backup_dir.display()));
        copy(&home_dir, backup_dir, &options).map_err(|source| BackupError::CopyDir {
            home: home_dir.clone(),
            backup: backup_dir.to_owned(),
            source,
        })?;
        info!(target: "unionvisor", "completed backup");
        Ok(())
    }

    pub fn try_wait(&mut self) -> Result<Option<ExitStatus>, TryWaitError> {
        match &mut self.child {
            Some(child) => Ok(child.try_wait()?),
            _ => unreachable!("try_waiting for a child should only happen after spawn"),
        }
    }

    pub fn kill(&mut self) -> Result<(), KillError> {
        if let Some(ref mut child) = self.child.take() {
            child.kill()?;
        } else {
            debug_assert!(false, "killing a child should only happen after spawn");
        }
        Ok(())
    }
}

#[derive(Debug, Error)]
#[error("unknown error while try_waiting for child")]
pub struct TryWaitError(#[from] io::Error);

#[derive(Debug, Error)]
#[error("unknown error while killing a child")]
pub struct KillError(#[from] io::Error);

#[derive(Debug, Error)]
pub enum SpawnError {
    #[error("error validating version path")]
    ValidateVersionPath(#[from] ValidateVersionPathError),
    #[error("error spawning child with command {command}")]
    SpawnChildError { source: io::Error, command: String },
}

#[derive(Debug, Error)]
pub enum BackupError {
    #[error("Cannot create backup dir {0}")]
    CreateDir(PathBuf, #[source] io::Error),
    #[error("Cannot copy home dir to backup dir")]
    CopyDir {
        home: PathBuf,
        backup: PathBuf,
        source: fs_extra::error::Error,
    },
}

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("error spawning uniond")]
    Spawn(#[from] SpawnError),
    #[error("error try waiting")]
    TryWait(#[from] TryWaitError),
    #[error("cannot get current version")]
    CurrentVersion(#[from] CurrentVersionError),
    #[error("cannot kill supervisor")]
    SupervisorKill(#[from] KillError),
    #[error("supervisor cannot make backup")]
    SupervisorBackup(#[from] BackupError),
    #[error("cannot swap symlink")]
    Symlinker(#[from] SymlinkerError),
    #[error("cannot validate new version's path")]
    ValidateVersionPath(#[from] ValidateVersionPathError),
    #[error("binary {} unavailable", name)]
    BinaryUnavailable {
        name: String,
        source: ValidateVersionPathError,
    },
    #[error("uniond exited with code: {code}")]
    UniondExit { code: ExitStatus },
    #[error("unknown FileReaderError while polling for upgrades")]
    FileReader(#[from] FileReaderError),
    #[error("cannot fixup legacy files")]
    Fixup(#[from] std::io::Error),
}

pub fn run_and_upgrade<S: AsRef<OsStr>, I: IntoIterator<Item = S> + Clone>(
    root: impl Into<PathBuf>,
    logformat: LogFormat,
    symlinker: &Symlinker,
    args: &I,
    pol_interval: Duration,
) -> Result<(), RuntimeError> {
    let root = root.into();
    symlinker.fix_legacy_paths()?;
    let mut supervisor = Supervisor::new(root.clone(), symlinker.clone());
    let home = supervisor.home_dir();
    let mut watcher = FileReader::new(home.join("data/upgrade-info.json"));

    info!(target: "unionvisor", "spawning supervisor process for the current uniond binary");
    supervisor
        .spawn(logformat, args.clone())
        .inspect_err(|_err| {
            warn!(target: "supervisor", "failed to spawn initial binary call");
        })?;
    info!(target: "unionvisor", "spawned uniond, starting poll for upgrade signals");
    std::thread::sleep(Duration::from_millis(300));
    loop {
        if let Some(code) = supervisor.try_wait()? {
            return Err(RuntimeError::UniondExit { code });
        }

        match watcher.poll() {
            Err(FileReaderError::FileNotFound) | Ok(None) => continue,
            Err(err) => {
                return Err(RuntimeError::FileReader(err));
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
                    info!(target: "unionvisor", "detected upgrade {}, but already running that binary. sleeping for {} milliseconds.", &upgrade.name, pol_interval.as_millis());
                    std::thread::sleep(pol_interval);
                    continue;
                }

                info!(
                    target: "unionvisor",
                    name = upgrade.name.as_str(),
                    height = upgrade.height,
                    "upgrade detected"
                );
                info!(target: "unionvisor", "checking binary availability");

                symlinker
                    .bundle
                    .path_to(&upgrade_name)
                    .validate()
                    .map_err(|source| RuntimeError::BinaryUnavailable {
                        name: upgrade.name.clone(),
                        source,
                    })?;

                info!(target: "unionvisor", "killing supervisor process");
                supervisor.kill()?;
                let backup_dir = root.join("home_backup");

                // If we fail to backup, the file system is incorrectly configured (permissions) or we are running
                // out of disk space. Either way we exit the node as now the server itself has become unreliable.
                info!(target: "unionvisor", "backing up current home");
                supervisor.backup(&backup_dir)?;

                info!(target: "unionvisor", "creating new symlink for {}", &upgrade.name);
                symlinker.swap(&upgrade_name)?;

                supervisor = Supervisor::new(root.clone(), symlinker.clone());

                // If this upgrade fails, we'll revert the local DB and exit the node, ensuring we keep the filesystem in
                // the last correct state.
                info!(target: "unionvisor", "spawning new supervisor process for {}", &upgrade.name);
                supervisor.spawn(logformat, args.clone()).inspect_err(|err| {
                    error!(target: "unionvisor", err = err.to_string().as_str(), "spawning new supervisor process for {} failed", &upgrade.name);
                    // This error is most likely caused by incorrect args because of an upgrade. We can reduce the chance of that happening
                    // by introducing a configuration file with name -> args mappings.
                })?;
            }
        }
        info!(target: "unionvisor", "no upgrade detected, sleeping for {} milliseconds.", &pol_interval.as_millis());
        std::thread::sleep(pol_interval);
    }
}

#[cfg(test)]
mod tests {
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

        if let RuntimeError::BinaryUnavailable { name, source: _ } = err {
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

        if let RuntimeError::BinaryUnavailable { name, source: _ } = err {
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

        assert!(matches!(err, RuntimeError::UniondExit { .. }));
    }
}
