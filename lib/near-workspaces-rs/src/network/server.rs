use std::fs::File;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::path::PathBuf;

use crate::error::{ErrorKind, SandboxErrorCode};
use crate::result::Result;
use crate::types::SecretKey;

use fs2::FileExt;

use near_account_id::AccountId;
use reqwest::Url;
use tempfile::TempDir;
use tokio::process::Child;

use tracing::info;

use near_sandbox_utils as sandbox;
use tokio::net::TcpListener;

// Must be an IP address as `neard` expects socket address for network address.
const DEFAULT_RPC_HOST: &str = "127.0.0.1";

fn rpc_socket(port: u16) -> String {
    format!("{DEFAULT_RPC_HOST}:{}", port)
}

/// Request an unused port from the OS.
pub async fn pick_unused_port() -> Result<u16> {
    // Port 0 means the OS gives us an unused port
    // Important to use localhost as using 0.0.0.0 leads to users getting brief firewall popups to
    // allow inbound connections on MacOS.
    let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0);
    let listener = TcpListener::bind(addr)
        .await
        .map_err(|err| ErrorKind::Io.full("failed to bind to random port", err))?;
    let port = listener
        .local_addr()
        .map_err(|err| ErrorKind::Io.full("failed to get local address for random port", err))?
        .port();
    Ok(port)
}

/// Acquire an unused port and lock it for the duration until the sandbox server has
/// been started.
async fn acquire_unused_port() -> Result<(u16, File)> {
    loop {
        let port = pick_unused_port().await?;
        let lockpath = std::env::temp_dir().join(format!("near-sandbox-port{}.lock", port));
        let lockfile = File::create(lockpath).map_err(|err| {
            ErrorKind::Io.full(format!("failed to create lockfile for port {}", port), err)
        })?;
        if lockfile.try_lock_exclusive().is_ok() {
            break Ok((port, lockfile));
        }
    }
}

#[allow(dead_code)]
async fn init_home_dir() -> Result<TempDir> {
    init_home_dir_with_version(sandbox::DEFAULT_NEAR_SANDBOX_VERSION).await
}

async fn init_home_dir_with_version(version: &str) -> Result<TempDir> {
    let home_dir = tempfile::tempdir().map_err(|e| ErrorKind::Io.custom(e))?;

    let output = sandbox::init_with_version(&home_dir, version)
        .map_err(|e| SandboxErrorCode::InitFailure.custom(e))?
        .wait_with_output()
        .await
        .map_err(|e| SandboxErrorCode::InitFailure.custom(e))?;

    info!(target: "workspaces", "sandbox init: {:?}", output);

    Ok(home_dir)
}

#[derive(Debug)]
#[non_exhaustive]
pub enum ValidatorKey {
    HomeDir(PathBuf),
    Known(AccountId, SecretKey),
}

pub struct SandboxServer {
    pub(crate) validator_key: ValidatorKey,
    rpc_addr: Url,
    net_port: Option<u16>,
    rpc_port_lock: Option<File>,
    net_port_lock: Option<File>,
    process: Option<Child>,
}

impl SandboxServer {
    /// Connect a sandbox server that's already been running, provided we know the rpc_addr
    /// and home_dir pointing to the sandbox process.
    pub(crate) async fn connect(rpc_addr: String, validator_key: ValidatorKey) -> Result<Self> {
        let rpc_addr = Url::parse(&rpc_addr).map_err(|e| {
            SandboxErrorCode::InitFailure.full(format!("Invalid rpc_url={rpc_addr}"), e)
        })?;
        Ok(Self {
            validator_key,
            rpc_addr,
            net_port: None,
            rpc_port_lock: None,
            net_port_lock: None,
            process: None,
        })
    }

    /// Run a new SandboxServer, spawning the sandbox node in the process.
    #[allow(dead_code)]
    pub(crate) async fn run_new() -> Result<Self> {
        Self::run_new_with_version(sandbox::DEFAULT_NEAR_SANDBOX_VERSION).await
    }

    pub(crate) async fn run_new_with_version(version: &str) -> Result<Self> {
        // Suppress logs for the sandbox binary by default:
        suppress_sandbox_logs_if_required();

        let home_dir = init_home_dir_with_version(version).await?.into_path();
        // Configure `$home_dir/config.json` to our liking. Sandbox requires extra settings
        // for the best user experience, and being able to offer patching large state payloads.
        crate::network::config::set_sandbox_configs(&home_dir)?;

        // Try running the server with the follow provided rpc_ports and net_ports
        let (rpc_port, rpc_port_lock) = acquire_unused_port().await?;
        let (net_port, net_port_lock) = acquire_unused_port().await?;
        // It's important that the address doesn't have a scheme, since the sandbox expects
        // a valid socket address.
        let rpc_addr = rpc_socket(rpc_port);
        let net_addr = rpc_socket(net_port);

        info!(target: "workspaces", "Starting up sandbox at localhost:{}", rpc_port);

        let options = &[
            "--home",
            home_dir
                .as_os_str()
                .to_str()
                .expect("home_dir is valid utf8"),
            "run",
            "--rpc-addr",
            &rpc_addr,
            "--network-addr",
            &net_addr,
        ];

        let child = sandbox::run_with_options_with_version(options, version)
            .map_err(|e| SandboxErrorCode::RunFailure.custom(e))?;

        info!(target: "workspaces", "Started up sandbox at localhost:{} with pid={:?}", rpc_port, child.id());

        let rpc_addr: Url = format!("http://{rpc_addr}")
            .parse()
            .expect("static scheme and host name with variable u16 port numbers form valid urls");

        Ok(Self {
            validator_key: ValidatorKey::HomeDir(home_dir),
            rpc_addr,
            net_port: Some(net_port),
            rpc_port_lock: Some(rpc_port_lock),
            net_port_lock: Some(net_port_lock),
            process: Some(child),
        })
    }

    /// Unlock port lockfiles that were used to avoid port contention when starting up
    /// the sandbox node.
    pub(crate) fn unlock_lockfiles(&mut self) -> Result<()> {
        if let Some(rpc_port_lock) = self.rpc_port_lock.take() {
            rpc_port_lock.unlock().map_err(|e| {
                ErrorKind::Io.full(
                    format!(
                        "failed to unlock lockfile for rpc_port={:?}",
                        self.rpc_port()
                    ),
                    e,
                )
            })?;
        }
        if let Some(net_port_lock) = self.net_port_lock.take() {
            net_port_lock.unlock().map_err(|e| {
                ErrorKind::Io.full(
                    format!("failed to unlock lockfile for net_port={:?}", self.net_port),
                    e,
                )
            })?;
        }

        Ok(())
    }

    pub fn rpc_port(&self) -> Option<u16> {
        self.rpc_addr.port()
    }

    pub fn net_port(&self) -> Option<u16> {
        self.net_port
    }

    pub fn rpc_addr(&self) -> String {
        self.rpc_addr.to_string()
    }
}

impl Drop for SandboxServer {
    fn drop(&mut self) {
        if let Some(mut child) = self.process.take() {
            info!(
                target: "workspaces",
                "Cleaning up sandbox: pid={:?}",
                child.id()
            );

            child.start_kill().expect("failed to kill sandbox");
            let _ = child.try_wait();
        }
    }
}

/// Turn off neard-sandbox logs by default. Users can turn them back on with
/// NEAR_ENABLE_SANDBOX_LOG=1 and specify further parameters with the custom
/// NEAR_SANDBOX_LOG for higher levels of specificity. NEAR_SANDBOX_LOG args
/// will be forward into RUST_LOG environment variable as to not conflict
/// with similar named log targets.
fn suppress_sandbox_logs_if_required() {
    if let Ok(val) = std::env::var("NEAR_ENABLE_SANDBOX_LOG") {
        if val != "0" {
            return;
        }
    }

    // non-exhaustive list of targets to suppress, since choosing a default LogLevel
    // does nothing in this case, since nearcore seems to be overriding it somehow:
    std::env::set_var("NEAR_SANDBOX_LOG", "near=error,stats=error,network=error");
}
