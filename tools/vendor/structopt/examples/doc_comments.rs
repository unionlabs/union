//! How to use doc comments in place of `help/long_help`.
//!
//! Running this example with --help prints this message:
//! -----------------------------------------------------
//! basic 0.3.25
//! A basic example for the usage of doc comments as replacement of the arguments `help`, `long_help`, `about` and
//! `long_about`
//!
//! USAGE:
//!     doc_comments [FLAGS] <SUBCOMMAND>
//!
//! FLAGS:
//!     -f, --first-flag
//!             Just use doc comments to replace `help`, `long_help`, `about` or `long_about` input
//!
//!     -h, --help
//!             Prints help information
//!
//!     -s, --second-flag
//!             Split between `help` and `long_help`.
//!
//!             In the previous case structopt is going to present the whole comment both as text for the `help` and the
//!             `long_help` argument.
//!
//!             But if the doc comment is formatted like this example -- with an empty second line splitting the heading and
//!             the rest of the comment -- only the first line is used as `help` argument. The `long_help` argument will
//!             still contain the whole comment.
//!
//!             ## Attention
//!
//!             Any formatting next to empty lines that could be used inside a doc comment is currently not preserved. If
//!             lists or other well formatted content is required it is necessary to use the related structopt argument with
//!             a raw string as shown on the `third_flag` description.
//!     -t, --third-flag
//!             This is a raw string.
//!
//!             It can be used to pass well formatted content (e.g. lists or source
//!             code) in the description:
//!
//!              - first example list entry
//!              - second example list entry
//!
//!     -V, --version
//!             Prints version information
//!
//!
//! SUBCOMMANDS:
//!     first     The same rules described previously for flags. Are also true for in regards of sub-commands
//!     help      Prints this message or the help of the given subcommand(s)
//!     second    Applicable for both `about` an `help`
//! -----------------------------------------------------

use structopt::StructOpt;

/// A basic example for the usage of doc comments as replacement
/// of the arguments `help`, `long_help`, `about` and `long_about`.
#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    /// Just use doc comments to replace `help`, `long_help`,
    /// `about` or `long_about` input.
    #[structopt(short, long)]
    first_flag: bool,

    /// Split between `help` and `long_help`.
    ///
    /// In the previous case structopt is going to present
    /// the whole comment both as text for the `help` and the
    /// `long_help` argument.
    ///
    /// But if the doc comment is formatted like this example
    /// -- with an empty second line splitting the heading and
    /// the rest of the comment -- only the first line is used
    /// as `help` argument. The `long_help` argument will still
    /// contain the whole comment.
    ///
    /// ## Attention
    ///
    /// Any formatting next to empty lines that could be used
    /// inside a doc comment is currently not preserved. If
    /// lists or other well formatted content is required it is
    /// necessary to use the related structopt argument with a
    /// raw string as shown on the `third_flag` description.
    #[structopt(short, long)]
    second_flag: bool,

    #[structopt(
        short,
        long,
        long_help = r"This is a raw string.

It can be used to pass well formatted content (e.g. lists or source
code) in the description:

 - first example list entry
 - second example list entry
 "
    )]
    third_flag: bool,

    #[structopt(subcommand)]
    sub_command: SubCommand,
}

#[derive(StructOpt, Debug)]
#[structopt()]
enum SubCommand {
    /// The same rules described previously for flags. Are
    /// also true for in regards of sub-commands.
    First,

    /// Applicable for both `about` an `help`.
    ///
    /// The formatting rules described in the comment of the
    /// `second_flag` also apply to the description of
    /// sub-commands which is normally given through the `about`
    /// and `long_about` arguments.
    Second,
}

fn main() {
    let opt = Opt::from_args();
    println!("{:?}", opt);
}
