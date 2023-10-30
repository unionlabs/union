//! How to require presence of at least N values,
//! like `val1 val2 ... valN ... valM`.
//!
//! Running this example with --help prints this message:
//! -----------------------------------------------------
//! structopt 0.3.25
//!
//! USAGE:
//!     at_least_two <foos>...
//!
//! FLAGS:
//!     -h, --help       Prints help information
//!     -V, --version    Prints version information
//!
//! ARGS:
//!     <foos>...
//! -----------------------------------------------------

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(required = true, min_values = 2)]
    foos: Vec<String>,
}

fn main() {
    let opt = Opt::from_args();
    println!("{:?}", opt);
}
