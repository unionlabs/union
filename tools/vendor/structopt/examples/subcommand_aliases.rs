//! How to assign some aliases to subcommands
//!
//! Running this example with --help prints this message:
//! -----------------------------------------------------
//! structopt 0.3.25
//!
//! USAGE:
//!     subcommand_aliases <SUBCOMMAND>
//!
//! FLAGS:
//!     -h, --help       Prints help information
//!     -V, --version    Prints version information
//!
//! SUBCOMMANDS:
//!     bar
//!     foo
//!     help    Prints this message or the help of the given subcommand(s)
//! -----------------------------------------------------

use structopt::clap::AppSettings;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
// https://docs.rs/clap/2/clap/enum.AppSettings.html#variant.InferSubcommands
#[structopt(setting = AppSettings::InferSubcommands)]
enum Opt {
    // https://docs.rs/clap/2/clap/struct.App.html#method.alias
    #[structopt(alias = "foobar")]
    Foo,
    // https://docs.rs/clap/2/clap/struct.App.html#method.aliases
    #[structopt(aliases = &["baz", "fizz"])]
    Bar,
}

fn main() {
    let opt = Opt::from_args();
    println!("{:?}", opt);
}
