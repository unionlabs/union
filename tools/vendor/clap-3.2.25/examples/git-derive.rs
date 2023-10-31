use std::ffi::OsString;
use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

/// A fictional versioning CLI
#[derive(Debug, Parser)] // requires `derive` feature
#[clap(name = "git")]
#[clap(about = "A fictional versioning CLI", long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Clones repos
    #[clap(arg_required_else_help = true)]
    Clone {
        /// The remote to clone
        #[clap(value_parser)]
        remote: String,
    },
    /// pushes things
    #[clap(arg_required_else_help = true)]
    Push {
        /// The remote to target
        #[clap(value_parser)]
        remote: String,
    },
    /// adds things
    #[clap(arg_required_else_help = true)]
    Add {
        /// Stuff to add
        #[clap(required = true, value_parser)]
        path: Vec<PathBuf>,
    },
    Stash(Stash),
    #[clap(external_subcommand)]
    External(Vec<OsString>),
}

#[derive(Debug, Args)]
#[clap(args_conflicts_with_subcommands = true)]
struct Stash {
    #[clap(subcommand)]
    command: Option<StashCommands>,

    #[clap(flatten)]
    push: StashPush,
}

#[derive(Debug, Subcommand)]
enum StashCommands {
    Push(StashPush),
    Pop {
        #[clap(value_parser)]
        stash: Option<String>,
    },
    Apply {
        #[clap(value_parser)]
        stash: Option<String>,
    },
}

#[derive(Debug, Args)]
struct StashPush {
    #[clap(short, long, value_parser)]
    message: Option<String>,
}

fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::Clone { remote } => {
            println!("Cloning {}", remote);
        }
        Commands::Push { remote } => {
            println!("Pushing to {}", remote);
        }
        Commands::Add { path } => {
            println!("Adding {:?}", path);
        }
        Commands::Stash(stash) => {
            let stash_cmd = stash.command.unwrap_or(StashCommands::Push(stash.push));
            match stash_cmd {
                StashCommands::Push(push) => {
                    println!("Pushing {:?}", push);
                }
                StashCommands::Pop { stash } => {
                    println!("Popping {:?}", stash);
                }
                StashCommands::Apply { stash } => {
                    println!("Applying {:?}", stash);
                }
            }
        }
        Commands::External(args) => {
            println!("Calling out to {:?} with {:?}", &args[0], &args[1..]);
        }
    }

    // Continued program logic goes here...
}
