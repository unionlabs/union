//! A somewhat comprehensive example of a typical `StructOpt` usage.use
//!
//! Running this example with --help prints this message:
//! -----------------------------------------------------
//! basic 0.3.25
//! A basic example
//!
//! USAGE:
//!     basic [FLAGS] [OPTIONS] --output <output> [--] [FILE]...
//!
//! FLAGS:
//!     -d, --debug      Activate debug mode
//!     -h, --help       Prints help information
//!     -V, --version    Prints version information
//!     -v, --verbose    Verbose mode (-v, -vv, -vvv, etc.)
//!
//! OPTIONS:
//!     -l, --level <level>...     admin_level to consider
//!     -c, --nb-cars <nb-cars>    Number of cars
//!     -o, --output <output>      Output file
//!     -s, --speed <speed>        Set speed [default: 42]
//!
//! ARGS:
//!     <FILE>...    Files to process
//! -----------------------------------------------------

use std::path::PathBuf;
use structopt::StructOpt;

/// A basic example
#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    // A flag, true if used in the command line. Note doc comment will
    // be used for the help message of the flag. The name of the
    // argument will be, by default, based on the name of the field.
    /// Activate debug mode
    #[structopt(short, long)]
    debug: bool,

    // The number of occurrences of the `v/verbose` flag
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short, long, parse(from_occurrences))]
    verbose: u8,

    /// Set speed
    #[structopt(short, long, default_value = "42")]
    speed: f64,

    /// Output file
    #[structopt(short, long, parse(from_os_str))]
    output: PathBuf,

    // the long option will be translated by default to kebab case,
    // i.e. `--nb-cars`.
    /// Number of cars
    #[structopt(short = "c", long)]
    nb_cars: Option<i32>,

    /// admin_level to consider
    #[structopt(short, long)]
    level: Vec<String>,

    /// Files to process
    #[structopt(name = "FILE", parse(from_os_str))]
    files: Vec<PathBuf>,
}

fn main() {
    let opt = Opt::from_args();
    println!("{:#?}", opt);
}
