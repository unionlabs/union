//! `git.rs` serves as a demonstration of how to use subcommands,
//! as well as a demonstration of adding documentation to subcommands.
//! Documentation can be added either through doc comments or
//! `help`/`about` attributes.
//!
//! Running this example with --help prints this message:
//! -----------------------------------------------------
//! git 0.3.25
//! the stupid content tracker
//!
//! USAGE:
//!     git <SUBCOMMAND>
//!
//! FLAGS:
//!     -h, --help       Prints help information
//!     -V, --version    Prints version information
//!
//! SUBCOMMANDS:
//!     add
//!     fetch    fetch branches from remote repository
//!     help     Prints this message or the help of the given subcommand(s)
//! -----------------------------------------------------

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "git")]
/// the stupid content tracker
enum Opt {
    /// fetch branches from remote repository
    Fetch {
        #[structopt(long)]
        dry_run: bool,
        #[structopt(long)]
        all: bool,
        #[structopt(default_value = "origin")]
        repository: String,
    },
    #[structopt(help = "add files to the staging area")]
    Add {
        #[structopt(short)]
        interactive: bool,
        #[structopt(short)]
        all: bool,
        files: Vec<String>,
    },
}

fn main() {
    let matches = Opt::from_args();

    println!("{:?}", matches);
}
