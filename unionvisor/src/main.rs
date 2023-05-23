#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::question_mark)]

use clap::Parser;

mod bundle;
mod cli;
mod init;
mod logging;
mod network;
mod supervisor;
mod symlinker;
mod watcher;

#[cfg(test)]
mod testdata;

#[cfg(windows)]
compile_error!(
    "unionvisor heavily interacts with the fs, and hasn't been implemented to work on windows."
);

fn main() {
    let cli = cli::Cli::parse();

    logging::init(cli.log_format, cli.log_level);

    if let Err(err) = cli.run() {
        tracing::error!(target: "unionvisor", error = err.to_string().as_str(), "exited with error");
    }
}
