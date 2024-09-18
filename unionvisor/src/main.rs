#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use clap::Parser;
use color_eyre::eyre;

mod bundle;
mod cli;
mod init;
mod logging;
mod supervisor;
mod symlinker;
mod watcher;

#[cfg(test)]
mod testdata;

#[cfg(windows)]
compile_error!(
    "unionvisor heavily interacts with the fs, and hasn't been implemented to work on windows."
);

fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let cli = cli::Cli::parse();

    logging::init(cli.log_format, cli.log_level);

    cli.run()?;
    Ok(())
}
