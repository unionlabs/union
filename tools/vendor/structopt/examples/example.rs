//! Somewhat complex example of usage of structopt.
//!
//! Running this example with --help prints this message:
//! -----------------------------------------------------
//! example 0.3.25
//! An example of StructOpt usage
//!
//! USAGE:
//!     example [FLAGS] [OPTIONS] <input> [--] [output]
//!
//! FLAGS:
//!     -d, --debug      Activate debug mode
//!     -h, --help       Prints help information
//!     -V, --version    Prints version information
//!
//! OPTIONS:
//!         --log <log>         Log file, stdout if no file, no logging if not present
//!         --optv <optv>...
//!     -s, --speed <speed>     Set speed [default: 42]
//!
//! ARGS:
//!     <input>     Input file
//!     <output>    Output file, stdout if not present
//! -----------------------------------------------------

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "example")]
/// An example of StructOpt usage.
struct Opt {
    // A flag, true if used in the command line.
    #[structopt(short, long)]
    /// Activate debug mode
    debug: bool,

    // An argument of type float, with a default value.
    #[structopt(short, long, default_value = "42")]
    /// Set speed
    speed: f64,

    // Needed parameter, the first on the command line.
    /// Input file
    input: String,

    // An optional parameter, will be `None` if not present on the
    // command line.
    /// Output file, stdout if not present
    output: Option<String>,

    // An optional parameter with optional value, will be `None` if
    // not present on the command line, will be `Some(None)` if no
    // argument is provided (i.e. `--log`) and will be
    // `Some(Some(String))` if argument is provided (e.g. `--log
    // log.txt`).
    #[structopt(long)]
    #[allow(clippy::option_option)]
    /// Log file, stdout if no file, no logging if not present
    log: Option<Option<String>>,

    // An optional list of values, will be `None` if not present on
    // the command line, will be `Some(vec![])` if no argument is
    // provided (i.e. `--optv`) and will be `Some(Vec<String>)` if
    // argument list is provided (e.g. `--optv a b c`).
    #[structopt(long)]
    optv: Option<Vec<String>>,

    // Skipped option: it won't be parsed and will be filled with the
    // default value for its type (in this case it'll be an empty string).
    #[structopt(skip)]
    skipped: String,
}

fn main() {
    let opt = Opt::from_args();
    println!("{:?}", opt);
}
