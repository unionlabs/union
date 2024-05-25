use std::path::Path;
use std::process::{Child, Command};

pub fn run_with_options(options: &[&str]) -> anyhow::Result<Child> {
    let bin_path = crate::ensure_sandbox_bin()?;
    Command::new(bin_path)
        .args(options)
        .envs(crate::log_vars())
        .spawn()
        .map_err(Into::into)
}

pub fn run(home_dir: impl AsRef<Path>, rpc_port: u16, network_port: u16) -> anyhow::Result<Child> {
    let home_dir = home_dir.as_ref().to_str().unwrap();
    run_with_options(&[
        "--home",
        home_dir,
        "run",
        "--rpc-addr",
        &crate::local_addr(rpc_port),
        "--network-addr",
        &crate::local_addr(network_port),
    ])
}

pub fn init(home_dir: impl AsRef<Path>) -> anyhow::Result<Child> {
    let bin_path = crate::ensure_sandbox_bin()?;
    let home_dir = home_dir.as_ref().to_str().unwrap();
    Command::new(bin_path)
        .envs(crate::log_vars())
        .args(["--home", home_dir, "init"])
        .spawn()
        .map_err(Into::into)
}
