//! How to append a postscript to the help message generated.
//!
//! Running this example with --help prints this message:
//! -----------------------------------------------------
//! structopt 0.3.25
//! I am a program and I do things.
//!
//! Sometimes they even work.
//!
//! USAGE:
//!     after_help [FLAGS]
//!
//! FLAGS:
//!     -d
//!             Release the dragon
//!
//!     -h, --help
//!             Prints help information
//!
//!     -V, --version
//!             Prints version information
//!
//!
//! Beware `-d`, dragons be here
//! -----------------------------------------------------

use structopt::StructOpt;

/// I am a program and I do things.
///
/// Sometimes they even work.
#[derive(StructOpt, Debug)]
#[structopt(after_help = "Beware `-d`, dragons be here")]
struct Opt {
    /// Release the dragon.
    #[structopt(short)]
    dragon: bool,
}

fn main() {
    let opt = Opt::from_args();
    println!("{:?}", opt);
}
