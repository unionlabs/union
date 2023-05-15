use crate::{bindir::Bindir, init, logging::LogFormat, network::Network, supervisor};
use clap::Parser;
use color_eyre::Result;
use std::{ffi::OsString, path::PathBuf, process::Stdio, time::Duration};
use tracing::{debug, field::display as as_display};
use tracing_subscriber::filter::LevelFilter;

#[derive(Parser, Clone)]
#[command(about = "unionvisor is a process supervisor for uniond.", long_about = None)]
pub struct Cli {
    /// The home directory for unionvisor, used to store the data dir, binaries and configurations.
    #[arg(short, long, env = "UNIONVISOR_ROOT")]
    root: PathBuf,

    /// The log level for unionvisor. uniond logs are piped to stdout and stderr regardless of level.
    /// and should be controlled with the commands args.
    #[arg(short, long, env = "UNIONVISOR_LOG_LEVEL", default_value = "INFO")]
    pub log_level: LevelFilter,

    /// The log format for both unionvisor and uniond.
    #[arg(short, long, env = "UNIONVISOR_LOG_FORMAT", default_value = "json")]
    pub log_format: LogFormat,

    #[command(subcommand)]
    command: Command,
}

#[derive(Clone, Parser)]
enum Command {
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
    /// The fallback binary to use incase no symlink is found.
    #[arg(short, long, default_value = "genesis", env = "UNIONVISOR_FALLBACK")]
    fallback: String,

    /// Path to where the binaries are stored.
    #[arg(short, long, env = "UNIONVISOR_BINDIR")]
    bindir: PathBuf,

    args: Vec<OsString>,
}
#[derive(Clone, Parser)]
pub struct InitCmd {
    /// The validator's monniker.
    #[arg(short, long)]
    monniker: String,

    /// The fallback binary to use incase no symlink is found.
    #[arg(short, long, default_value = "genesis")]
    fallback: String,

    /// Path to where the binaries are stored.
    #[arg(short, long, env = "UNIONVISOR_BINDIR")]
    bindir: PathBuf,

    /// The network to create the configuration for (union-1 or union-testnet-1)
    #[arg(short, long, default_value = "union-testnet-1")]
    network: Network,
}

#[derive(Clone, Parser)]
pub struct RunCmd {
    /// Arguments to be directly passed to uniond.
    args: Vec<OsString>,

    /// The fallback binary to use incase no symlink is found.
    #[arg(short, long, default_value = "genesis", env = "UNIONVISOR_FALLBACK")]
    fallback: String,

    /// Milliseconds inbetween each poll for an upgrade.
    #[arg(short, long, env = "UNIONVISOR_POLL_INTERVAL")]
    pol_interval: Option<u64>,

    /// Path to where the binaries are stored.
    #[arg(short, long, env = "UNIONVISOR_BINDIR")]
    bindir: PathBuf,
}

impl Cli {
    pub fn run(self) -> Result<()> {
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
            }
        }
    }
}

impl InitCmd {
    fn init(&self, home: impl Into<PathBuf>) -> Result<()> {
        let home = home.into();
        let init = CallCmd {
            fallback: self.fallback.clone(),
            bindir: self.bindir.clone(),
            args: vec![
                OsString::from("--home"),
                home.clone().into_os_string(),
                OsString::from("init"),
                OsString::from(self.monniker.clone()),
                OsString::from("bn254"),
                OsString::from("--chain-id"),
                OsString::from(self.network.to_string()),
            ],
        };
        init.call_silent(home.clone())?;
        init::download_genesis(self.network, home.join("config/genesis.json"))?;
        init::set_seeds(self.network, home.join("config/config.toml"))?;
        Ok(())
    }
}

impl RunCmd {
    fn run(&self, root: impl Into<PathBuf>, logformat: LogFormat) -> Result<()> {
        let root = root.into();
        let bindir = Bindir::new(root.clone(), &self.bindir, &self.fallback)?;
        supervisor::run_and_upgrade(
            root,
            logformat,
            bindir,
            self.args.clone(),
            Duration::from_millis(self.pol_interval.unwrap_or(6000)),
        )?;
        Ok(())
    }
}

impl CallCmd {
    /// Executes the logic for the Call variant. Will panic if the enum is not Command::Call.
    fn call(&self, home: impl Into<PathBuf>) -> Result<()> {
        self.call_inner(home, Stdio::inherit(), Stdio::inherit(), Stdio::inherit())
    }

    fn call_silent(&self, home: impl Into<PathBuf>) -> Result<()> {
        self.call_inner(home, Stdio::null(), Stdio::null(), Stdio::null())
    }

    fn call_inner(
        &self,
        home: impl Into<PathBuf>,
        stdin: impl Into<Stdio>,
        stdout: impl Into<Stdio>,
        stderr: impl Into<Stdio>,
    ) -> Result<()> {
        let home = home.into();
        let bindir = Bindir::new(home.clone(), &self.bindir, &self.fallback)?;
        let current = bindir.current();
        debug!(target: "unionvisor",
            binary = as_display(current.display()),
            home = as_display(home.display()),
            "calling uniond binary at {}",
            as_display(current.display())
        );
        let mut child = std::process::Command::new(&current)
            .args(&self.args)
            .stdin(stdin.into())
            .stderr(stderr.into())
            .stdout(stdout.into())
            .spawn()?;
        child.wait()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::testdata;

    use super::*;

    #[test]
    fn test_call() {
        let tmp = testdata::temp_dir_with(&["test_call"]);
        let home = tmp.into_path().join("test_call");

        let command = CallCmd {
            args: vec![OsString::from("-f foo --r baz".to_owned())],
            fallback: "echo".to_owned(),
            bindir: home.join("bins"),
        };
        command.call_silent(home).unwrap()
    }

    #[test]
    fn test_init() {
        let tmp = testdata::temp_dir_with(&["test_init_cmd"]);
        let home = tmp.into_path().join("test_init_cmd");
        let command = InitCmd {
            monniker: String::from("test_init_monniker"),
            fallback: String::from("genesis"),
            bindir: home.join("bins"),
            network: Network::Testnet1,
        };
        command.init(home).unwrap();
    }
}
