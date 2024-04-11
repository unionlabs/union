use core::time::Duration;
use std::{
    ffi::OsString,
    fs,
    io::{self},
    path::PathBuf,
    process::Stdio,
};

use clap::Parser;
use thiserror::Error;
use tracing::{field::display as as_display, info};
use tracing_subscriber::filter::LevelFilter;

use crate::{
    bundle::{Bundle, NewBundleError, ValidateVersionPathError},
    init::{self, SetSeedsError},
    logging::LogFormat,
    supervisor::{self, RuntimeError},
    symlinker::{MakeFallbackLinkError, Symlinker},
};

#[derive(Parser, Clone)]
#[command(about = "unionvisor is a process supervisor for uniond.", long_about = None)]
/// [`Cli`]
pub struct Cli {
    /// The home directory for unionvisor, used to store unionvisor state.
    #[arg(short, long, env = "UNIONVISOR_ROOT")]
    pub root: PathBuf,

    /// The log level for unionvisor. uniond logs are piped to stdout and stderr regardless of level.
    /// and should be controlled with the commands args.
    #[arg(
        global = true,
        short,
        long,
        env = "UNIONVISOR_LOG_LEVEL",
        default_value = "INFO"
    )]
    pub log_level: LevelFilter,

    /// The log format for both unionvisor and uniond.
    #[arg(
        global = true,
        short = 'f',
        long,
        env = "UNIONVISOR_LOG_FORMAT",
        default_value = "json"
    )]
    pub log_format: LogFormat,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Clone, Parser)]
pub enum Command {
    /// Call the current binary, forwarding all arguments passed.
    /// `unionvisor call ..arg` is equivalent to `uniond ..args`.
    Call(CallCmd),

    /// Starts unionvisor, intended to be run under systemd or as a daemon
    Run(RunCmd),

    /// Initializes a local directory to join the union network.
    Init(InitCmd),
}

#[derive(Clone, Parser)]
pub struct CallCmd {
    /// Path to where the binary bundle is stored.
    #[arg(short, long, env = "UNIONVISOR_BUNDLE")]
    bundle: PathBuf,

    args: Vec<OsString>,
}

#[derive(Clone, Parser)]
pub struct InitCmd {
    /// Path to where the bundle of binaries is stored. Can be an immutable `/nix/store` dir.
    #[arg(short, long, env = "UNIONVISOR_BUNDLE")]
    bundle: PathBuf,

    /// The validator's moniker.
    #[arg(long)]
    moniker: String,

    /// The network identifier.
    #[arg(short, long)]
    network: String,

    /// Seeds to set in the config.toml.
    #[arg(long, default_value = "")]
    seeds: String,

    /// Determines if unionvisor initializes regardless of previous dirty state.
    /// This might still error depending on the behavior of the underlying uniond binary
    #[arg(short, long, default_value = "false")]
    allow_dirty: bool,
}

#[derive(Clone, Parser)]
pub struct RunCmd {
    /// Path to where the `Bundle` is stored.
    #[arg(short, long, env = "UNIONVISOR_BUNDLE")]
    bundle: PathBuf,

    /// Arguments to be directly passed to uniond.
    args: Vec<OsString>,

    /// Milliseconds in between each poll for an upgrade.
    #[arg(short, long, env = "UNIONVISOR_POLL_INTERVAL")]
    poll_interval: Option<u64>,
}

impl Cli {
    pub fn run(self) -> Result<(), RunCliError> {
        match &self.command {
            Command::Call(cmd) => {
                cmd.call(self.root)?;
                Ok(())
            }
            Command::Run(cmd) => {
                cmd.run(self.root, self.log_format)?;
                Ok(())
            }
            Command::Init(cmd) => {
                cmd.init(self.root)?;
                Ok(())
            } // Command::Merge(cmd) => cmd.merge(),
        }
    }
}

#[derive(Debug, Error)]
pub enum RunCliError {
    #[error("call command error")]
    Call(#[from] CallError),
    #[error("run command error")]
    Run(#[from] RunError),
    #[error("init command error")]
    Init(#[from] InitError),
}

/// The state that the init command left the fs in.
#[derive(PartialEq, Debug)]
pub enum InitState {
    None,
    SeedsConfigured,
}

impl InitCmd {
    fn init(&self, root: impl Into<PathBuf>) -> Result<InitState, InitError> {
        use InitError::{HomeExistsAndDirtyIsNotAllowed, SetGenesisError};
        let root = root.into();
        let home = root.join("home");

        let bundle = Bundle::new(self.bundle.clone())?;
        let symlinker = Symlinker::new(root.clone(), bundle.clone());

        if symlinker.current_validated().is_err() {
            symlinker.make_fallback_link()?;
        }

        if home.exists() {
            if self.allow_dirty {
                return Ok(InitState::None);
            }
            return Err(HomeExistsAndDirtyIsNotAllowed(home));
        }

        let init = CallCmd {
            bundle: self.bundle.clone(),
            args: vec![
                OsString::from("init"),
                OsString::from(self.moniker.clone()),
                OsString::from("--chain-id"),
                OsString::from(&self.network),
            ],
        };
        init.call_silent(root)?;
        fs::copy(bundle.genesis_json(), home.join("config/genesis.json"))
            .map_err(SetGenesisError)?;
        init::set_seeds(&self.seeds, home.join("config/config.toml"))?;

        info!(target: "unionvisor", "successfully initialized unionvisor");
        Ok(InitState::SeedsConfigured)
    }
}

#[derive(Debug, Error)]
pub enum InitError {
    #[error("cannot load bundle")]
    NewBundle(#[from] NewBundleError),
    #[error("cannot make fallback link")]
    MakeFallbackLink(#[from] MakeFallbackLinkError),
    #[error("home {0} already exists, refusing to override")]
    HomeExistsAndDirtyIsNotAllowed(PathBuf),
    #[error("set genesis error")]
    SetGenesisError(#[from] io::Error),
    #[error("set seeds error")]
    SetSeeds(#[from] SetSeedsError),
    #[error("cannot call")]
    CallError(#[from] CallError),
}

impl RunCmd {
    fn run(&self, root: impl Into<PathBuf>, logformat: LogFormat) -> Result<(), RunError> {
        let root = root.into();
        let bundle = Bundle::new(self.bundle.clone())?;
        info!(target: "unionvisor", "running with bundle: {bundle}");
        let symlinker = Symlinker::new(root.clone(), bundle);
        supervisor::run_and_upgrade(
            root,
            logformat,
            &symlinker,
            &self.args,
            Duration::from_millis(self.poll_interval.unwrap_or(6000)),
        )?;
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum RunError {
    #[error("new bundle error")]
    NewBundle(#[from] NewBundleError),
    #[error("runtime error")]
    Runtime(#[from] RuntimeError),
}

impl CallCmd {
    /// Executes the logic for the Call variant. Will panic if the enum is not [`Command::Call`].
    fn call(&self, root: impl Into<PathBuf>) -> Result<(), CallError> {
        self.call_inner(root, Stdio::inherit(), Stdio::inherit(), Stdio::inherit())
    }

    fn call_silent(&self, root: impl Into<PathBuf>) -> Result<(), CallError> {
        self.call_inner(root, Stdio::null(), Stdio::null(), Stdio::null())
    }

    fn call_inner(
        &self,
        root: impl Into<PathBuf>,
        stdin: impl Into<Stdio>,
        stdout: impl Into<Stdio>,
        stderr: impl Into<Stdio>,
    ) -> Result<(), CallError> {
        let root = root.into();
        let home = root.join("home");

        let bundle = Bundle::new(self.bundle.clone())?;
        let symlinker = Symlinker::new(root.clone(), bundle);
        let current = symlinker.current_validated()?;
        info!(target: "unionvisor",
            binary = as_display(current.0.display()),
            root = as_display(root.display()),
            "calling uniond binary at {}",
            as_display(current.0.display())
        );

        let args = if self.args.contains(&"--home".to_owned().into()) {
            self.args.clone()
        } else {
            let mut args = vec![OsString::from("--home"), home.clone().into_os_string()];
            args.extend(self.args.clone());
            args
        };

        let mut child = std::process::Command::new(&current.0)
            .args(&args)
            .stdin(stdin.into())
            .stderr(stderr.into())
            .stdout(stdout.into())
            .spawn()
            .map_err(CallError::SpawnChildProcess)?;
        child.wait().map_err(CallError::ChildExitedWithError)?;
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum CallError {
    #[error("cannot init new bundle")]
    NewBundle(#[from] NewBundleError),
    #[error("cannot validating version path")]
    ValidateVersionPath(#[from] ValidateVersionPathError),
    #[error("cannot spawn child process")]
    SpawnChildProcess(#[source] io::Error),
    #[error("child process exited with error")]
    ChildExitedWithError(#[source] io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testdata;

    /// Verifies that calling unionvisor init -i will return without impacting the fs.
    #[test]
    fn test_init_disallow_dirty_no_error() {
        let tmp = testdata::temp_dir_with(&["home", "bundle"]);
        let root = tmp.into_path();
        let state = InitCmd {
            bundle: root.join("bundle"),
            moniker: "test_init_moniker".to_owned(),
            network: "union-testnet-1".to_owned(),
            seeds: "some_seed.io".to_owned(),
            allow_dirty: true,
        }
        .init(root)
        .unwrap();
        assert_eq!(InitState::None, state);
    }

    #[test]
    fn test_init_errors_if_dirty() {
        let tmp = testdata::temp_dir_with(&["home", "bundle"]);
        let root = tmp.into_path();
        let _ = InitCmd {
            bundle: root.join("bundle"),
            moniker: "test_init_moniker".to_owned(),
            network: "union-testnet-3".to_owned(),
            seeds: "some_seed.io".to_owned(),
            allow_dirty: false,
        }
        .init(root)
        .expect_err("unionvisor should refuse to initialize if the home directory is populated");
    }
}
