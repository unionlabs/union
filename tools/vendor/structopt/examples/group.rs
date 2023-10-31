//! How to use `clap::Arg::group`
//!
//! Running this example with --help prints this message:
//! -----------------------------------------------------
//! structopt 0.3.25
//!
//! USAGE:
//!     group [OPTIONS] <--method <method>|--get|--head|--post|--put|--delete>
//!
//! FLAGS:
//!         --delete     HTTP DELETE
//!         --get        HTTP GET
//!     -h, --help       Prints help information
//!         --head       HTTP HEAD
//!         --post       HTTP POST
//!         --put        HTTP PUT
//!     -V, --version    Prints version information
//!
//! OPTIONS:
//!         --method <method>    Set a custom HTTP verb
//! -----------------------------------------------------

use structopt::{clap::ArgGroup, StructOpt};

#[derive(StructOpt, Debug)]
#[structopt(group = ArgGroup::with_name("verb").required(true))]
struct Opt {
    /// Set a custom HTTP verb
    #[structopt(long, group = "verb")]
    method: Option<String>,
    /// HTTP GET
    #[structopt(long, group = "verb")]
    get: bool,
    /// HTTP HEAD
    #[structopt(long, group = "verb")]
    head: bool,
    /// HTTP POST
    #[structopt(long, group = "verb")]
    post: bool,
    /// HTTP PUT
    #[structopt(long, group = "verb")]
    put: bool,
    /// HTTP DELETE
    #[structopt(long, group = "verb")]
    delete: bool,
}

fn main() {
    let opt = Opt::from_args();
    println!("{:?}", opt);
}
