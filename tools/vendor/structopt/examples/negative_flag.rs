//! How to add `no-thing` flag which is `true` by default and
//! `false` if passed.
//!
//! Running this example with --help prints this message:
//! -----------------------------------------------------
//! structopt 0.3.25
//!
//! USAGE:
//!     negative_flag [FLAGS]
//!
//! FLAGS:
//!     -h, --help          Prints help information
//!     -V, --version       Prints version information
//!         --no-verbose
//! -----------------------------------------------------

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(long = "no-verbose", parse(from_flag = std::ops::Not::not))]
    verbose: bool,
}

fn main() {
    let cmd = Opt::from_args();
    println!("{:#?}", cmd);
}
