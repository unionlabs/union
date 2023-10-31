//! How to use flattening.
//!
//! Running this example with --help prints this message:
//! -----------------------------------------------------
//! structopt 0.3.25
//!
//! USAGE:
//!     flatten [FLAGS] -g <group> -u <user>
//!
//! FLAGS:
//!     -h, --help       Prints help information
//!     -V, --version    Prints version information
//!     -v               switch verbosity on
//!
//! OPTIONS:
//!     -g <group>        daemon group
//!     -u <user>         daemon user
//! -----------------------------------------------------

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Cmdline {
    /// switch verbosity on
    #[structopt(short)]
    verbose: bool,

    #[structopt(flatten)]
    daemon_opts: DaemonOpts,
}

#[derive(StructOpt, Debug)]
struct DaemonOpts {
    /// daemon user
    #[structopt(short)]
    user: String,

    /// daemon group
    #[structopt(short)]
    group: String,
}

fn main() {
    let opt = Cmdline::from_args();
    println!("{:?}", opt);
}
