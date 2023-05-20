use clap::Parser;

mod bundle;
mod cli;
mod init;
mod logging;
mod network;
mod supervisor;
mod watcher;

#[cfg(test)]
mod testdata;

#[cfg(windows)]
compile_error!(
    "unionvisor heavily interacts with the fs, and hasn't been implemented to work on windows."
);

fn main() -> color_eyre::Result<()> {
    let cli = cli::Cli::parse();

    logging::init(cli.log_format.clone(), cli.log_level);

    if let Err(err) = cli.run() {
        tracing::error!(target: "unionvisor", error = &err.to_string().as_str(), "exited with error {}", err);
    }
    Ok(())
}
