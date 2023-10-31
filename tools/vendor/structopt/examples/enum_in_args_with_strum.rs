//! Running this example with --help prints this message:
//! -----------------------------------------------------
//! structopt 0.3.25
//! 
//! USAGE:
//!     enum_in_args_with_strum [OPTIONS]
//! 
//! FLAGS:
//!     -h, --help       Prints help information
//!     -V, --version    Prints version information
//! 
//! OPTIONS:
//!         --format <format>     [default: txt]  [possible values: txt, md, html]
//! -----------------------------------------------------

use structopt::StructOpt;
use strum::{EnumString, EnumVariantNames, VariantNames};

const DEFAULT: &str = "txt";

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(
        long,
        possible_values = Format::VARIANTS,
        case_insensitive = true,
        default_value = DEFAULT,
    )]
    format: Format,
}

#[derive(EnumString, EnumVariantNames, Debug)]
#[strum(serialize_all = "kebab_case")]
enum Format {
    Txt,
    Md,
    Html,
}

fn main() {
    println!("{:?}", Opt::from_args());
}
