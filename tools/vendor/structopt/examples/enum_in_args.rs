//! How to use `arg_enum!` with `StructOpt`.
//!
//! Running this example with --help prints this message:
//! -----------------------------------------------------
//! structopt 0.3.25
//!
//! USAGE:
//!     enum_in_args <i>
//!
//! FLAGS:
//!     -h, --help       Prints help information
//!     -V, --version    Prints version information
//!
//! ARGS:
//!     <i>    Important argument [possible values: Foo, Bar, FooBar]
//! -----------------------------------------------------

use clap::arg_enum;
use structopt::StructOpt;

arg_enum! {
    #[derive(Debug)]
    enum Baz {
        Foo,
        Bar,
        FooBar
    }
}

#[derive(StructOpt, Debug)]
struct Opt {
    /// Important argument.
    #[structopt(possible_values = &Baz::variants(), case_insensitive = true)]
    i: Baz,
}

fn main() {
    let opt = Opt::from_args();
    println!("{:?}", opt);
}
