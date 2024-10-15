use std::ffi::OsString;

use clap::Parser;
use unionlabs::parse_wasm_client_type;

#[derive(Parser)]
#[command(arg_required_else_help = true)]
struct Args {
    file_path: OsString,
    /// Optionally provide a client type to expect, exiting with a non-zero status code if it's incorrect.
    expected_client_type: Option<String>,
}

fn main() {
    let args = Args::parse();

    let bz = std::fs::read(args.file_path).unwrap();

    match (parse_wasm_client_type(bz), args.expected_client_type) {
        (Some(ty), Some(expected)) => assert_eq!(ty, expected),
        (Some(ty), None) => println!("{ty}"),
        (None, _) => panic!("file does not contain a wasm client type"),
    }
}
