use crate::{
    bundle::Bundle, init, logging::LogFormat, network::Network, supervisor, symlinker::Symlinker,
};
use clap::Parser;
use color_eyre::{eyre::bail, eyre::eyre, Result};
use figment::{
    providers::{Data, Format as FigmentFormat, Json, Toml},
    Figment,
};
use serde::de::DeserializeOwned;
use std::{ffi::OsString, io::Read, path::PathBuf, process::Stdio, time::Duration};
use tracing::{debug, field::display as as_display};
use tracing_subscriber::filter::LevelFilter;

#[derive(Parser, Clone)]
#[command(about = "unionvisor is a process supervisor for uniond.", long_about = None)]
pub struct Cli {
    /// The home directory for unionvisor, used to store unionvisor state.
    #[arg(short, long, env = "UNIONVISOR_ROOT")]
    root: PathBuf,

    /// The log level for unionvisor. uniond logs are piped to stdout and stderr regardless of level.
    /// and should be controlled with the commands args.
    #[arg(short, long, env = "UNIONVISOR_LOG_LEVEL", default_value = "INFO")]
    pub log_level: LevelFilter,

    /// The log format for both unionvisor and uniond.
    #[arg(
        short = 'f',
        long,
        env = "UNIONVISOR_LOG_FORMAT",
        default_value = "json"
    )]
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

    /// Initializes a local directory to join the union network. FOOBAR
    Init(InitCmd),

    /// Merges toml or json configuration files.
    Merge(MergeCmd),
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
    #[arg(short, long)]
    moniker: String,

    /// The network to create the configuration for (union-1 or union-testnet-1)
    #[arg(short, long, default_value = "union-testnet-1")]
    network: Network,

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

/// Merges toml or json files and writes the merged output to `file`.
#[derive(Clone, Parser)]
pub struct MergeCmd {
    /// The file to use as base and write to.
    file: PathBuf,

    /// Input file to read from. If omitted, stdin is used.
    #[arg(short, long)]
    from: Option<PathBuf>,
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
            Command::Merge(cmd) => cmd.merge(),
        }
    }
}

pub trait MergeFormat {
    type Output: DeserializeOwned + ToString;
    type Format: FigmentFormat;
}

impl MergeFormat for Json {
    type Output = serde_json::Value;
    type Format = Self;
}

impl MergeFormat for Toml {
    type Output = toml::map::Map<String, toml::Value>;
    type Format = Self;
}

impl MergeCmd {
    fn merge_to_string(&self, input: String) -> Result<String> {
        let output = &self.file;
        let ext = output
            .extension()
            .ok_or(eyre!("file must have either a .json or .toml extension"))?;
        let base = std::fs::read_to_string(output)?;
        let data = match ext.to_str().unwrap() {
            "toml" => merge_inner::<Toml>(input, base)?.to_string(),
            "json" => merge_inner::<Json>(input, base)?.to_string(),
            _ => bail!("unknown extension: {:?}", ext),
        };
        Ok(data)
    }

    fn merge_from_reader_or_file<R: Read>(&self, mut r: R) -> Result<String> {
        let input = if let Some(file) = &self.from {
            std::fs::read_to_string(file)?
        } else {
            let mut buffer = Vec::new();
            r.read_to_end(&mut buffer)?;
            String::from_utf8(buffer)?
        };
        self.merge_to_string(input)
    }

    fn merge(&self) -> Result<()> {
        let output = self.merge_from_reader_or_file(std::io::stdin().lock())?;
        write_to_file(&self.file, &output)?;
        Ok(())
    }
}

fn merge_inner<F: MergeFormat>(add: String, base: String) -> Result<F::Output> {
    let value: F::Output = Figment::new()
        .merge(Data::<<F as MergeFormat>::Format>::string(&base))
        .merge(Data::<<F as MergeFormat>::Format>::string(&add))
        .extract()?;
    Ok(value)
}

fn write_to_file(path: impl Into<PathBuf>, contents: &str) -> Result<()> {
    let path = path.into();
    let mut tmp = path.clone();
    tmp.set_file_name("__unionvisor.tmp");
    let mut backup = path.clone();
    backup.set_file_name("__unionvisor.bak");
    std::fs::rename(&path, &backup)?;

    // We try writing to the temp file. If that fails, we remove the temp file and rename back the original.
    // If the write succeeds, we rename the temp file to the original, if that fails we perform the same cleanup.
    // If cleanup fails, we ignore errors and just show the original
    std::fs::write(&tmp, contents)
        .or_else(|err| {
            std::fs::remove_file(&tmp)?;
            Err(err)
        })
        .and_then(|_| std::fs::rename(&tmp, &path))
        .map_err(|err| {
            // Best effort to restore the original file
            let _ = std::fs::rename(&backup, &path);
            let _ = std::fs::remove_file(&tmp);
            err
        })?;
    std::fs::remove_file(backup)?;
    Ok(())
}

/// The state that the init command left the fs in.
#[derive(PartialEq, Debug)]
pub enum InitState {
    None,
    SeedsConfigured,
}

impl InitCmd {
    fn init(&self, root: impl Into<PathBuf>) -> Result<InitState> {
        let root = root.into();
        let home = root.join("home");

        let bundle = Bundle::new(self.bundle.clone())?;
        let symlinker = Symlinker::new(root.clone(), bundle);

        if symlinker.current_validated().is_err() {
            symlinker.make_fallback_link()?;
        }

        if home.exists() {
            if self.allow_dirty {
                return Ok(InitState::None);
            } else {
                bail!("{} already exists, refusing to override", home.display())
            }
        }

        let init = CallCmd {
            bundle: self.bundle.clone(),
            args: vec![
                OsString::from("--home"),
                home.clone().into_os_string(),
                OsString::from("init"),
                OsString::from(self.moniker.clone()),
                OsString::from("bn254"),
                OsString::from("--chain-id"),
                OsString::from(self.network.to_string()),
            ],
        };
        init.call_silent(root)?;
        init::download_genesis(self.network, home.join("config/genesis.json"))?;
        init::set_seeds(self.network, home.join("config/config.toml"))?;
        Ok(InitState::SeedsConfigured)
    }
}

impl RunCmd {
    fn run(&self, root: impl Into<PathBuf>, logformat: LogFormat) -> Result<()> {
        let root = root.into();
        let bundle = Bundle::new(self.bundle.clone())?;
        let symlinker = Symlinker::new(root.clone(), bundle);
        supervisor::run_and_upgrade(
            root,
            logformat,
            symlinker,
            self.args.clone(),
            Duration::from_millis(self.poll_interval.unwrap_or(6000)),
        )?;
        Ok(())
    }
}

impl CallCmd {
    /// Executes the logic for the Call variant. Will panic if the enum is not Command::Call.
    fn call(&self, root: impl Into<PathBuf>) -> Result<()> {
        self.call_inner(root, Stdio::inherit(), Stdio::inherit(), Stdio::inherit())
    }

    fn call_silent(&self, root: impl Into<PathBuf>) -> Result<()> {
        self.call_inner(root, Stdio::null(), Stdio::null(), Stdio::null())
    }

    fn call_inner(
        &self,
        root: impl Into<PathBuf>,
        stdin: impl Into<Stdio>,
        stdout: impl Into<Stdio>,
        stderr: impl Into<Stdio>,
    ) -> Result<()> {
        let root = root.into();
        let bundle = Bundle::new(self.bundle.clone())?;
        let symlinker = Symlinker::new(root.clone(), bundle);
        let current = symlinker.current_validated()?;
        debug!(target: "unionvisor",
            binary = as_display(current.0.display()),
            root = as_display(root.display()),
            "calling uniond binary at {}",
            as_display(current.0.display())
        );
        let mut child = std::process::Command::new(&current.0)
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
    use super::*;
    use crate::testdata;
    use tracing_test::traced_test;

    #[test]
    fn test_write_to_file() {
        let tmp = testdata::temp_dir_with(&["home"]);
        let home = tmp.into_path().join("home");
        let path = home.join("config/client.toml");
        write_to_file(&path, "hello").unwrap();
        let contents = std::fs::read_to_string(&path).unwrap();
        assert_eq!(contents, "hello")
    }

    #[test]
    fn test_merge_from_reader() {
        use toml::toml;

        let tmp = testdata::temp_dir_with(&["home"]);
        let home = tmp.into_path().join("home");

        let cmd = MergeCmd {
            file: home.join("config").join("client.toml"),
            from: None,
        };

        let input = toml! {
            broadcast-mode = "async"
            foo = "bar"
        };

        let output = cmd
            .merge_from_reader_or_file(input.to_string().as_bytes())
            .unwrap();
        let expected = toml! {
            chain-id = "union"
            keyring-backend = "os"
            output = "text"
            node = "tcp://localhost:26657"
            broadcast-mode = "async"
            foo = "bar"
        };
        assert_eq!(output, expected.to_string());
    }

    #[test]
    fn test_merge_to_string() {
        use toml::toml;

        let tmp = testdata::temp_dir_with(&["home"]);
        let home = tmp.into_path().join("home");

        let cmd = MergeCmd {
            file: home.join("config").join("client.toml"),
            from: None,
        };

        let input = toml! {
            broadcast-mode = "async"
            foo = "bar"
        };

        let output = cmd.merge_to_string(input.to_string()).unwrap();
        let expected = toml! {
            chain-id = "union"
            keyring-backend = "os"
            output = "text"
            node = "tcp://localhost:26657"
            broadcast-mode = "async"
            foo = "bar"
        };
        assert_eq!(output, expected.to_string());
    }

    #[test]
    fn test_merge_inner_json() {
        use serde_json::json;

        let base = json!({"a": true, "b": false});
        let added = json!({"b": true, "c": true});
        let result = merge_inner::<Json>(added.to_string(), base.to_string()).unwrap();
        assert_eq!(result, json!({"a": true, "b": true, "c": true}))
    }

    #[test]
    fn test_merge_inner_toml() {
        use toml::toml;

        let base = toml! {
            [package]
            name = "toml"
            version = "1"
        };

        let added = toml! {
            [package]
            name = "json"

            [dependencies]
            serde = "1.0"
        };

        let expected = toml! {
            [package]
            name = "json"
            version = "1"

            [dependencies]
            serde = "1.0"
        };
        let result = merge_inner::<Toml>(added.to_string(), base.to_string()).unwrap();
        assert_eq!(result, expected)
    }

    /// Verifies that calling unionvisor init -i will return without impacting the fs.
    #[test]
    fn test_init_disallow_dirty_no_error() {
        let tmp = testdata::temp_dir_with(&["home", "bundle"]);
        let root = tmp.into_path();
        let state = InitCmd {
            bundle: root.join("bundle"),
            moniker: String::from("test_init_moniker"),
            network: Network::Testnet1,
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
            moniker: String::from("test_init_moniker"),
            network: Network::Testnet1,
            allow_dirty: false,
        }
        .init(root)
        .expect_err("unionvisor should refuse to initialize if the home directory is populated");
    }

    #[test]
    #[ignore = "Currently cannot do networked I/O required to fetch the genesis.json inside of the sandbox"]
    #[traced_test]
    fn test_init() {
        let tmp = testdata::temp_dir_with(&["test_init_cmd"]);
        let root = tmp.into_path().join("test_init_cmd");
        let command = InitCmd {
            bundle: root.join("bundle"),
            moniker: String::from("test_init_moniker"),
            network: Network::Testnet1,
            allow_dirty: false,
        };
        command.init(root).unwrap();
    }
}
