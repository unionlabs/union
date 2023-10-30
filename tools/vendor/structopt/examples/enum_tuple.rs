//! How to extract subcommands' args into external structs.
//!
//! Running this example with --help prints this message:
//! -----------------------------------------------------
//! classify 0.3.25
//!
//! USAGE:
//!     enum_tuple <SUBCOMMAND>
//!
//! FLAGS:
//!     -h, --help       Prints help information
//!     -V, --version    Prints version information
//!
//! SUBCOMMANDS:
//!     foo
//!     help    Prints this message or the help of the given subcommand(s)
//! -----------------------------------------------------

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Foo {
    pub bar: Option<String>,
}

#[derive(Debug, StructOpt)]
pub enum Command {
    #[structopt(name = "foo")]
    Foo(Foo),
}

#[derive(Debug, StructOpt)]
#[structopt(name = "classify")]
pub struct ApplicationArguments {
    #[structopt(subcommand)]
    pub command: Command,
}

fn main() {
    let opt = ApplicationArguments::from_args();
    println!("{:?}", opt);
}
